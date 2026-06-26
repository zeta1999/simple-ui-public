//! Example: a whole trading terminal from the four widgets in one screen.
//!
//!   ┌──────────────────────────┬───────────────┐
//!   │                          │  Depth Ladder │
//!   │      Candlestick Chart   ├───────────────┤
//!   │                          │  Time & Sales │
//!   ├──────────────────────────┴───────────────┤
//!   │            Positions Blotter              │
//!   └───────────────────────────────────────────┘
//!
//!   cargo run -p widget-examples --bin trading_terminal
//!   cargo run -p widget-examples --bin trading_terminal -- --headless
//!
//! Keys:  ↑/↓ select position · p sort by PnL · +/- zoom chart · ←/→ pan chart

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use simple_ui_widgets::{
    chart::{Chart, ChartView},
    data_table::{sort_rows, DataTable, TableState},
    depth_ladder::DepthLadder,
    time_and_sales::TimeAndSales,
};

use widget_examples::{
    feed::{blotter_columns, blotter_rows, Market, BLOTTER_PNL_COL},
    harness, Example,
};

struct Terminal {
    market: Market,
    chart: ChartView,
    blotter: TableState,
    /// `None` = natural order; `Some(asc)` = sorted by PnL ascending/descending.
    pnl_sort: Option<bool>,
}

impl Example for Terminal {
    fn title(&self) -> &str {
        "Trading Terminal"
    }

    fn on_tick(&mut self) {
        self.market.step();
    }

    fn on_key(&mut self, key: KeyCode) {
        let len = self.market.positions.len();
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.blotter.select_next(len),
            KeyCode::Up | KeyCode::Char('k') => self.blotter.select_prev(len),
            // Toggle PnL sort: off → desc → asc → off.
            KeyCode::Char('p') => {
                self.pnl_sort = match self.pnl_sort {
                    None => Some(false),
                    Some(false) => Some(true),
                    Some(true) => None,
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.chart.visible = self.chart.visible.saturating_sub(5).max(10)
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.chart.visible = (self.chart.visible + 5).min(300)
            }
            KeyCode::Left => self.chart.offset += 5,
            KeyCode::Right => self.chart.offset = self.chart.offset.saturating_sub(5),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        // 1. Carve the screen into panels (see the ASCII sketch at the top).
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(frame.size());
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(rows[0]);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(top[1]);
        let (chart_area, ladder_area, tape_area, blotter_area) =
            (top[0], right[0], right[1], rows[1]);

        // 2. Chart (top-left).
        frame.render_widget(
            Chart::new(&self.market.candles)
                .view(self.chart)
                .sma(20)
                .block(bordered(" BTCUSDT — Chart ")),
            chart_area,
        );

        // 3. Depth ladder + bot quote (top-right, upper).
        frame.render_widget(
            DepthLadder::new(&self.market.depth)
                .levels(8)
                .quote(self.market.quote)
                .block(bordered(" Depth ")),
            ladder_area,
        );

        // 4. Time & sales tape (top-right, lower).
        frame.render_widget(
            TimeAndSales::new(&self.market.tape).block(bordered(" Time & Sales ")),
            tape_area,
        );

        // 5. Positions blotter (full width, bottom) — sorted on demand.
        let columns = blotter_columns();
        let mut rows = blotter_rows(&self.market.positions);
        if let Some(ascending) = self.pnl_sort {
            sort_rows(&mut rows, BLOTTER_PNL_COL, ascending);
        }
        let blotter = DataTable::new(&columns, &rows)
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .block(bordered(" Positions — ↑/↓ select · p sort PnL "));
        if let Some(ascending) = self.pnl_sort {
            // Keep the ▲/▼ indicator in sync with the active sort.
            frame.render_stateful_widget(
                blotter.sort_indicator(BLOTTER_PNL_COL, ascending),
                blotter_area,
                &mut self.blotter,
            );
        } else {
            frame.render_stateful_widget(blotter, blotter_area, &mut self.blotter);
        }
    }

    fn hint(&self) -> &str {
        "↑/↓ select · p sort PnL · +/- zoom · ←/→ pan · q quit"
    }
}

/// A bordered block with a title — every panel uses one, so name it once.
fn bordered(title: &str) -> Block<'_> {
    Block::default().title(title).borders(Borders::ALL)
}

fn main() -> std::io::Result<()> {
    harness::run(Terminal {
        market: Market::new(),
        chart: ChartView {
            visible: 70,
            offset: 0,
        },
        blotter: TableState {
            offset: 0,
            selected: Some(0),
        },
        pnl_sort: None,
    })
}
