use gloo_utils::format::JsValueSerdeExt;
use js_sys::{Error, JsString};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use acvm::acir::circuit::OpcodeLocation;

#[wasm_bindgen(typescript_custom_section)]
const EXECUTION_ERROR: &'static str = r#"
export class ExecutionError extends Error {
    constructor(message: string, private callStack?: string[]) {
        super(message);
    }
}
"#;

// ExecutionError
#[wasm_bindgen]
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
                let call_stack: Vec<_> = call_stack.iter().map(|loc| format!("{}", loc)).collect();
                <JsValue as JsValueSerdeExt>::from_serde(&call_stack).unwrap()
            }
            None => JsValue::UNDEFINED,
        };
        JsExecutionError::new(JsString::from(message).into(), call_stack)
    }
}
