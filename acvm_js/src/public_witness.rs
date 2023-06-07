use acvm::acir::{
    circuit::Circuit,
    native_types::{Witness, WitnessMap},
};
use js_sys::JsString;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::JsWitnessMap;

fn extract_indices(witness_map: &WitnessMap, indices: Vec<Witness>) -> Result<WitnessMap, String> {
    let mut extracted_witness_map = WitnessMap::new();
    for witness in indices {
        let witness_value = witness_map.get(&witness).ok_or(format!(
            "Failed to extract witness {} from witness map. Witness not found.",
            witness.0
        ))?;
        extracted_witness_map.insert(witness, *witness_value);
    }
    Ok(extracted_witness_map)
}

#[wasm_bindgen(js_name = getReturnWitness)]
pub fn get_return_witness(
    circuit: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let witness_map = WitnessMap::from(solved_witness);

    let return_witness =
        extract_indices(&witness_map, circuit.return_values.0.into_iter().collect())?;

    Ok(JsWitnessMap::from(return_witness))
}

#[wasm_bindgen(js_name = getPublicParametersWitness)]
pub fn get_public_parameters_witness(
    circuit: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let witness_map = WitnessMap::from(solved_witness);

    let public_params_witness =
        extract_indices(&witness_map, circuit.public_parameters.0.into_iter().collect())?;

    Ok(JsWitnessMap::from(public_params_witness))
}

#[wasm_bindgen(js_name = getPublicWitness)]
pub fn get_public_witness(
    circuit: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let witness_map = WitnessMap::from(solved_witness);

    let public_witness =
        extract_indices(&witness_map, circuit.public_inputs().0.into_iter().collect())?;

    Ok(JsWitnessMap::from(public_witness))
}
