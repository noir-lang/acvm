use acvm::{acir::circuit::opcodes::OracleData, FieldElement};

use js_sys::JsString;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::js_transforms::{js_value_to_field_element, field_element_to_js_string};

#[wasm_bindgen(typescript_custom_section)]
const ORACLE_CALLBACK: &'static str = r#"
/**
* A callback which performs an oracle call and returns the response as an array of outputs.
* @callback OracleCallback
* @param {string} name - The identifier for the type of oracle call being performed.
* @param {string[]} inputs - An array of hex encoded inputs to the oracle call.
* @returns {Promise<string[]>} outputs - An array of hex encoded outputs containing the results of the oracle call.
*/
export type OracleCallback = (name: string, inputs: string[]) => Promise<string[]>;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "OracleCallback")]
    pub type OracleCallback;
}

pub(super) async fn resolve_oracle(
    oracle_callback: &OracleCallback,
    mut unresolved_oracle_call: OracleData,
) -> Result<OracleData, String> {
    // Prepare to call
    let (name, inputs) = prepare_oracle_args(&unresolved_oracle_call);

    // Perform foreign call
    let outputs = perform_foreign_call(oracle_callback, name, inputs).await?;

    // Insert result into oracle data.
    unresolved_oracle_call.output_values = outputs;
    let outputs_len = unresolved_oracle_call.outputs.len();
    let output_values_len = unresolved_oracle_call.output_values.len();
    if outputs_len != output_values_len {
        return Err(format!(
            "Expected output from oracle '{}' of {} elements, but instead received {}",
            unresolved_oracle_call.name, outputs_len, output_values_len
        ));
    }

    Ok(unresolved_oracle_call)
}

fn prepare_oracle_args(unresolved_oracle_call: &OracleData) -> (JsString, js_sys::Array) {
    let name = JsString::from(unresolved_oracle_call.name.clone());

    let inputs = js_sys::Array::default();
    for input_value in &unresolved_oracle_call.input_values {
        let hex_js_string = field_element_to_js_string(input_value);
        inputs.push(&hex_js_string);
    }

    assert_eq!(unresolved_oracle_call.inputs.len(), unresolved_oracle_call.input_values.len());

    (name, inputs)
}

async fn perform_foreign_call(
    foreign_call_callback: &OracleCallback,
    name: JsString,
    inputs: js_sys::Array,
) -> Result<Vec<FieldElement>, String> {
    // Call and await
    let this = JsValue::null();
    let ret_js_val = foreign_call_callback
        .call2(&this, &name, &inputs)
        .map_err(|err| format!("Error calling `foreign_call_callback`: {}", format_js_err(err)))?;
    let ret_js_prom: js_sys::Promise = ret_js_val.into();
    let ret_future: wasm_bindgen_futures::JsFuture = ret_js_prom.into();
    let js_resolution = ret_future
        .await
        .map_err(|err| format!("Error awaiting `foreign_call_callback`: {}", format_js_err(err)))?;

    // Check that result conforms to expected shape.
    if !js_resolution.is_array() {
        return Err("`foreign_call_callback` must return a Promise<string[]>".into());
    }
    let js_arr = js_sys::Array::from(&js_resolution);

    let mut outputs = Vec::with_capacity(js_arr.length() as usize);
    for elem in js_arr.iter() {
        if !elem.is_string() {
            return Err("Non-string element in oracle_resolver return".into());
        }
        outputs.push(js_value_to_field_element(elem)?)
    }

    Ok(outputs)
}

fn format_js_err(err: JsValue) -> String {
    match err.as_string() {
        Some(str) => str,
        None => "Unknown".to_owned(),
    }
}
