use acvm::acir::native_types::WitnessMap;
use js_sys::JsString;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::JsWitnessMap;

#[wasm_bindgen(js_name = compressWitness)]
pub fn compress_witness(witness_map: JsWitnessMap) -> Result<Vec<u8>, JsString> {
    console_error_panic_hook::set_once();

    let witness_map = WitnessMap::from(witness_map);
    let compressed_witness_map: Vec<u8> =
        Vec::<u8>::try_from(witness_map).map_err(|err| err.to_string())?;

    Ok(compressed_witness_map)
}

#[wasm_bindgen(js_name = decompressWitness)]
pub fn decompress_witness(compressed_witness: Vec<u8>) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();

    let witness_map =
        WitnessMap::try_from(compressed_witness.as_slice()).map_err(|err| err.to_string())?;

    Ok(witness_map.into())
}
