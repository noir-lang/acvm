use iter_extended::{btree_map, try_btree_map};
use noirc_abi::{errors::InputParserError, input_parser::InputValue, Abi, MAIN_RETURN_NAME};
use serde::Serialize;
use std::collections::BTreeMap;

use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod temp;

use crate::{js_map_to_witness_map, witness_map_to_js_map};

use self::temp::{input_value_from_json_type, JsonTypes};

#[wasm_bindgen]
pub fn abi_encode(abi: JsValue, inputs: JsValue, return_value: JsValue) -> js_sys::Map {
    console_error_panic_hook::set_once();
    let abi: Abi = JsValueSerdeExt::into_serde(&abi).expect("could not decode abi");
    let inputs: BTreeMap<String, JsonTypes> =
        JsValueSerdeExt::into_serde(&inputs).expect("could not decode inputs");
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
            .expect("Could not decode return value"),
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
        .expect("Could not convert from jsontypes to inputvalues");

    let witness_map = abi.encode(&parsed_inputs, return_value).expect("abi encoding error");

    witness_map_to_js_map(witness_map)
}

#[wasm_bindgen]
pub fn abi_decode(abi: JsValue, witness_map: js_sys::Map) -> JsValue {
    console_error_panic_hook::set_once();
    let abi: Abi = JsValueSerdeExt::into_serde(&abi).expect("could not decode abi");

    let witness_map = js_map_to_witness_map(witness_map);

    let (inputs, return_value) = abi.decode(&witness_map).expect("abi decoding error");

    let inputs_map: BTreeMap<String, JsonTypes> =
        btree_map(inputs, |(key, value)| (key, JsonTypes::from(value)));
    let return_value = return_value.and_then(|rv| Some(JsonTypes::from(rv)));

    #[derive(Serialize)]
    struct InputsAndReturn {
        inputs: BTreeMap<String, JsonTypes>,
        return_value: Option<JsonTypes>,
    }

    let return_struct = InputsAndReturn { inputs: inputs_map, return_value };
    <wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(&return_struct).unwrap()
}
