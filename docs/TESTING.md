# Simple UI - Testing Guide

This guide provides instructions on how to manually test all targets (TUI, Desktop Application, Web, and Mobile), as well as how to run the automated CI and Headless UI replay tests.

## 1. Prerequisites
Ensure you have the following installed on your system:
- **Rust Toolchain** (cargo, rustc)
- **Node.js** (npm)
- **wasm-pack** (for compiling Rust to WASM)

---

## 1b. Trading widget examples (`simple-ui-widgets`)

The reusable widgets live in the `simple-ui-widgets` crate and have runnable
examples in `examples/widget-examples` — one per widget plus a combined terminal.
Each example runs **interactively** or **headless**.

**Interactive (live TUI, `q` quits):**
```bash
cargo run -p widget-examples --bin trading_terminal   # all four widgets
cargo run -p widget-examples --bin depth_ladder        # or one at a time:
cargo run -p widget-examples --bin time_and_sales      #   time_and_sales
cargo run -p widget-examples --bin data_table          #   data_table
cargo run -p widget-examples --bin chart               #   chart
```

**Headless (no TTY — for scripted agents / CI):** `--headless` advances the
deterministic feed a fixed number of frames, renders one frame to an off-screen
buffer, and prints it as plain text. The output is stable run-to-run, so it can
be snapshot-diffed.
```bash
cargo run -p widget-examples --bin trading_terminal -- --headless
cargo run -p widget-examples --bin depth_ladder -- --headless --frames 50 --size 48x16
```
Flags: `--headless`, `--frames N`, `--size WxH`, `--fps N` (interactive).

**What's covered by `cargo test`:** the widgets' pure logic — column sizing,
scroll-offset math, numeric/lexicographic sort, the SMA, the chart viewport,
nearest-level, and a 10k-row viewport test (`cargo test -p simple-ui-widgets`,
21 tests). **What still needs a human:** on-screen smoothness / "no flicker" and
visual polish — eyeball the interactive examples in a real terminal.

---

## 2. Terminal User Interface (TUI)
The TUI provides a native console interface using `ratatui`.

**To run the TUI:**
1. Navigate to the root of the project.
2. Ensure you have a valid markdown file (e.g., `docs/demo.md`). You can use the provided demo file in the `graphical/public` folder.
3. Run the following command:
   ```bash
   cargo run -p tui -- --file graphical/public/demo.md
   ```
4. **Verification**: 
   - You should see a terminal application split into panes.
   - Use the `Left`/`Right` arrow keys to navigate between the different tabs (Dashboard, Financials, Terminal).
   - Press `ESC` to quit the application.

---

## 3. Desktop Application (Electron)
The desktop application embeds the React front-end inside an Electron window.

**To run the Desktop App:**
1. First, make sure the WASM engine is built:
   ```bash
   make build-wasm
   ```
   *(This runs `wasm-pack build --target web` in the `markdown_engine` directory.)*
2. Navigate into the graphical folder and install dependencies:
   ```bash
   cd graphical
   npm install
   ```
3. Start the Electron/Vite dev server:
   ```bash
   npm run dev
   ```
4. **Verification**:
   - An Electron window will launch automatically.
   - The header has a **Terminal / Document** toggle. **Terminal** (the default)
     shows the trading widgets — the same set as the TUI examples (chart, depth
     ladder, time & sales, positions blotter) — animating off the synthetic feed;
     click a blotter header to sort and a row to select. **Document** shows the
     markdown engine view.
   - On the Document view, ensure the markdown parses properly and the styling is
     dark-mode premium. Try clicking a cell in the "Interactive Spreadsheet", type
     a new number, and press Enter. The "Total" column should recalculate.

---

## 4. Phone/Tablet App (Mobile Web Responsive)
The same Vite codebase used for Electron also runs natively as a responsive web app that you can access from your phone or tablet on the same local network.

**To run the Mobile Web View:**
1. Keep the Vite dev server running from the previous step. Because we configured Vite with `--host` (or `host: true`), it listens on your local IP address.
2. Look at your terminal output for a line similar to:
   ```text
   ➜  Network: http://192.168.1.XX:5173/
   ```
3. Open a browser on your phone or tablet (must be connected to the same Wi-Fi network) and enter that URL.
4. **Verification**:
   - The UI should scale down responsively.
   - Verify that the layout remains functional and the candlestick chart still renders on a smaller screen.
   - Test interaction with the "Quick Questions" radio buttons using touch inputs.

---

### Troubleshooting
- **WASM Initialization Failure**: If you see a blank screen or a "Loading Core Engine..." message stuck forever, ensure `make build-wasm` succeeded and that `npm install` finished successfully in the `graphical` directory.
- **TUI Errors**: Ensure your terminal emulator supports raw mode and 256 colors. For the best experience, use iTerm2, Alacritty, or the standard MacOS Terminal.

---

## 5. Automated CI Pipeline
We use a custom, local CI pipeline to enforce code quality without relying on GitHub Actions.
1. Run the CI script from the project root:
   ```bash
   bash scripts/ci.sh
   ```
2. **What it does**:
   - Formats Rust code (`cargo fmt --all -- --check`).
   - Runs the Rust linter (`cargo clippy --all-targets --all-features -- -D warnings`).
   - Runs the Rust test suite (`cargo test --workspace`).
   - Builds the Graphical target to ensure TypeScript and Vite compile correctly.

---

## 6. Headless Replay Testing
For testing the React/Web GUI with a low-resource agent, we provide a Playwright-based headless client that injects a scenario of events and produces a semantic description of the GUI state.

**To run the Headless Replay Client:**
1. Ensure the `graphical` dev server is running (`npm run dev` in the `graphical` folder).
2. Install the Playwright dependencies in the `scripts` directory (only required once):
   ```bash
   cd scripts
   npm install
   npx playwright install chromium
   ```
3. Run the replay client passing in a JSON scenario:
   ```bash
   node scripts/replay_client.mjs scripts/scenario_example.json
   ```
4. **Verification**:
   - The script will launch a headless browser and navigate to `http://localhost:5173`.
   - It will apply the events (e.g., clicking radio buttons or typing numbers in the spreadsheet) from the JSON file.
   - You will see the `=== GUI STATE ===` dumped to stdout as a clean semantic tree, describing headings, tables, forms, and buttons available on the screen at each step.

### 6.1 Authoring a scenario

A scenario is a JSON **array of events**, applied in order. Each event is:

```jsonc
{ "action": "click", "target": "<selector>" }              // click an element
{ "action": "type",  "target": "<selector>", "value": "9" } // fill an input, then press Enter
```

- **`target`** is a [Playwright selector](https://playwright.dev/docs/selectors).
  The simplest is `text=<visible text>` (e.g. `text=Submit Response`, `text=PnL`);
  CSS works too (e.g. `.grid th:nth-child(5)`, `input[name="q1"]`).
- After each event the script waits 500ms and re-dumps the GUI state, so the
  output is a **step-by-step trace** of how the UI responded.

Two ready-made scenarios live next to the client:
- `scenario_example.json` — the markdown/Document view (radio + spreadsheet).
- `scenario_terminal.json` — the trading Terminal view: sort the blotter by
  **PnL**, then by **Symbol**, then switch to the **Document** view.

```bash
node scripts/replay_client.mjs scripts/scenario_terminal.json
```

### 6.2 Discovering selectors / inspecting state (empty scenario)

You don't have to guess selectors. Run the client with an **empty scenario**
(`[]`) — it dumps the *initial* GUI state and exits without doing anything:

```bash
echo '[]' > /tmp/inspect.json
node scripts/replay_client.mjs /tmp/inspect.json
```

Read the `=== GUI STATE ===` block to see the headings, table rows, inputs (with
`name=`), and button labels actually on screen, then use those texts/names as
your `target`s. This is the recommended loop: **inspect → write a step → re-run →
read the trace → add the next step.**

> **What the trace captures:** headings (`h1–h3`), radios/inputs, `<table>`
> contents, and buttons — enough to assert on the blotter grid and the
> view-switch / form controls. The depth ladder and tape are purely visual
> (`<div>`s); to *see* them rendered, capture a PNG instead:
> `node scripts/screenshot.mjs out.png http://localhost:5173`.

### 6.3 Relationship to the TUI headless mode

This is the **desktop** quick-check. The **TUI** equivalent is each widget
example's `--headless` flag (see §1b), which prints a deterministic text frame to
stdout. Both let an agent verify the same widget set without a real display.
