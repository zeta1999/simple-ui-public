# Extended Markdown GUI Specification

This document details the exact text formats and schema used to define GUI elements within the Extended Markdown framework. The parser relies on standard markdown code blocks annotated with specific languages (`plot`, `question`, `spreadsheet`) to embed interactive components seamlessly into documents.

## 1. Plots (`plot`)
Used to render rich terminal charts (including candlesticks) or web-based visualizations. The content is parsed as JSON.

**Syntax Example:**
```plot
{
  "type": "candlestick",
  "title": "Daily Stock Price",
  "dates": ["2023-01-01", "2023-01-02", "2023-01-03"],
  "open": [100.5, 102.1, 101.0],
  "high": [105.0, 104.5, 106.2],
  "low": [98.0, 100.0, 99.5],
  "close": [102.1, 101.0, 105.8]
}
```

**Schema Requirements:**
- `type`: String. Supported types: `candlestick`, `line`, `bar`, `scatter`.
- `title`: (Optional) String. The title of the chart.
- Data Arrays: For candlesticks, exactly matching lengths for `dates`, `open`, `high`, `low`, and `close` are required.

## 2. Quick Questions (`question`)
Used to embed interactive forms and "quick questions" directly into the reading flow.

**Syntax Example:**
```question
{
  "id": "q_favorite_lang",
  "question": "Which language should we use for UDFs?",
  "options": ["Rust", "Golang", "C++"],
  "allowOther": true
}
```

**Schema Requirements:**
- `id`: String. A unique identifier to bind the user's answer to the application state.
- `question`: String. The prompt text displayed to the user.
- `options`: Array of Strings. The list of choices (rendered as radio buttons or a CLI menu).
- `allowOther`: (Optional) Boolean. If `true`, a free-text input box is appended for custom answers.

## 3. Spreadsheet & Formulas (`spreadsheet`)
Used to define interactive data grids with support for Excel-like formulas and host-language User Defined Functions (UDFs).

**Syntax Example:**
```spreadsheet
{
  "id": "sheet_budget",
  "data": [
    ["Item", "Cost", "Quantity", "Total"],
    ["Apple", 1.2, 5, "=B2*C2"],
    ["Orange", 0.8, 10, "=B3*C3"],
    ["Total", "", "", "=SUM(D2:D3)"],
    ["Custom", "", "", "=MY_RUST_UDF(10)"]
  ]
}
```

**Schema Requirements:**
- `id`: (Optional) String. To reference the table's state elsewhere.
- `data`: 2D Array (List of Lists). Represents the rows and columns. 
- **Formulas**: Any cell string starting with `=` is parsed as a formula. The engine supports standard arithmetic, Excel-style cell references (e.g., `B2`), and function calls.
- **UDFs**: Functions like `MY_RUST_UDF()` are intercepted by the parser and routed to the compiled host environment (Rust or Golang) for evaluation during rendering.
