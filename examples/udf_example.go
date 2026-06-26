// examples/udf_example.go
//
// This example demonstrates how the host language (Golang) registers
// a User Defined Function (UDF) into the spreadsheet evaluation engine.
// When the Markdown contains a cell with `=FETCH_PRICE("BTC")`, this Go
// function will be invoked natively.

package main

import (
	"fmt"
	"strings"
)

// UDFCallback defines the signature for a User Defined Function
type UDFCallback func(args []string) string

// FormulaEngine represents our simplistic evaluator wrapper
type FormulaEngine struct {
	udfs map[string]UDFCallback
}

func NewFormulaEngine() *FormulaEngine {
	return &FormulaEngine{
		udfs: make(map[string]UDFCallback),
	}
}

// RegisterUDF registers a custom host-language function
func (e *FormulaEngine) RegisterUDF(name string, callback UDFCallback) {
	e.udfs[strings.ToUpper(name)] = callback
}

// EvaluateCell simulates evaluating a cell formula from the Markdown spreadsheet
func (e *FormulaEngine) EvaluateCell(formula string) string {
	if strings.HasPrefix(formula, "=FETCH_PRICE(") {
		// Simplified parsing for demonstration
		argsStr := strings.TrimSuffix(strings.TrimPrefix(formula, "=FETCH_PRICE("), ")")
		arg := strings.ReplaceAll(argsStr, "\"", "")

		if callback, exists := e.udfs["FETCH_PRICE"]; exists {
			return callback([]string{arg})
		}
	}
	return "#ERROR"
}

func main() {
	engine := NewFormulaEngine()

	// 1. Declare and register the UDF in Go
	engine.RegisterUDF("FETCH_PRICE", func(args []string) string {
		ticker := ""
		if len(args) > 0 {
			ticker = args[0]
		}

		// Native Go logic (e.g., querying a database or API)
		price := "0.00"
		switch ticker {
		case "BTC":
			price = "42000.50"
		case "AAPL":
			price = "175.50"
		}

		fmt.Printf("Golang UDF executed! Fetching price for %s\n", ticker)
		return price
	})

	// 2. The Markdown engine encounters a formula
	markdownCell := "=FETCH_PRICE(\"BTC\")"

	// 3. Evaluate it using the Go engine
	result := engine.EvaluateCell(markdownCell)
	fmt.Printf("Evaluation result: %s\n", result)
}
