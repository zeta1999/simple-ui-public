//! Example: the candlestick chart with a zoom/pan viewport and an SMA overlay.
//!
//! The chart never owns a scroll position — you hold a `ChartView` (how many
//! candles are visible, and how far back from the newest the right edge sits)
//! and hand it in each frame. Here the arrow keys drive that view.
//!
//!   cargo run -p widget-examples --bin chart
//!   cargo run -p widget-examples --bin chart -- --headless
//!
//! Keys:  +/- = zoom   ←/→ = pan   0 = jump to newest

use crossterm::event::KeyCode;
use ratatui::{
    widgets::{Block, Borders},
    Frame,
};
use simple_ui_widgets::chart::{Chart, ChartView};
use widget_examples::{feed::Market, harness, Example};

struct Demo {
    market: Market,
    view: ChartView,
}

impl Example for Demo {
    fn title(&self) -> &str {
        "Candlestick Chart"
    }

    fn on_tick(&mut self) {
        self.market.step();
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            // Zoom: fewer/more visible candles (clamped sensibly).
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.view.visible = self.view.visible.saturating_sub(5).max(10);
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.view.visible = (self.view.visible + 5).min(300);
            }
            // Pan: move the window back/forward in history.
            KeyCode::Left => self.view.offset += 5,
            KeyCode::Right => self.view.offset = self.view.offset.saturating_sub(5),
            KeyCode::Char('0') => self.view.offset = 0, // snap to the latest candle
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chart = Chart::new(&self.market.candles)
            .view(self.view)
            .sma(20) // a 20-period moving-average overlay (the cyan dots)
            .block(
                Block::default()
                    .title(" BTCUSDT — Chart  (+/- zoom · ←/→ pan · 0 newest) ")
                    .borders(Borders::ALL),
            );
        frame.render_widget(chart, frame.size());
    }

    fn hint(&self) -> &str {
        "+/- zoom · ←/→ pan · 0 newest · q quit"
    }
}

fn main() -> std::io::Result<()> {
    harness::run(Demo {
        market: Market::new(),
        view: ChartView {
            visible: 80,
            offset: 0,
        },
    })
}
