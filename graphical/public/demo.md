# Simple UI: Extended Markdown Framework

Welcome to the **Simple UI** demonstration. This framework allows you to build rich, interactive desktop and terminal applications directly from Markdown files. It leverages a blazing fast Rust core compiled to WebAssembly (WASM) for the web, and a native terminal UI (TUI) for CLI environments.

## Text Formatting

Standard Markdown features are fully supported. You can write paragraphs, lists, and code blocks natively.

- **Bold** and *italic* text
- Ordered and unordered lists
- Inline `code snippets`

```rust
// The core engine is built in Rust
fn main() {
    println!("Hello, Extended Markdown!");
}
```

## Interactive Components

The real power of Simple UI lies in its extended blocks. Below are examples of the interactive components you can embed.

### 1. Data Visualization

You can embed native charts directly. Here is a Candlestick chart tracking hypothetical stock data over a week.

```plot
{
  "type": "candlestick",
  "dates": ["Mon", "Tue", "Wed", "Thu", "Fri"],
  "open": [100.5, 102.0, 101.5, 104.0, 103.5],
  "high": [103.0, 104.5, 105.0, 106.0, 108.0],
  "low": [99.0, 101.0, 100.0, 102.5, 102.0],
  "close": [102.0, 101.5, 104.0, 103.5, 106.5],
  "title": "Weekly Tech Stock Performance"
}
```

### 2. Live Spreadsheets

Need to perform quick calculations? Embed an Excel-like spreadsheet. Try editing the quantities below; the totals will update automatically using our WASM-compiled Rhai evaluation engine!

```spreadsheet
{
  "data": [
    ["Item", "Price ($)", "Quantity", "Total"],
    ["Compute Node", 1200, 5, "=B2*C2"],
    ["Storage Array", 850, 2, "=B3*C3"],
    ["Network Switch", 400, 3, "=B4*C4"],
    ["Grand Total", "", "", "=SUM(D2:D4)"]
  ]
}
```

### 3. Quick Questions

Gather user input seamlessly with interactive forms.

```question
{
  "id": "deployment-pref",
  "question": "What is your preferred deployment target for new internal tools?",
  "options": [
    "Web Application (React/Vue)",
    "Desktop (Electron/Tauri)",
    "Terminal UI (TUI)",
    "Native Mobile App"
  ],
  "allowOther": true
}
```

---
*Built with ❤️ using Rust, WASM, and React.*
