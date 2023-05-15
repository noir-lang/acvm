#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
use acvm::{acir::native_types::Witness, FieldElement};
use gloo_utils::format::JsValueSerdeExt;
use log::Level;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};
use wasm_bindgen::prelude::*;

mod abi;
mod execute;

pub use abi::{abi_decode, abi_encode};
pub use execute::execute_circuit;

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    git_hash: &'static str,
    version: &'static str,
    dirty: &'static str,
}

#[wasm_bindgen]
pub fn init_log_level(level: String) {
    // Set the static variable from Rust
    use std::sync::Once;

    let log_level = Level::from_str(&level).unwrap_or(Level::Error);
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        wasm_logger::init(wasm_logger::Config::new(log_level));
    });
}

const BUILD_INFO: BuildInfo = BuildInfo {
    git_hash: env!("GIT_COMMIT"),
    version: env!("CARGO_PKG_VERSION"),
    dirty: env!("GIT_DIRTY"),
};

#[wasm_bindgen]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}

fn js_map_to_witness_map(js_map: js_sys::Map) -> BTreeMap<Witness, FieldElement> {
    let mut witness_map: BTreeMap<Witness, FieldElement> = BTreeMap::new();
    js_map.for_each(&mut |value, key| {
        let witness_index = Witness(key.as_string().unwrap().parse::<u32>().unwrap());
        // let witness_value: String = js_sys::BigInt::from(value)
        //     .to_string(16)
        //     .expect("Could not get value of witness")
        //     .into();
        let witness_value: String = value.as_string().expect("Could not get value of witness");

        let witness_value =
            FieldElement::from_hex(&witness_value).expect("could not convert bigint to fields");
        witness_map.insert(witness_index, witness_value);
    });
    witness_map
}

fn witness_map_to_js_map(witness_map: BTreeMap<Witness, FieldElement>) -> js_sys::Map {
    let js_map = js_sys::Map::new();
    for (key, value) in witness_map {
        // This currently maps `0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000`
        // to the bigint `-1n`. This fails when converting back to a `FieldElement`.

        // let witness_bigint = js_sys::BigInt::from_str(&value.to_hex())
        // .expect("could not convert field to bigint");

        let witness_bigint = JsValue::from_str(&value.to_hex());

        js_map.set(
            &wasm_bindgen::JsValue::from_str(&key.witness_index().to_string()),
            &witness_bigint,
        );
    }
    js_map
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acvm::{acir::native_types::Witness, FieldElement};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    use crate::witness_map_to_js_map;

    #[wasm_bindgen_test]
    fn test_witness_map_to_js() {
        let witness_map = BTreeMap::from([
            (Witness(1), FieldElement::one()),
            (Witness(2), FieldElement::zero()),
            (Witness(3), -FieldElement::one()),
        ]);

        let js_map = witness_map_to_js_map(witness_map);

        assert_eq!(js_map.get(&JsValue::from("1")), JsValue::from_str("1"));
    }
}
