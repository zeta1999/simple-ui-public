# Extended Markdown GUI Framework

This project will build a framework that allows you to create and update simple GUIs directly from an extended Markdown specification. The framework supports basic charts (including candlesticks), tabs, menus, interactive questions, and tables with Excel-like formulas. UDFs are declared in the host language (Rust). At compile time, you can choose to target Electron, React Native, or a native TUI. The TUI target additionally supports window splitting and embedding external TUI apps.

## Proposed Architecture: Core + Multi-Frontend

To support both the rich text-only target (with pseudo-windows and terminal embedding) AND the graphical targets (Electron and React Native), we will use a decoupled architecture:

## Proposed Architecture: Rust Core + WASM + Multi-Frontend

To support the rich text-only target (with pseudo-windows and terminal embedding) AND the graphical targets (Electron and React Native) without using Python, we will use a decoupled Rust-centric architecture:

1. **Core Engine (Logic & Parsing) in Rust**:
   - The core logic will be written in **Rust** (or Golang, but Rust offers superior WASM bindings via `wasm-bindgen`).
   - **Markdown Parser**: Uses `pulldown-cmark` (Rust) to parse standard text and our custom blocks (`plot`, `question`, `spreadsheet`) into a generic Abstract Syntax Tree (AST).
   - **Formula Evaluator**: A Rust-based expression evaluator to process Excel-like calculations and UDFs securely and extremely fast.
   - **WASM Interop**: The Rust core will be compiled to WebAssembly (WASM) so the exact same parsing and formula evaluation logic runs natively in the browser/Electron and Node.js.

2. **Text-Only Target (TUI) in Rust**:
   - Built entirely in Rust using the `ratatui` framework. This provides a high-performance, Bloomberg-terminal-like interface.
   - **Window Splits**: `ratatui`'s native `Layout` constraints handle split panes and pseudo-windows easily.
   - **Terminal Embedding**: Using a Rust PTY library (like `portable-pty` + `vt100` parsing) to embed interactive sub-shells within a TUI pane.
   - **Charts**: Rendering native terminal candlestick plots using `ratatui` canvas or third-party charting extensions.

3. **Graphical/UX Targets (TS/JS)**:
   - **Desktop**: **Electron** + React (TypeScript). Consumes the Rust WASM core for parsing/logic. Uses `recharts` for beautiful candlestick charts, and interactive web grids for the spreadsheet.
   - **Mobile/Cross-Platform**: **React Native**. Can consume the shared Core logic (via a native Rust bridge or WASM if supported) and renders native mobile components.

4. **IPC & Event Streaming (Local Daemon)**:
   - **Interface**: Exposes a local IPC server (HTTP with sec tokens bound to `127.0.0.1` and/or Unix Domain Sockets).
   - **Two-Way Sync**: External programs can push updates to the `.md` content dynamically.
   - **Event Emitter**: Emits UI events (keyboard inputs, form submissions, cursor movements) to the connected external program.

## Proposed Extensions

### 1. Terminal Plots (including Candlesticks)
Using Rust-based terminal charting (e.g., `tui-rs` / `ratatui` chart extensions or custom canvas drawing) to render responsive plots directly in the TUI.
**Syntax:**
```plot
{
  "type": "candlestick",
  "dates": ["2023-01-01", "2023-01-02", "2023-01-03"],
  "open": [100, 102, 101],
  "high": [105, 104, 106],
  "low": [98, 100, 99],
  "close": [102, 101, 105],
  "title": "Stock Price"
}
```

### 2. Simple Interactions (Quick Questions)
Interactive forms and "quick questions a la Claude Code" rendered natively within the TUI or Graphical targets.
**Syntax:**
```question
{
  "id": "q1",
  "question": "Choose an option:",
  "options": ["1", "2", "3", "4"],
  "allowOther": true
}
```
*Behavior*: The app renders a radio set or quick selection menu, optionally allowing free text input for "other".

### 3. Formulas & Spreadsheet Tables
Using `ratatui` Table widgets for the TUI, and a Rust-based expression parser for the engine to evaluate cell formulas securely, allowing custom UDFs written in Rust or Golang.
**Syntax:**
```spreadsheet
{
  "data": [
    ["Item", "Cost", "Quantity", "Total"],
    ["Apple", 1.2, 5, "=B2*C2"],
    ["Orange", 0.8, 10, "=B3*C3"],
    ["Total", "", "", "=SUM(D2:D3)"],
    ["Custom", "", "", "=MY_UDF(10)"]
  ]
}
```
*Behavior*: The engine resolves formulas (e.g., `B2` and `C2`), calls registered UDFs (like `MY_UDF`), and populates the interactive tables.

## User Interface & Features
- **Tabs / Menu Bar**: The application supports multiple markdown documents and layouts natively via TUI pane management and tabs.
- **Event Handling**: We will include global key bindings (e.g., `F1` or `?` for Help, `q` to quit) mapping directly to TUI actions.
- **Editable Text Elements**: Specific text blocks can be made editable directly in the UI, featuring configurable syntax highlighting. "Natural events" (e.g., text saved, block edited) are streamed back via the IPC interface.

## User Review Required

> [!IMPORTANT]
> Please review the finalized Rust-centric design decisions:
> 1. **Rust Framework**: The core is built in Rust to support WASM and high-performance TUI. Python is completely removed. UDFs will be declared in Rust (or Golang via FFI/RPC if needed, but native Rust is preferred for WASM compilation).
> 2. **Layout Structure**: We plan to use `ratatui` for the text target with splits, embedded PTYs, and native charts. 

## Verification Plan

1. Setup a Rust Cargo Workspace with `core` and `tui` crates.
2. Scaffold a `ratatui` app with a Header, Footer, and Tabbed layout.
3. Build the markdown parser (`pulldown-cmark`) that translates custom code blocks into AST structures.
4. Register a sample Rust UDF.
5. Create an `example.md` containing all features.
6. Run `cargo run` and manually verify the candlestick chart renders, quick questions are interactive, formulas evaluate correctly, and window splits function properly.
