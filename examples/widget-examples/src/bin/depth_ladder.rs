//! Example: the depth-of-book ladder.
//!
//! Asks above the mid (red), bids below (green), a size bar per level, and the
//! bot's own resting quote overlaid (the ◀ markers + the `mine` readout on the
//! mid row).
//!
//!   cargo run -p widget-examples --bin depth_ladder
//!   cargo run -p widget-examples --bin depth_ladder -- --headless

use ratatui::{
    widgets::{Block, Borders},
    Frame,
};
use simple_ui_widgets::depth_ladder::DepthLadder;
use widget_examples::{feed::Market, harness, Example};

struct Demo {
    market: Market,
}

impl Example for Demo {
    fn title(&self) -> &str {
        "Depth Ladder"
    }

    fn on_tick(&mut self) {
        self.market.step();
    }

    fn draw(&mut self, frame: &mut Frame) {
        let ladder = DepthLadder::new(&self.market.depth)
            .levels(10)
            .quote(self.market.quote)
            .block(
                Block::default()
                    .title(" BTCUSDT — Depth ")
                    .borders(Borders::ALL),
            );
        frame.render_widget(ladder, frame.size());
    }
}

fn main() -> std::io::Result<()> {
    harness::run(Demo {
        market: Market::new(),
    })
}
