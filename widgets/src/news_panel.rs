//! News panel — a scrollable, filterable headline list with a detail pane, colored by
//! source. Parity target: `this-is-not-bbg/tui/internal/views/news.go`.
//!
//! Dependency-free by design: the widget knows nothing about any feed type. The consumer
//! maps its own news model onto [`NewsRow`] (one per visible headline) and, when a row is
//! selected, passes the wrapped body to render in the detail pane. The bounded TTL cache
//! that produces these rows lives in the consumer (N2's `n2-client::news`), which is the
//! object the formal model covers — this file is rendering only.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Widget},
};

use crate::truncate;

/// One render-ready headline. `age` is a pre-formatted "2m"/"1h" string (the consumer
/// owns the clock); `tickers` is a pre-joined display string (e.g. "BTCUSDT BTC").
#[derive(Clone, Debug, Default)]
pub struct NewsRow {
    pub age: String,
    pub source: String,
    pub title: String,
    pub tickers: String,
}

/// Format a nanosecond age into a compact "now/5s/3m/2h/1d" label.
pub fn age_label(age_ns: i64) -> String {
    let s = age_ns.max(0) / 1_000_000_000;
    if s < 1 {
        "now".to_string()
    } else if s < 60 {
        format!("{s}s")
    } else if s < 3_600 {
        format!("{}m", s / 60)
    } else if s < 86_400 {
        format!("{}h", s / 3_600)
    } else {
        format!("{}d", s / 86_400)
    }
}

/// A stable color per source name (so a given outlet keeps its hue across frames).
fn source_color(source: &str) -> Color {
    const PALETTE: [Color; 6] = [
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Magenta,
        Color::Blue,
        Color::LightRed,
    ];
    let h = source
        .bytes()
        .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
    PALETTE[(h as usize) % PALETTE.len()]
}

pub struct NewsPanel<'a> {
    rows: &'a [NewsRow],
    selected: usize,
    /// Optional already-wrapped detail lines for the selected row (consumer wraps to width).
    detail: Option<&'a [String]>,
    block: Option<Block<'a>>,
}

impl<'a> NewsPanel<'a> {
    pub fn new(rows: &'a [NewsRow]) -> Self {
        Self {
            rows,
            selected: 0,
            detail: None,
            block: None,
        }
    }
    pub fn selected(mut self, i: usize) -> Self {
        self.selected = i;
        self
    }
    pub fn detail(mut self, lines: &'a [String]) -> Self {
        self.detail = Some(lines);
        self
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for NewsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let NewsPanel {
            rows,
            selected,
            detail,
            block,
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
        let w = inner.width as usize;

        // Split: detail pane (if any) takes the bottom third, list the rest.
        let (list_h, detail_h) = match detail {
            Some(d) if !d.is_empty() && inner.height >= 6 => {
                let dh = (inner.height / 3).min(d.len() as u16 + 1).max(2);
                (inner.height - dh, dh)
            }
            _ => (inner.height, 0),
        };

        // Scroll so the selection stays visible (simple windowing).
        let list_h_us = list_h as usize;
        let start = selected.saturating_sub(list_h_us.saturating_sub(1));
        for (row_idx, item) in rows
            .iter()
            .enumerate()
            .skip(start)
            .take(list_h_us)
            .enumerate()
        {
            let (abs_idx, news) = item;
            let y = inner.y + row_idx as u16;
            let is_sel = abs_idx == selected;
            // "  2m  Source            Title …            [BTCUSDT]"
            let age = format!("{:>4}", news.age);
            let src = truncate(&news.source, 12);
            let head = format!("{age}  {src:<12}  ");
            let tick = if news.tickers.is_empty() {
                String::new()
            } else {
                format!("  [{}]", news.tickers)
            };
            let title_w = w.saturating_sub(head.chars().count() + tick.chars().count());
            let title = truncate(&news.title, title_w);
            let line = format!("{head}{title}{tick}");
            let base = Style::default().fg(source_color(&news.source));
            let style = if is_sel {
                base.bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                base
            };
            // Pad to full width so the selection highlight spans the row.
            let padded = format!("{:<width$}", truncate(&line, w), width = w);
            buf.set_string(inner.x, y, padded, style);
        }

        // Detail pane.
        if detail_h > 0 {
            if let Some(lines) = detail {
                let dy0 = inner.y + list_h;
                // a thin separator row
                buf.set_string(
                    inner.x,
                    dy0,
                    "─".repeat(w),
                    Style::default().fg(Color::DarkGray),
                );
                for (i, l) in lines.iter().take(detail_h as usize - 1).enumerate() {
                    buf.set_string(
                        inner.x,
                        dy0 + 1 + i as u16,
                        truncate(l, w),
                        Style::default().fg(Color::Gray),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(age: &str, src: &str, title: &str) -> NewsRow {
        NewsRow {
            age: age.into(),
            source: src.into(),
            title: title.into(),
            tickers: "BTCUSDT".into(),
        }
    }

    #[test]
    fn age_label_buckets() {
        assert_eq!(age_label(0), "now");
        assert_eq!(age_label(5 * 1_000_000_000), "5s");
        assert_eq!(age_label(120 * 1_000_000_000), "2m");
        assert_eq!(age_label(7_200 * 1_000_000_000), "2h");
        assert_eq!(age_label(172_800 * 1_000_000_000), "2d");
    }

    #[test]
    fn source_color_is_stable() {
        assert_eq!(source_color("DeskDesk"), source_color("DeskDesk"));
    }

    #[test]
    fn renders_list_and_detail_without_panicking() {
        let rows = vec![
            row("2m", "DeskDesk", "headline one"),
            row("5m", "RiskRoom", "headline two"),
        ];
        let detail = vec!["wrapped body line 1".to_string(), "line 2".to_string()];
        let area = Rect::new(0, 0, 60, 12);
        let mut buf = Buffer::empty(area);
        NewsPanel::new(&rows)
            .selected(1)
            .detail(&detail)
            .block(Block::default().title("News"))
            .render(area, &mut buf);
    }

    #[test]
    fn empty_rows_is_safe() {
        let rows: Vec<NewsRow> = vec![];
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);
        NewsPanel::new(&rows).render(area, &mut buf);
    }
}
