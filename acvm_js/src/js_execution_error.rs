use acvm::acir::circuit::OpcodeLocation;
use js_sys::{Array, Error, JsString, Reflect};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const EXECUTION_ERROR: &'static str = r#"
export type ExecutionError = Error & {
    callStack?: string[];
};
"#;

/// JsExecutionError is a raw js error.
/// It'd be ideal that execution error was a subclass of Error, but for that we'd need to use JS snippets or a js module.
/// Currently JS snippets don't work with a nodejs target. And a module would be too much for just a custom error type.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Error, js_name = "ExecutionError", typescript_type = "ExecutionError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsExecutionError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    pub fn new(message: JsString) -> JsExecutionError;
}

impl JsExecutionError {
    /// Sets the call stack in an execution error.
    pub fn set_call_stack(&mut self, call_stack: Vec<OpcodeLocation>) {
        let js_array = Array::new();
        for loc in call_stack {
            js_array.push(&JsValue::from(format!("{}", loc)));
        }
        assert!(
            Reflect::set(self, &JsValue::from("callStack"), &js_array)
                .expect("Errors should be objects"),
            "Errors should be writable"
        );
    }
}
