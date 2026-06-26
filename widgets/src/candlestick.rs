use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{
        canvas::{Canvas, Line, Rectangle},
        Block, Widget,
    },
};

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub struct CandlestickChart<'a> {
    block: Option<Block<'a>>,
    data: &'a [Candle],
}

impl<'a> CandlestickChart<'a> {
    pub fn new(data: &'a [Candle]) -> Self {
        Self { block: None, data }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for CandlestickChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (inner_area, block) = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                (inner, Some(b))
            }
            None => (area, None),
        };

        if let Some(b) = block {
            b.render(area, buf);
        }

        if self.data.is_empty() {
            return;
        }

        let min_y = self
            .data
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);
        let max_y = self
            .data
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);

        let max_x = self.data.len() as f64 * 4.0; // 4 units per candle

        Canvas::default()
            .x_bounds([0.0, max_x])
            .y_bounds([min_y, max_y])
            .paint(|ctx| {
                for (i, candle) in self.data.iter().enumerate() {
                    let color = if candle.close >= candle.open {
                        Color::Green
                    } else {
                        Color::Red
                    };

                    let x_center = i as f64 * 4.0 + 2.0;

                    // Wick
                    ctx.draw(&Line {
                        x1: x_center,
                        y1: candle.low,
                        x2: x_center,
                        y2: candle.high,
                        color,
                    });

                    // Body
                    let (bottom, top) = if candle.close >= candle.open {
                        (candle.open, candle.close)
                    } else {
                        (candle.close, candle.open)
                    };

                    let width = 2.0;
                    ctx.draw(&Rectangle {
                        x: x_center - width / 2.0,
                        y: bottom,
                        width,
                        height: (top - bottom).max(0.1), // Ensure body is visible
                        color,
                    });
                }
            })
            .render(inner_area, buf);
    }
}
