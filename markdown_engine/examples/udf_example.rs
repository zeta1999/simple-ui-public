// examples/udf_example.rs
//
// This example demonstrates how the host language (Rust) registers
// a User Defined Function (UDF) into the spreadsheet evaluation engine.
// When the Markdown contains a cell with `=FETCH_PRICE("BTC")`, this Rust
// closure will be invoked natively.

use std::collections::HashMap;

// A simple trait/type alias representing a UDF
type UdfCallback = Box<dyn Fn(Vec<String>) -> String>;

pub struct FormulaEngine {
    udfs: HashMap<String, UdfCallback>,
}

impl Default for FormulaEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FormulaEngine {
    pub fn new() -> Self {
        Self {
            udfs: HashMap::new(),
        }
    }

    /// Register a custom host-language function
    pub fn register_udf<F>(&mut self, name: &str, func: F)
    where
        F: Fn(Vec<String>) -> String + 'static,
    {
        self.udfs.insert(name.to_uppercase(), Box::new(func));
    }

    /// Simulate evaluating a cell formula from the Markdown spreadsheet
    pub fn evaluate_cell(&self, formula: &str) -> String {
        if formula.starts_with("=FETCH_PRICE(") {
            // Simplified parsing for demonstration
            let args_str = formula
                .trim_start_matches("=FETCH_PRICE(")
                .trim_end_matches(')');
            let arg = args_str.replace("\"", "");

            if let Some(func) = self.udfs.get("FETCH_PRICE") {
                return func(vec![arg]);
            }
        }
        "#ERROR".to_string()
    }
}

fn main() {
    let mut engine = FormulaEngine::new();

    // 1. Declare and register the UDF in Rust
    engine.register_udf("FETCH_PRICE", |args| {
        let ticker = args.first().map(|s| s.as_str()).unwrap_or("");

        // Native Rust logic (e.g., querying a database or API)
        let price = match ticker {
            "BTC" => "42000.50",
            "AAPL" => "175.50",
            _ => "0.00",
        };

        println!("Rust UDF executed! Fetching price for {}", ticker);
        price.to_string()
    });

    // 2. The Markdown engine encounters a formula
    let markdown_cell = "=FETCH_PRICE(\"BTC\")";

    // 3. Evaluate it using the Rust engine
    let result = engine.evaluate_cell(markdown_cell);
    println!("Evaluation result: {}", result);
}
