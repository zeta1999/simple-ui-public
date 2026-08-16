# Extended Markdown Project Tasks

- [x] Repository Setup
  - [x] Initialize Rust Cargo Workspace in `simple-ui` (`markdown_engine` and `tui` crates).
  - [x] Initialize Node.js Monorepo for the `graphical` apps (React/Electron).
- [x] Core Engine (Rust `markdown_engine` crate)
  - [x] Implement Markdown Parser using `pulldown-cmark`.
  - [x] Implement Formula Evaluator and UDF support in Rust.
  - [x] Setup `wasm-bindgen` to expose parsing and evaluation to JS.
- [x] IPC & Event Daemon (Rust)
  - [x] Implement HTTP Server (127.0.0.1) with Security Tokens.
  - [x] Implement UNIX Domain Socket interface.
  - [x] Build Event Stream (listen to UI inputs, emit natural events).
  - [x] API to dynamically update the `.md` AST/content.
- [x] Text Target (Rust `tui` crate)
  - [x] Setup `ratatui` application with window splits (Layouts).
  - [x] Render Markdown AST to terminal.
  - [x] Implement terminal candlestick charts.
  - [x] Add terminal emulator embedding (`portable-pty`).
  - [x] Implement Editable Text blocks with syntax highlighting (`syntect` or similar).
- [x] Graphical Target (TS/JS React)
  - [x] Integrate the Rust WASM core.
  - [x] Setup `recharts` for candlestick plots.
  - [x] Build React components for Interactive Questions and Spreadsheet.
- [x] Demonstration
  - [x] Write `example.md` showcasing all features.
  - [x] Run both the Rust TUI and the React Web/Electron targets to verify.

## Trading-terminal widgets (for the N2 consumer — see `CHANGE_REQUESTS.md`)

- [x] **Widgets-as-library split** (CR #1): new `simple-ui-widgets` crate
  (ratatui-only) with the candlestick chart; N2 depends on it (not the `tui`
  demo). Fixed a `tui-textarea`/ratatui version skew for consumers.
- [x] `cargo fmt --all` — workspace is rustfmt-clean.
- [x] Upstream **DepthLadder**, **TimeAndSales**, **DataTable**, **Chart** into
  `simple-ui-widgets` (see `STATUS.md` / `CHANGE_REQUESTS.md`).
## Still open (future improvements)

- [ ] `tui` still carries its own candlestick/editor/PTY copies instead of
  depending on `simple-ui-widgets`. Do **not** drop editor/PTY: they have
  consumers (SSH-style pipes, `simple-remote` interactive sessions). The work
  is to export a reusable PTY/editor widget from `simple-ui-widgets` so those
  crates share one implementation.
- [ ] IPC still only acks updates; it does not apply them to a live AST.
- [ ] Chart zoom/pan/overlay and a reusable data-binding helper are still
  missing (`CHANGE_REQUESTS.md` #5 / #6 / #7 panel host).
- [ ] This workspace has no sibling path deps (self-contained). That is
  intentional today; git submodules / vendoring stay deferred.
