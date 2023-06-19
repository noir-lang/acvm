use acvm::FieldElement;

use js_sys::JsString;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::js_transforms::js_value_to_field_element;

#[wasm_bindgen(typescript_custom_section)]
const FOREIGN_CALL_HANDLER: &'static str = r#"
/**
* A callback which performs an foreign call and returns the response.
* @callback ForeignCallHandler
* @param {string} name - The identifier for the type of foreign call being performed.
* @param {string[]} inputs - An array of hex encoded inputs to the foreign call.
* @returns {Promise<string[]>} outputs - An array of hex encoded outputs containing the results of the foreign call.
*/
export type ForeignCallHandler = (name: string, inputs: string[]) => Promise<string[]>;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "ForeignCallHandler")]
    pub type ForeignCallHandler;
}

#[allow(dead_code)]
async fn perform_foreign_call(
    foreign_call_handler: &ForeignCallHandler,
    name: JsString,
    inputs: js_sys::Array,
) -> Result<Vec<FieldElement>, String> {
    // Call and await
    let this = JsValue::null();
    let ret_js_val = foreign_call_handler
        .call2(&this, &name, &inputs)
        .map_err(|err| format!("Error calling `foreign_call_callback`: {}", format_js_err(err)))?;
    let ret_js_prom: js_sys::Promise = ret_js_val.into();
    let ret_future: wasm_bindgen_futures::JsFuture = ret_js_prom.into();
    let js_resolution = ret_future
        .await
        .map_err(|err| format!("Error awaiting `foreign_call_handler`: {}", format_js_err(err)))?;

    // Check that result conforms to expected shape.
    if !js_resolution.is_array() {
        return Err("`foreign_call_handler` must return a Promise<string[]>".into());
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

#[allow(dead_code)]
fn format_js_err(err: JsValue) -> String {
    match err.as_string() {
        Some(str) => str,
        None => "Unknown".to_owned(),
    }
}
