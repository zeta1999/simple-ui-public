use rhai::{Dynamic, Engine, Scope};
use std::collections::HashMap;

/// A lightweight wrapper around the Rhai scripting engine tailored for evaluating
/// expressions and formulas in extended markdown documents (e.g., spreadsheets).
pub struct FormulaEvaluator {
    engine: Engine,
}

impl FormulaEvaluator {
    /// Create a new FormulaEvaluator with default built-in functions.
    pub fn new() -> Self {
        let engine = Engine::new();

        // We can register custom global functions (UDFs) here if needed.
        // For example:
        // engine.register_fn("SUM", |x: i64, y: i64| x + y);

        Self { engine }
    }

    /// Register a custom User-Defined Function (UDF) that takes a single number.
    pub fn register_udf_1<F>(&mut self, name: &str, func: F)
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        self.engine.register_fn(name, func);
    }

    /// Register a custom User-Defined Function (UDF) that takes two numbers.
    pub fn register_udf_2<F>(&mut self, name: &str, func: F)
    where
        F: Fn(f64, f64) -> f64 + Send + Sync + 'static,
    {
        self.engine.register_fn(name, func);
    }

    /// Evaluate an expression given a set of contextual variables (e.g., cell values).
    /// Variables are provided as a HashMap where keys are variable names (e.g., "A1")
    /// and values are f64 numbers.
    pub fn eval(&self, expression: &str, context: &HashMap<String, f64>) -> Result<f64, String> {
        let mut scope = Scope::new();

        for (key, value) in context {
            scope.push(key, *value);
        }

        self.engine
            .eval_with_scope::<Dynamic>(&mut scope, expression)
            .map_err(|e: Box<rhai::EvalAltResult>| e.to_string())
            .and_then(|result: Dynamic| {
                if let Ok(num) = result.as_float() {
                    Ok(num)
                } else if let Ok(num) = result.as_int() {
                    Ok(num as f64)
                } else {
                    Err("Expression did not evaluate to a number".to_string())
                }
            })
    }
}

impl Default for FormulaEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_eval() {
        let eval = FormulaEvaluator::new();
        let mut context = HashMap::new();
        context.insert("A1".to_string(), 10.0);
        context.insert("B1".to_string(), 20.0);

        let result = eval.eval("A1 + B1 * 2.0", &context).unwrap();
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_udf() {
        let mut eval = FormulaEvaluator::new();
        eval.register_udf_2("MULTIPLY", |a, b| a * b);

        let mut context = HashMap::new();
        context.insert("X".to_string(), 5.0);

        let result = eval.eval("MULTIPLY(X, 3.0)", &context).unwrap();
        assert_eq!(result, 15.0);
    }
}
