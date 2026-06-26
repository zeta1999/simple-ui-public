# simple-ui — desktop (graphical) target

A React + Vite app, runnable in the browser or wrapped as an Electron desktop
window. It has two views (toggle in the header):

- **Terminal** — a trading terminal built from the **same four widgets** as the
  TUI examples: a candlestick chart (with SMA overlay), a depth-of-book ladder
  (with the bot's quote overlaid), a time & sales tape, and a sortable/selectable
  positions blotter. Driven by the same deterministic synthetic feed.
- **Document** — the markdown engine view (renders `public/demo.md` through the
  Rust→WASM `markdown_engine`, with interactive questions / spreadsheet / chart).

## The trading widgets — same set as the TUI

| Desktop component (`src/trading/`) | TUI twin (`simple-ui-widgets`) |
|---|---|
| `Chart.tsx` | `chart::Chart` |
| `DepthLadder.tsx` | `depth_ladder::DepthLadder` |
| `TimeAndSales.tsx` | `time_and_sales::TimeAndSales` |
| `DataTable.tsx` | `data_table::DataTable` |
| `feed.ts` | `examples/widget-examples/src/feed.rs` |

`feed.ts` is a direct port of the Rust example feed — same `noise()` function and
parameters — so the desktop terminal and the TUI terminal show the same market.

## Run it

```bash
cd graphical
npm install          # first time only
npm run dev          # browser at http://localhost:5173
# or, as a desktop window:
npm run build && npx electron .
```

## Headless check (for simple agents / CI)

No TTY/GUI needed beyond a headless browser. With the dev server running:

```bash
cd ../scripts
npm install                                              # first time (Playwright)
node screenshot.mjs out.png http://localhost:5173        # capture a PNG
node replay_client.mjs scenario_example.json             # semantic GUI dump
```

`replay_client.mjs` prints a text description of the rendered GUI (headings,
tables, inputs, buttons) and can drive a scripted scenario of clicks/typing — the
desktop counterpart of the TUI examples' `--headless` snapshot. See
`../docs/TESTING.md`.
