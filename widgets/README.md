# simple-ui-widgets

Reusable [`ratatui`](https://ratatui.rs) widgets for building data-dense terminal
UIs — trading blotters, depth ladders, tapes, and charts. Split out of the
`simple-ui` `tui` demo app (per `../CHANGE_REQUESTS.md` #1) so consumers depend on
**just these widgets + ratatui**, not the demo's `tokio`/`syntect`/`portable-pty`/
`tui-textarea` tree.

This is the **v1 surface** of simple-ui (the project is TUI-only for v1; see
`../STATUS.md`). The N2 trading terminal consumes this crate directly.

```toml
[dependencies]
simple-ui-widgets = { path = "../simple-ui/widgets" }  # or a version/git ref
ratatui = "0.26"
```

## The widgets

Every widget implements `ratatui::widgets::Widget` (and `DataTable` also
`StatefulWidget`), so you render them the way you render any ratatui widget:
`frame.render_widget(widget, area)`. None of them own a render loop or any I/O —
you give them borrowed data, they draw it. That keeps them trivially testable and
lets you bind them to whatever live feed you have.

| Module | Widget | What it draws |
|---|---|---|
| `depth_ladder` | `DepthLadder` | Two-sided price ladder: asks above the mid (red), bids below (green), size bars per level, optional bot-quote overlay. |
| `time_and_sales` | `TimeAndSales` | Streaming trade tape, newest first, colored by aggressor side, backed by a bounded ring (`TapeBuffer`). |
| `data_table` | `DataTable` | Column/row grid for blotters & watchlists. Stateless, or stateful with scroll + selection + sort indicator. Draws only the visible window, so 10k+ rows are cheap. |
| `chart` | `Chart` | Candlestick chart with a zoom/pan viewport (`ChartView`) and an optional SMA overlay + price-axis gutter. |
| `candlestick` | `CandlestickChart` | The original simpler canvas-based candlestick (kept for compatibility; `chart` supersedes it). |

## Design notes

- **Data in, pixels out.** Widgets borrow your data (`&DepthData`, `&TapeBuffer`,
  `&[Vec<Cell>]`, `&[Ohlc]`) and render the current frame. State that must persist
  across frames (scroll offset, selection, zoom) lives in small `Copy`/owned
  structs *you* hold — `TableState`, `ChartView`, `TapeBuffer` — not hidden inside
  the widget.
- **Pure logic is separated and tested.** Sizing (`DataTable::column_widths`),
  scrolling (`scroll_offset`), sorting (`sort_rows`), the SMA (`sma_at`), the
  chart viewport (`ChartView::range`), and nearest-level (`nearest_index`) are
  free functions / methods you can unit-test without a terminal. `cargo test -p
  simple-ui-widgets` covers them (21 tests, incl. a 10k-row viewport test).
- **No flicker by construction.** Widgets never clear the screen or sleep; pair
  them with ratatui's double-buffered `Terminal::draw`, which diffs and writes
  only changed cells. Cap your redraw rate in your own loop (see the examples'
  frame pacer).

## Examples

Runnable, heavily-commented examples live in `../examples/` and render all four
trading widgets against a synthetic market feed — individually and as a combined
terminal:

```bash
cargo run -p widget-examples --bin depth_ladder    # one widget at a time
cargo run -p widget-examples --bin time_and_sales
cargo run -p widget-examples --bin data_table
cargo run -p widget-examples --bin chart
cargo run -p widget-examples --bin trading_terminal # all four, one screen
```

Press `q` to quit any interactive example.

### Headless mode (for quick checks by simple agents / CI)

Every example takes `--headless`: it renders a fixed number of frames of the
synthetic feed into ratatui's off-screen `TestBackend` and prints the final frame
as plain text to stdout, then exits — **no TTY required**. This is the quick-check
path an agent (or CI) uses to confirm a widget still renders, mirroring N2's
`n2-app --probe`:

```bash
cargo run -p widget-examples --bin trading_terminal -- --headless
cargo run -p widget-examples --bin depth_ladder -- --headless --frames 50
```

The desktop (React) build renders the *same* four widgets and has its own headless
check via the Playwright replay client (`../scripts/replay_client.mjs`, see
`../docs/TESTING.md`).
