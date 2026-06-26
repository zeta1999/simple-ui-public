//! Example: the DataTable as a scrollable, sortable, selectable blotter.
//!
//! Shows the three things that make it "blotter-grade": a scrolling viewport
//! over more rows than fit on screen, header-click-free sorting, and a moving
//! selection that's always kept in view. Only the visible window is ever drawn,
//! so the same code handles 10k rows.
//!
//!   cargo run -p widget-examples --bin data_table
//!   cargo run -p widget-examples --bin data_table -- --headless
//!
//! Keys:  ↑/↓ or j/k = move selection   s = cycle sort column   r = reverse

use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use simple_ui_widgets::data_table::{sort_rows, Cell, Column, DataTable, TableState};

use widget_examples::{harness, Example};

/// One watchlist instrument with enough state to re-mark each tick.
struct Instrument {
    symbol: String,
    last: f64,
    change_pct: f64,
}

struct Demo {
    instruments: Vec<Instrument>,
    state: TableState,
    /// Which column we're sorting by, and ascending vs descending.
    sort_col: usize,
    ascending: bool,
    tick: u64,
}

const COLS: [&str; 3] = ["Symbol", "Last", "Chg%"];

impl Demo {
    fn new() -> Self {
        // A 120-row watchlist — more than fits on screen, so scroll is real.
        let bases = ["BTC", "ETH", "SOL", "ADA", "XRP", "DOGE", "AVAX", "LINK"];
        let instruments = (0..120)
            .map(|i| Instrument {
                symbol: format!("{}{}USDT", bases[i % bases.len()], i / bases.len()),
                last: 10.0 + (i as f64 * 7.0) % 500.0,
                change_pct: 0.0,
            })
            .collect();
        Demo {
            instruments,
            state: TableState {
                offset: 0,
                selected: Some(0),
            },
            sort_col: 0,
            ascending: true,
            tick: 0,
        }
    }

    fn columns(&self) -> Vec<Column> {
        // Symbol left-aligned; the two numeric columns right-aligned.
        vec![
            Column::left(COLS[0]),
            Column::right(COLS[1]),
            Column::right(COLS[2]),
        ]
    }

    /// Build the table rows from current instrument state, then sort them by the
    /// active column. Sorting is the *caller's* job — the widget just shows the
    /// ▲/▼ indicator via `sort_indicator`.
    fn rows(&self) -> Vec<Vec<Cell>> {
        let mut rows: Vec<Vec<Cell>> = self
            .instruments
            .iter()
            .map(|i| {
                let chg_color = if i.change_pct >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                vec![
                    Cell::new(&i.symbol),
                    Cell::new(format!("{:.2}", i.last)),
                    Cell::colored(format!("{:+.2}", i.change_pct), chg_color),
                ]
            })
            .collect();
        sort_rows(&mut rows, self.sort_col, self.ascending);
        rows
    }
}

impl Example for Demo {
    fn title(&self) -> &str {
        "DataTable — Watchlist"
    }

    fn on_tick(&mut self) {
        // Drift each last price a touch so the Chg% column lives.
        self.tick += 1;
        for (i, inst) in self.instruments.iter_mut().enumerate() {
            let bump = ((self.tick.wrapping_mul(31) + i as u64) % 7) as f64 - 3.0;
            inst.last = (inst.last + bump * 0.05).max(0.01);
            inst.change_pct += bump * 0.01;
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        let len = self.instruments.len();
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.state.select_next(len),
            KeyCode::Up | KeyCode::Char('k') => self.state.select_prev(len),
            // Cycle the sort column across the three columns.
            KeyCode::Char('s') => self.sort_col = (self.sort_col + 1) % COLS.len(),
            KeyCode::Char('r') => self.ascending = !self.ascending,
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let columns = self.columns();
        let rows = self.rows();
        let table = DataTable::new(&columns, &rows)
            .sort_indicator(self.sort_col, self.ascending)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .block(
                Block::default()
                    .title(" Watchlist — ↑/↓ select · s sort · r reverse ")
                    .borders(Borders::ALL),
            );
        // The stateful render scrolls the selected row into view and writes the
        // new offset back into `self.state`, so scrolling persists across frames.
        frame.render_stateful_widget(table, frame.size(), &mut self.state);
    }

    fn hint(&self) -> &str {
        "↑/↓ select · s cycle sort column · r reverse · q quit"
    }
}

fn main() -> std::io::Result<()> {
    harness::run(Demo::new())
}
