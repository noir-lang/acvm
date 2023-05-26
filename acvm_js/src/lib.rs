#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

use gloo_utils::format::JsValueSerdeExt;
use js_sys::Map;
use log::Level;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

mod abi;
mod execute;
mod js_transforms;

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

#[wasm_bindgen(js_name = buildInfo)]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}

#[wasm_bindgen(typescript_custom_section)]
const WITNESS_MAP: &'static str = r#"
// Map from witness index to hex string value of witness.
export type WitnessMap = Map<number, string>;
"#;

// WitnessMap
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Map, js_name = "WitnessMap", typescript_type = "WitnessMap")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsWitnessMap;

    #[wasm_bindgen(constructor, js_class = "Map")]
    pub fn new() -> JsWitnessMap;

}

impl Default for JsWitnessMap {
    fn default() -> Self {
        Self::new()
    }
}
