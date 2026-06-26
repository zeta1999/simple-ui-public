//! Example: the time & sales tape.
//!
//! A streaming list of trade prints, newest at the top, green for buyer-aggressed
//! and red for seller-aggressed. The feed pushes into a bounded ring
//! (`TapeBuffer`) so it never grows without limit.
//!
//!   cargo run -p widget-examples --bin time_and_sales
//!   cargo run -p widget-examples --bin time_and_sales -- --headless

use ratatui::{
    widgets::{Block, Borders},
    Frame,
};
use simple_ui_widgets::time_and_sales::TimeAndSales;
use widget_examples::{feed::Market, harness, Example};

struct Demo {
    market: Market,
}

impl Example for Demo {
    fn title(&self) -> &str {
        "Time & Sales"
    }

    fn on_tick(&mut self) {
        self.market.step();
    }

    fn draw(&mut self, frame: &mut Frame) {
        let tape = TimeAndSales::new(&self.market.tape).block(
            Block::default()
                .title(" BTCUSDT — Time & Sales (B/S  price  size) ")
                .borders(Borders::ALL),
        );
        frame.render_widget(tape, frame.size());
    }
}

fn main() -> std::io::Result<()> {
    harness::run(Demo {
        market: Market::new(),
    })
}
