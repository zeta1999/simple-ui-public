use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::{evaluator::FormulaEvaluator, parse_markdown};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn parse_markdown_js(input: &str) -> Result<JsValue, JsValue> {
    let doc = parse_markdown(input);
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    doc.serialize(&serializer)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub struct EvaluatorWrapper {
    inner: FormulaEvaluator,
}

#[wasm_bindgen]
impl EvaluatorWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: FormulaEvaluator::new(),
        }
    }

    /// Evaluates an expression with a given context object.
    /// The context object should be a mapping of variable names to numbers.
    #[wasm_bindgen]
    pub fn eval(&self, expression: &str, context_js: JsValue) -> Result<f64, JsValue> {
        let context: HashMap<String, f64> = serde_wasm_bindgen::from_value(context_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid context object: {}", e)))?;

        self.inner
            .eval(expression, &context)
            .map_err(|e| JsValue::from_str(&e))
    }
}

impl Default for EvaluatorWrapper {
    fn default() -> Self {
        Self::new()
    }
}
