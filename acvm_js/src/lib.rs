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
const TS_APPEND_CONTENT: &'static str = r#"
// Map from witness index to hex string value of witness.
export type WitnessMap = Map<number, string>;

"#;

// WitnessMap
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Map, js_name = "WitnessMap", typescript_type = "WitnessMap")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsWitnessMap;

    /// The `clear()` method removes all elements from a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/clear)
    #[wasm_bindgen(method, js_class = "Map")]
    pub fn clear(this: &JsWitnessMap);

    /// The `delete()` method removes the specified element from a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/delete)
    #[wasm_bindgen(method, js_class = "Map")]
    pub fn delete(this: &JsWitnessMap, key: &JsValue) -> bool;

    /// The `forEach()` method executes a provided function once per each
    /// key/value pair in the Map object, in insertion order.
    /// Note that in Javascript land the `Key` and `Value` are reversed compared to normal expectations:
    /// # Examples
    /// ```
    /// let js_map = Map::new();
    /// js_map.for_each(&mut |value, key| {
    ///     // Do something here...
    /// })
    /// ```
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/forEach)
    #[wasm_bindgen(method, js_class = "Map", js_name = forEach)]
    pub fn for_each(this: &JsWitnessMap, callback: &mut dyn FnMut(JsValue, JsValue));

    /// The `get()` method returns a specified element from a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/get)
    #[wasm_bindgen(method, js_class = "Map")]
    pub fn get(this: &JsWitnessMap, key: &JsValue) -> JsValue;

    /// The `has()` method returns a boolean indicating whether an element with
    /// the specified key exists or not.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/has)
    #[wasm_bindgen(method, js_class = "Map")]
    pub fn has(this: &JsWitnessMap, key: &JsValue) -> bool;

    /// The Map object holds key-value pairs. Any value (both objects and
    /// primitive values) maybe used as either a key or a value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map)
    #[wasm_bindgen(constructor, js_class = "Map")]
    pub fn new() -> JsWitnessMap;

    /// The `set()` method adds or updates an element with a specified key
    /// and value to a Map object.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/set)
    #[wasm_bindgen(method, js_class = "Map")]
    pub fn set(this: &JsWitnessMap, key: &JsValue, value: &JsValue) -> Map;

    /// The value of size is an integer representing how many entries
    /// the Map object has. A set accessor function for size is undefined;
    /// you can not change this property.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/size)
    #[wasm_bindgen(method, js_class = "Map", getter, structural)]
    pub fn size(this: &JsWitnessMap) -> u32;
}

impl Default for JsWitnessMap {
    fn default() -> Self {
        Self::new()
    }
}
