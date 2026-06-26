//! Candlestick chart with a viewport (zoom + pan) and a moving-average overlay
//! — the "usable chart" beyond the rudimentary candlestick (P1 gate criterion).
//!
//! Zoom = how many candles are visible; pan = how many candles back from the
//! newest the right edge sits. An optional SMA(n) line is drawn over the window.
//! Prototyped here; upstream to `simple-ui` once stable.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ohlc {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// A viewport over a candle series: show `visible` candles ending `offset`
/// candles back from the newest. `offset = 0` pins the right edge to the latest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartView {
    pub visible: usize,
    pub offset: usize,
}

impl Default for ChartView {
    fn default() -> Self {
        Self {
            visible: 60,
            offset: 0,
        }
    }
}

impl ChartView {
    /// The `[start, end)` index range into a series of length `len` that this
    /// view selects (both clamped so the window is always valid and non-empty
    /// when `len > 0`).
    pub fn range(&self, len: usize) -> (usize, usize) {
        if len == 0 {
            return (0, 0);
        }
        let visible = self.visible.clamp(1, len);
        let offset = self.offset.min(len - visible);
        let end = len - offset;
        let start = end - visible;
        (start, end)
    }
}

/// Simple moving average of closes ending at global index `g` (inclusive) over
/// `period` candles. None if there isn't enough history.
pub fn sma_at(candles: &[Ohlc], g: usize, period: usize) -> Option<f64> {
    if period == 0 || g >= candles.len() || g + 1 < period {
        return None;
    }
    let sum: f64 = candles[g + 1 - period..=g].iter().map(|c| c.close).sum();
    Some(sum / period as f64)
}

pub struct Chart<'a> {
    candles: &'a [Ohlc],
    view: ChartView,
    sma_period: Option<usize>,
    block: Option<Block<'a>>,
}

impl<'a> Chart<'a> {
    pub fn new(candles: &'a [Ohlc]) -> Self {
        Self {
            candles,
            view: ChartView::default(),
            sma_period: None,
            block: None,
        }
    }
    pub fn view(mut self, view: ChartView) -> Self {
        self.view = view;
        self
    }
    pub fn sma(mut self, period: usize) -> Self {
        self.sma_period = if period > 1 { Some(period) } else { None };
        self
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

/// Map a price in `[lo, hi]` to a row within `[top, bottom]` (higher price →
/// higher up = smaller y).
fn price_to_y(p: f64, lo: f64, hi: f64, top: u16, bottom: u16) -> u16 {
    if hi <= lo || bottom <= top {
        return bottom;
    }
    let frac = ((p - lo) / (hi - lo)).clamp(0.0, 1.0);
    let span = (bottom - top) as f64;
    bottom - (frac * span).round() as u16
}

impl Widget for Chart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match &self.block {
            Some(b) => {
                let i = b.inner(area);
                b.clone().render(area, buf);
                i
            }
            None => area,
        };
        if inner.width < 4 || inner.height < 2 || self.candles.is_empty() {
            return;
        }

        // A left gutter for price labels when there's room.
        let axis_w: u16 = if inner.width > 16 { 9 } else { 0 };
        let chart_x = inner.x + axis_w;
        let chart_w = inner.width - axis_w;
        let top = inner.y;
        let bottom = inner.bottom() - 1;

        let (start, end) = self.view.range(self.candles.len());
        let window = &self.candles[start..end];
        // If the window is wider than the chart, show its most recent columns.
        let n = window.len().min(chart_w as usize);
        let shown_start = start + (window.len() - n);
        let shown = &self.candles[shown_start..shown_start + n];

        // Price range over what's actually shown (include SMA values? keep to
        // candle extent — the SMA stays within it by construction).
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        for c in shown {
            lo = lo.min(c.low);
            hi = hi.max(c.high);
        }
        if !lo.is_finite() || !hi.is_finite() {
            return;
        }
        if (hi - lo).abs() < 1e-9 {
            hi = lo + 1.0; // flat series: avoid a zero range
        }

        // Price-axis labels in the gutter (hi top, lo bottom, mid middle).
        if axis_w > 0 {
            let mid = (hi + lo) / 2.0;
            let lab = |v: f64| format!("{v:>8.1}");
            buf.set_string(inner.x, top, lab(hi), Style::default().fg(Color::DarkGray));
            buf.set_string(
                inner.x,
                (top + bottom) / 2,
                lab(mid),
                Style::default().fg(Color::DarkGray),
            );
            buf.set_string(
                inner.x,
                bottom,
                lab(lo),
                Style::default().fg(Color::DarkGray),
            );
        }

        // Candles: one column each, oldest-left / newest-right.
        for (j, c) in shown.iter().enumerate() {
            let x = chart_x + j as u16;
            let up = c.close >= c.open;
            let color = if up { Color::Green } else { Color::Red };
            let style = Style::default().fg(color);

            // Wick: high → low.
            let y_hi = price_to_y(c.high, lo, hi, top, bottom);
            let y_lo = price_to_y(c.low, lo, hi, top, bottom);
            for y in y_hi..=y_lo {
                buf.set_string(x, y, "│", style);
            }
            // Body: open ↔ close (drawn over the wick).
            let y_open = price_to_y(c.open, lo, hi, top, bottom);
            let y_close = price_to_y(c.close, lo, hi, top, bottom);
            let (b_top, b_bot) = (y_open.min(y_close), y_open.max(y_close));
            if b_top == b_bot {
                buf.set_string(x, b_top, "─", style); // doji
            } else {
                for y in b_top..=b_bot {
                    buf.set_string(x, y, "█", style);
                }
            }
        }

        // SMA overlay (drawn last so it stays visible).
        if let Some(period) = self.sma_period {
            for (j, _) in shown.iter().enumerate() {
                let g = shown_start + j;
                if let Some(v) = sma_at(self.candles, g, period) {
                    if v >= lo && v <= hi {
                        let x = chart_x + j as u16;
                        let y = price_to_y(v, lo, hi, top, bottom);
                        buf.set_string(x, y, "•", Style::default().fg(Color::Cyan));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(n: usize) -> Vec<Ohlc> {
        (0..n)
            .map(|i| {
                let p = 100.0 + i as f64;
                Ohlc {
                    open: p,
                    high: p + 1.0,
                    low: p - 1.0,
                    close: p + 0.5,
                }
            })
            .collect()
    }

    #[test]
    fn view_range_empty_is_zero() {
        assert_eq!(ChartView::default().range(0), (0, 0));
    }

    #[test]
    fn view_range_default_pins_to_newest() {
        let v = ChartView {
            visible: 60,
            offset: 0,
        };
        assert_eq!(v.range(100), (40, 100));
    }

    #[test]
    fn view_range_pan_moves_window_back() {
        let v = ChartView {
            visible: 60,
            offset: 20,
        };
        assert_eq!(v.range(100), (20, 80));
    }

    #[test]
    fn view_range_clamps_zoom_and_pan() {
        // visible larger than len → whole series
        assert_eq!(
            ChartView {
                visible: 200,
                offset: 0
            }
            .range(100),
            (0, 100)
        );
        // offset past the start → clamped so the window stays valid
        assert_eq!(
            ChartView {
                visible: 60,
                offset: 500
            }
            .range(100),
            (0, 60)
        );
    }

    #[test]
    fn sma_needs_enough_history() {
        let s = series(10);
        assert_eq!(sma_at(&s, 0, 3), None); // not enough
        assert_eq!(sma_at(&s, 1, 3), None);
        // closes are p+0.5 = 100.5, 101.5, 102.5 → mean 101.5 at g=2
        assert_eq!(sma_at(&s, 2, 3), Some(101.5));
        assert_eq!(sma_at(&s, 5, 1), Some(105.5)); // period 1 = the close itself
    }

    #[test]
    fn renders_into_buffer_without_panicking() {
        let s = series(200);
        let area = Rect::new(0, 0, 80, 20);
        let mut buf = Buffer::empty(area);
        Chart::new(&s)
            .view(ChartView {
                visible: 60,
                offset: 10,
            })
            .sma(20)
            .block(Block::default().title("chart"))
            .render(area, &mut buf);
        // tiny area + empty series must not panic
        let mut b2 = Buffer::empty(Rect::new(0, 0, 3, 1));
        Chart::new(&s).render(Rect::new(0, 0, 3, 1), &mut b2);
        let mut b3 = Buffer::empty(area);
        Chart::new(&[]).render(area, &mut b3);
    }
}
