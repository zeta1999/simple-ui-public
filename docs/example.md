# Extended Markdown GUI Example

Welcome to the **Extended Markdown GUI**! This document serves as both the specification and the living UI. 

Because the TUI and graphical targets render this file live, any edits you make via the IPC interface will instantly update the interface.

---

## 1. Application Layouts & Tabs

You can define layout structures using `layout` blocks. This allows splitting windows and creating tabs.

```layout
{
  "type": "tabs",
  "active_tab": "Dashboard",
  "tabs": [
    { "name": "Dashboard", "content_ref": "#dashboard-content" },
    { "name": "Financials", "content_ref": "#financial-content" },
    { "name": "Terminal", "content_ref": "#terminal-content" }
  ]
}
```

---

## 2. Interactive Forms (Quick Questions)

<div id="dashboard-content">

Here is a "quick question a la Claude Code". Selecting an option will emit an event over the IPC socket.

```question
{
  "id": "q_deployment",
  "question": "Which target are you deploying to?",
  "options": ["TUI", "Electron", "React Native"],
  "allowOther": true
}
```
</div>

---

## 3. Financial Dashboards & Plots

<div id="financial-content">

We can embed advanced Bloomberg-terminal style charts directly into the document. 

```plot
{
  "type": "candlestick",
  "title": "TSLA Weekly Action",
  "dates": ["2023-10-01", "2023-10-08", "2023-10-15"],
  "open": [250.0, 245.5, 260.1],
  "high": [265.0, 260.0, 270.5],
  "low": [240.0, 240.5, 255.0],
  "close": [245.5, 260.1, 268.0]
}
```

You can also use simple bar charts:
```plot
{
  "type": "bar",
  "title": "Q4 Revenue",
  "labels": ["Oct", "Nov", "Dec"],
  "data": [12000, 15000, 18000]
}
```

## 4. Spreadsheets with Rust UDFs

Below is a live spreadsheet. Notice that the final row uses a custom User Defined Function `FETCH_PRICE()` which is evaluated natively in Rust.

```spreadsheet
{
  "id": "sheet_portfolio",
  "data": [
    ["Ticker", "Shares", "Price", "Total Value"],
    ["AAPL", 150, 175.5, "=B2*C2"],
    ["MSFT", 50, 330.0, "=B3*C3"],
    ["Custom", 100, "=FETCH_PRICE(\"BTC\")", "=B4*C4"],
    ["PORTFOLIO TOTAL", "", "", "=SUM(D2:D4)"]
  ]
}
```
</div>

---

## 5. Embedded Terminal Applications

<div id="terminal-content">

You can embed external TUI applications directly inside the interface pane.

```terminal
{
  "id": "system_monitor",
  "command": "htop",
  "args": [],
  "cwd": "/",
  "env": { "TERM": "xterm-256color" }
}
```
</div>

---

## 6. Editable Text Blocks

If the host renderer supports it, specific markdown blocks can be made directly editable within the GUI.

```editable
This is a standard text block, but when you click or focus it in the UI, it becomes an input field. 
When you save, the TUI emits an `edited` event over the IPC bus containing this new text.
```
