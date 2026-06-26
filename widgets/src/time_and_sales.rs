//! Time & sales tape — streaming trade prints, newest first, colored by
//! aggressor side (green buy / red sell). Fully fed by N1's live trade stream
//! (`subscribe`).

use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

use crate::truncate;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TradePrint {
    pub ts_ns: i64,
    pub price: f64,
    pub size: f64,
    pub buy: bool,
}

/// Bounded ring of recent trade prints (newest at the back).
#[derive(Debug)]
pub struct TapeBuffer {
    cap: usize,
    items: VecDeque<TradePrint>,
}

impl TapeBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            items: VecDeque::new(),
        }
    }

    pub fn push(&mut self, t: TradePrint) {
        self.items.push_back(t);
        while self.items.len() > self.cap {
            self.items.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate newest-first.
    pub fn iter_newest(&self) -> impl Iterator<Item = &TradePrint> {
        self.items.iter().rev()
    }
}

pub struct TimeAndSales<'a> {
    tape: &'a TapeBuffer,
    block: Option<Block<'a>>,
}

impl<'a> TimeAndSales<'a> {
    pub fn new(tape: &'a TapeBuffer) -> Self {
        Self { tape, block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for TimeAndSales<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let TimeAndSales { tape, block } = self;
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
        for (row, t) in tape.iter_newest().take(inner.height as usize).enumerate() {
            let y = inner.y + row as u16;
            let color = if t.buy { Color::Green } else { Color::Red };
            let side = if t.buy { "B" } else { "S" };
            let line = format!("{side} {:>11.2}  {:>12.6}", t.price, t.size);
            buf.set_string(
                inner.x,
                y,
                truncate(&line, inner.width as usize),
                Style::default().fg(color),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print(ts: i64, buy: bool) -> TradePrint {
        TradePrint {
            ts_ns: ts,
            price: 100.0 + ts as f64,
            size: 1.0,
            buy,
        }
    }

    #[test]
    fn ring_caps_and_keeps_newest() {
        let mut tape = TapeBuffer::new(3);
        assert!(tape.is_empty());
        for ts in 0..5 {
            tape.push(print(ts, ts % 2 == 0));
        }
        assert_eq!(tape.len(), 3); // capped at 3
        let newest: Vec<i64> = tape.iter_newest().map(|t| t.ts_ns).collect();
        assert_eq!(newest, vec![4, 3, 2]); // newest first, oldest dropped
    }

    #[test]
    fn renders_into_buffer_without_panicking() {
        let mut tape = TapeBuffer::new(100);
        tape.push(print(1, true));
        tape.push(print(2, false));
        let area = Rect::new(0, 0, 32, 8);
        let mut buf = Buffer::empty(area);
        TimeAndSales::new(&tape)
            .block(Block::default().title("Time & Sales"))
            .render(area, &mut buf);
    }
}
