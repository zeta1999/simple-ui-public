//! Depth-of-book ladder — the market-maker's primary view: asks above the mid
//! (red), bids below (green), each row a size bar scaled to the largest level.
//!
//! Renders whatever depth it's given. N1's seam serves only top-of-book today,
//! so full multi-level data is pending an N1 depth method (tracked in TODO);
//! this widget is ready to consume it.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Widget},
};

use crate::truncate;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthLevel {
    pub price: f64,
    pub size: f64,
}

/// The bot's own resting two-sided quote, overlaid on the book. A side with
/// `size <= 0` isn't drawn (risk shut it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LadderQuote {
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
}

/// Index of the level whose price is closest to `price` (None if empty).
pub fn nearest_index(levels: &[DepthLevel], price: f64) -> Option<usize> {
    levels
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (a.price - price)
                .abs()
                .partial_cmp(&(b.price - price).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// Order-book depth. `bids` descending (best first); `asks` ascending (best first).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DepthData {
    pub bids: Vec<DepthLevel>,
    pub asks: Vec<DepthLevel>,
}

impl DepthData {
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }
    pub fn mid(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(b), Some(a)) => Some((a + b) / 2.0),
            _ => None,
        }
    }
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(b), Some(a)) => Some(a - b),
            _ => None,
        }
    }
    /// Largest size across both sides, for bar scaling. 0.0 if empty.
    pub fn max_size(&self) -> f64 {
        self.bids
            .iter()
            .chain(self.asks.iter())
            .map(|l| l.size)
            .fold(0.0, f64::max)
    }
}

pub struct DepthLadder<'a> {
    data: &'a DepthData,
    block: Option<Block<'a>>,
    levels: usize,
    quote: Option<LadderQuote>,
}

impl<'a> DepthLadder<'a> {
    pub fn new(data: &'a DepthData) -> Self {
        Self {
            data,
            block: None,
            levels: 10,
            quote: None,
        }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    pub fn levels(mut self, n: usize) -> Self {
        self.levels = n.max(1);
        self
    }
    /// Overlay the bot's resting quote (mid-row readout + nearest-level markers).
    pub fn quote(mut self, q: LadderQuote) -> Self {
        self.quote = Some(q);
        self
    }
}

impl Widget for DepthLadder<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let DepthLadder {
            data,
            block,
            levels,
            quote,
        } = self;
        let inner = match block {
            Some(b) => {
                let i = b.inner(area);
                b.render(area, buf);
                i
            }
            None => area,
        };
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let max_size = data.max_size();
        let bar_room = (inner.width as usize).saturating_sub(20);
        // Rows available for each side (leave one for the mid row).
        let per_side = levels
            .min(((inner.height as usize).saturating_sub(1)) / 2)
            .max(1);

        // Which book level (if any) is nearest the bot's quote on each side.
        // Only a side that's actually resting (size > 0) is marked.
        let marked_ask = quote
            .filter(|q| q.ask_size > 0.0)
            .and_then(|q| nearest_index(&data.asks, q.ask_price));
        let marked_bid = quote
            .filter(|q| q.bid_size > 0.0)
            .and_then(|q| nearest_index(&data.bids, q.bid_price));

        let mut y = inner.y;
        // Asks worst→best so the best ask sits just above the mid row.
        for (idx, lvl) in data
            .asks
            .iter()
            .take(per_side)
            .enumerate()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            if y >= inner.bottom() {
                return;
            }
            draw_row(
                buf,
                inner.x,
                y,
                inner.width,
                lvl,
                max_size,
                bar_room,
                Color::Red,
                marked_ask == Some(idx),
            );
            y += 1;
        }
        if y < inner.bottom() {
            if let (Some(mid), Some(spread)) = (data.mid(), data.spread()) {
                let mut s = format!("── mid {mid:.2}  spr {spread:.4}");
                if let Some(q) = quote {
                    s.push_str(&format!("  mine {:.2}/{:.2}", q.bid_price, q.ask_price));
                }
                s.push_str(" ──");
                buf.set_string(
                    inner.x,
                    y,
                    truncate(&s, inner.width as usize),
                    Style::default().fg(Color::Yellow),
                );
            }
            y += 1;
        }
        for (idx, lvl) in data.bids.iter().take(per_side).enumerate() {
            if y >= inner.bottom() {
                return;
            }
            draw_row(
                buf,
                inner.x,
                y,
                inner.width,
                lvl,
                max_size,
                bar_room,
                Color::Green,
                marked_bid == Some(idx),
            );
            y += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    lvl: &DepthLevel,
    max_size: f64,
    bar_room: usize,
    color: Color,
    marked: bool,
) {
    let mut style = Style::default().fg(color);
    if marked {
        style = style.add_modifier(Modifier::BOLD);
    }
    buf.set_string(x, y, format!("{:>9.2}", lvl.price), style);
    let bar_len = if max_size > 0.0 {
        ((lvl.size / max_size) * bar_room as f64).round() as usize
    } else {
        0
    };
    buf.set_string(x + 10, y, "█".repeat(bar_len.min(bar_room)), style);
    let size = format!("{:.4}", lvl.size);
    let sx = x + width.saturating_sub(size.len() as u16);
    buf.set_string(sx, y, size, style);
    // A cyan marker in the gap between price and bar flags the bot's quote level.
    if marked {
        buf.set_string(
            x + 9,
            y,
            "◀",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DepthData {
        DepthData {
            bids: vec![
                DepthLevel {
                    price: 100.0,
                    size: 2.0,
                },
                DepthLevel {
                    price: 99.0,
                    size: 5.0,
                },
            ],
            asks: vec![
                DepthLevel {
                    price: 101.0,
                    size: 3.0,
                },
                DepthLevel {
                    price: 102.0,
                    size: 1.0,
                },
            ],
        }
    }

    #[test]
    fn computes_top_of_book_metrics() {
        let d = sample();
        assert_eq!(d.best_bid(), Some(100.0));
        assert_eq!(d.best_ask(), Some(101.0));
        assert_eq!(d.mid(), Some(100.5));
        assert_eq!(d.spread(), Some(1.0));
        assert_eq!(d.max_size(), 5.0);
    }

    #[test]
    fn empty_depth_has_no_metrics() {
        let d = DepthData::default();
        assert_eq!(d.mid(), None);
        assert_eq!(d.spread(), None);
        assert_eq!(d.max_size(), 0.0);
    }

    #[test]
    fn renders_into_buffer_without_panicking() {
        let area = Rect::new(0, 0, 40, 12);
        let mut buf = Buffer::empty(area);
        DepthLadder::new(&sample())
            .block(Block::default().title("DOB"))
            .levels(5)
            .render(area, &mut buf);
        // empty data path too
        let mut buf2 = Buffer::empty(area);
        DepthLadder::new(&DepthData::default()).render(area, &mut buf2);
    }

    #[test]
    fn nearest_index_picks_closest_level() {
        let d = sample();
        // closest ask to 101.4 is the best ask (101.0) at index 0
        assert_eq!(nearest_index(&d.asks, 101.4), Some(0));
        // closest ask to 101.9 is 102.0 at index 1
        assert_eq!(nearest_index(&d.asks, 101.9), Some(1));
        // closest bid to 98.6 is 99.0 at index 1
        assert_eq!(nearest_index(&d.bids, 98.6), Some(1));
        assert_eq!(nearest_index(&[], 100.0), None);
    }

    #[test]
    fn renders_with_quote_overlay_without_panicking() {
        let area = Rect::new(0, 0, 44, 12);
        let mut buf = Buffer::empty(area);
        // ask quoting, bid shut by risk (size 0 → not marked)
        let q = LadderQuote {
            bid_price: 100.0,
            bid_size: 0.0,
            ask_price: 101.2,
            ask_size: 0.01,
        };
        DepthLadder::new(&sample())
            .levels(5)
            .quote(q)
            .render(area, &mut buf);
    }
}
