# simple-ui

Terminal UI for tools and agents: a widget library plus a markdown engine you can drive in real time.

Part of [**simple tools**](https://zeta1999.github.io/renoir42/simple-tools.html) — small, composable Rust libraries for building tooling fast from a harness.

## What's here

- **`widgets`** — a TUI widget set: `DataTable` (scroll / select / sort), `DepthLadder`, `TimeAndSales`, `Chart`, a `NewsPanel`, and an `AgentLog` for streaming AI-agent activity. Many were upstreamed from a trading terminal into a reusable crate.
- **`markdown_engine`** — renders interactive markdown documents. A unix-domain-socket IPC path lets a process push AST updates into a live view, and a formula evaluator (with WASM bindings) supports user-defined functions.
- **`tui`** — the rendering layer that ties the widgets and engine together.

**TUI-first.** v1 targets the terminal; the graphical target is parked rather than half-shipped. For agent work this is the layer a model can render to and update as it goes.

## Layout

```
widgets/          reusable TUI widgets
markdown_engine/  interactive markdown + formula evaluator
tui/              terminal rendering layer
examples/         runnable demos
```

## Build

```sh
cargo build --workspace
cargo run -p examples --bin full_tui
```

## License

MIT OR Apache-2.0
