//! Agent activity log — a streaming view of an AI agent's events (thoughts, tool
//! calls, results, final answer), newest at the bottom. Mirrors the
//! `time_and_sales` tape pattern (bounded ring + stateless renderer) so consumers
//! like the `le-harnais` harness can render a live agent loop with only ratatui.
//!
//! Added per a consumer request; purely additive (no change to existing widgets).

use std::collections::VecDeque;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

use crate::truncate;

/// Kind of agent event, used for color/glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    Thought,
    Tool,
    Ok,
    Err,
    Final,
}

impl EventKind {
    fn glyph(self) -> &'static str {
        match self {
            EventKind::Thought => "·",
            EventKind::Tool => "→",
            EventKind::Ok => "✓",
            EventKind::Err => "✗",
            EventKind::Final => "★",
        }
    }
    fn color(self) -> Color {
        match self {
            EventKind::Thought => Color::DarkGray,
            EventKind::Tool => Color::Cyan,
            EventKind::Ok => Color::Green,
            EventKind::Err => Color::Red,
            EventKind::Final => Color::Yellow,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentEvent {
    pub kind: EventKind,
    pub text: String,
}

impl AgentEvent {
    pub fn new(kind: EventKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

/// Bounded ring of agent events (newest at the back).
#[derive(Debug, Default)]
pub struct AgentLogBuffer {
    cap: usize,
    items: VecDeque<AgentEvent>,
}

impl AgentLogBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            items: VecDeque::new(),
        }
    }
    pub fn push(&mut self, ev: AgentEvent) {
        self.items.push_back(ev);
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
    /// Iterate oldest-first (render order, newest at the bottom).
    pub fn iter(&self) -> impl Iterator<Item = &AgentEvent> {
        self.items.iter()
    }
}

/// Stateless renderer for an [`AgentLogBuffer`]. Shows the most recent events
/// that fit, one line each (glyph + colored text), newest at the bottom.
pub struct AgentLog<'a> {
    log: &'a AgentLogBuffer,
    block: Option<Block<'a>>,
}

impl<'a> AgentLog<'a> {
    pub fn new(log: &'a AgentLogBuffer) -> Self {
        Self { log, block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for AgentLog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let AgentLog { log, block } = self;
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
        let rows = inner.height as usize;
        // Take the last `rows` events so the newest are visible at the bottom.
        let total = log.len();
        let skip = total.saturating_sub(rows);
        for (row, ev) in log.iter().skip(skip).enumerate() {
            let y = inner.y + row as u16;
            let line = format!("{} {}", ev.kind.glyph(), ev.text.replace('\n', " "));
            let line = truncate(&line, inner.width as usize);
            buf.set_string(inner.x, y, &line, Style::default().fg(ev.kind.color()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn ring_caps_and_orders() {
        let mut b = AgentLogBuffer::new(2);
        b.push(AgentEvent::new(EventKind::Thought, "a"));
        b.push(AgentEvent::new(EventKind::Tool, "b"));
        b.push(AgentEvent::new(EventKind::Ok, "c"));
        assert_eq!(b.len(), 2);
        let texts: Vec<_> = b.iter().map(|e| e.text.clone()).collect();
        assert_eq!(texts, vec!["b", "c"]); // oldest dropped, order preserved
    }

    #[test]
    fn renders_newest_at_bottom() {
        let mut b = AgentLogBuffer::new(10);
        for i in 0..5 {
            b.push(AgentEvent::new(EventKind::Thought, format!("e{i}")));
        }
        let mut term = Terminal::new(TestBackend::new(20, 3)).unwrap();
        term.draw(|f| f.render_widget(AgentLog::new(&b), f.size()))
            .unwrap();
        let buf = term.backend().buffer().clone();
        let bottom: String = (0..20)
            .map(|x| buf.get(x, 2).symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            bottom.contains("e4"),
            "newest event should be on the bottom row: {bottom:?}"
        );
    }
}
