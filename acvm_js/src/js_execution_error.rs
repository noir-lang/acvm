use acvm::acir::circuit::OpcodeLocation;
use js_sys::{Array, Error, JsString};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const EXECUTION_ERROR: &'static str = r#"
export declare class ExecutionError extends Error {
    callStack?: string[] | undefined;
    constructor(message: string, callStack?: string[] | undefined);
}
"#;

// ExecutionError
#[wasm_bindgen(module = "src/js/executionError.js")]
extern "C" {
    #[wasm_bindgen(extends = Error, js_name = "ExecutionError", typescript_type = "ExecutionError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsExecutionError;

    #[wasm_bindgen(constructor, js_class = "ExecutionError")]
    pub fn new(message: JsString, call_stack: JsValue) -> JsExecutionError;
}

impl JsExecutionError {
    pub fn create(message: String, call_stack: Option<Vec<OpcodeLocation>>) -> JsExecutionError {
        let call_stack = match call_stack {
            Some(call_stack) => {
                let js_array = Array::new();
                for loc in call_stack {
                    js_array.push(&JsValue::from(format!("{}", loc)));
                }
                js_array.into()
            }
            None => JsValue::UNDEFINED,
        };
        JsExecutionError::new(JsString::from(message).into(), call_stack)
    }
}
