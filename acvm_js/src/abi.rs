use acvm::{acir::native_types::Witness, FieldElement};
use iter_extended::{btree_map, try_btree_map};
use noirc_abi::{errors::InputParserError, input_parser::InputValue, Abi, MAIN_RETURN_NAME};
use serde::Serialize;
use std::collections::BTreeMap;

use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod temp;

use crate::JsWitnessMap;

use self::temp::{input_value_from_json_type, JsonTypes};

#[wasm_bindgen(js_name = abiEncode)]
pub fn abi_encode(
    abi: JsValue,
    inputs: JsValue,
    return_value: JsValue,
) -> Result<JsWitnessMap, JsValue> {
    console_error_panic_hook::set_once();
    let abi: Abi = JsValueSerdeExt::into_serde(&abi).map_err(|err| err.to_string())?;
    let inputs: BTreeMap<String, JsonTypes> =
        JsValueSerdeExt::into_serde(&inputs).map_err(|err| err.to_string())?;
    let return_value: Option<InputValue> = if return_value.is_undefined() || return_value.is_null()
    {
        None
    } else {
        let toml_return_value =
            JsValueSerdeExt::into_serde(&return_value).expect("could not decode return value");
        Some(
            input_value_from_json_type(
                toml_return_value,
                abi.return_type.as_ref().unwrap(),
                MAIN_RETURN_NAME,
            )
            .map_err(|err| err.to_string())?,
        )
    };

    let abi_map = abi.to_btree_map();
    let parsed_inputs: BTreeMap<String, InputValue> =
        try_btree_map(abi_map, |(arg_name, abi_type)| {
            // Check that toml contains a value for each argument in the ABI.
            let value = inputs
                .get(&arg_name)
                .ok_or_else(|| InputParserError::MissingArgument(arg_name.clone()))?;
            input_value_from_json_type(value.clone(), &abi_type, &arg_name)
                .map(|input_value| (arg_name, input_value))
        })
        .map_err(|err| err.to_string())?;

    let witness_map = abi.encode(&parsed_inputs, return_value).map_err(|err| err.to_string())?;

    Ok(witness_map.into())
}

#[wasm_bindgen(js_name = abiDecode)]
pub fn abi_decode(abi: JsValue, witness_map: JsWitnessMap) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    let abi: Abi = JsValueSerdeExt::into_serde(&abi).map_err(|err| err.to_string())?;

    let witness_map: BTreeMap<Witness, FieldElement> = witness_map.into();

    let (inputs, return_value) = abi.decode(&witness_map).map_err(|err| err.to_string())?;

    let inputs_map: BTreeMap<String, JsonTypes> =
        btree_map(inputs, |(key, value)| (key, JsonTypes::from(value)));
    let return_value = return_value.map(JsonTypes::from);

    #[derive(Serialize)]
    struct InputsAndReturn {
        inputs: BTreeMap<String, JsonTypes>,
        return_value: Option<JsonTypes>,
    }

    let return_struct = InputsAndReturn { inputs: inputs_map, return_value };
    <wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(&return_struct)
        .map_err(|err| err.to_string().into())
}
