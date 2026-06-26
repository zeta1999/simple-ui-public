//! DataTable — a column/row grid for blotters: positions, PnL, watchlists, and
//! (later) a greeks grid. Columns carry a title + alignment; cells carry text +
//! an optional color. Widths are computed from content and shrunk to fit the
//! area (`column_widths`).
//!
//! Two ways to render:
//! - [`Widget`] — stateless, draws from the top (no scroll/selection).
//! - [`StatefulWidget`] with [`TableState`] — a scrolling viewport + a selected
//!   row, so it backs large tables (10k+ rows) by drawing only the visible
//!   window. Sorting is a caller concern via [`sort_rows`]; pass
//!   [`DataTable::sort_indicator`] so the header shows ▲/▼.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::truncate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub title: String,
    pub align: Align,
}

impl Column {
    pub fn left(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            align: Align::Left,
        }
    }
    pub fn right(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            align: Align::Right,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cell {
    pub text: String,
    pub color: Option<Color>,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
        }
    }
    pub fn colored(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color: Some(color),
        }
    }
}

/// Scroll + selection state for the [`StatefulWidget`] render. `offset` is the
/// first visible row; `selected` is the highlighted row (kept in view on render).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableState {
    pub offset: usize,
    pub selected: Option<usize>,
}

impl TableState {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
    pub fn select(&mut self, i: Option<usize>) {
        self.selected = i;
    }
    /// Move selection down one (selects the first row if none), clamped to the end.
    pub fn select_next(&mut self, len: usize) {
        self.selected = if len == 0 {
            None
        } else {
            Some(match self.selected {
                Some(i) => (i + 1).min(len - 1),
                None => 0,
            })
        };
    }
    /// Move selection up one (selects the first row if none), clamped at the top.
    pub fn select_prev(&mut self, len: usize) {
        self.selected = if len == 0 {
            None
        } else {
            Some(match self.selected {
                Some(i) => i.saturating_sub(1),
                None => 0,
            })
        };
    }
}

/// New `offset` so the `selected` row stays within a `visible`-row window of a
/// `len`-row table. Pure (no rendering) so it's exhaustively testable — this is
/// what makes a 10k-row table cheap: only the window is ever drawn.
pub fn scroll_offset(offset: usize, selected: Option<usize>, visible: usize, len: usize) -> usize {
    if visible == 0 || len == 0 {
        return 0;
    }
    let max_off = len.saturating_sub(visible);
    let mut off = offset.min(max_off);
    if let Some(sel) = selected {
        let sel = sel.min(len - 1);
        if sel < off {
            off = sel;
        } else if sel >= off + visible {
            off = sel + 1 - visible;
        }
    }
    off.min(max_off)
}

fn parse_num(s: &str) -> Option<f64> {
    s.trim()
        .trim_end_matches('%')
        .replace(',', "")
        .parse::<f64>()
        .ok()
}

/// Sort `rows` in place by column `col` — numerically when both cells parse as
/// numbers (tolerating a trailing `%` and thousands `,`), else lexicographically.
pub fn sort_rows(rows: &mut [Vec<Cell>], col: usize, ascending: bool) {
    use std::cmp::Ordering;
    rows.sort_by(|a, b| {
        let sa = a.get(col).map(|c| c.text.as_str()).unwrap_or("");
        let sb = b.get(col).map(|c| c.text.as_str()).unwrap_or("");
        let ord = match (parse_num(sa), parse_num(sb)) {
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            _ => sa.cmp(sb),
        };
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}

/// One space between columns.
const COL_GAP: usize = 1;
/// Don't shrink a column below this (keeps headers/values legible).
const MIN_COL: usize = 3;

pub struct DataTable<'a> {
    columns: &'a [Column],
    rows: &'a [Vec<Cell>],
    block: Option<Block<'a>>,
    highlight_style: Style,
    sort: Option<(usize, bool)>,
}

impl<'a> DataTable<'a> {
    pub fn new(columns: &'a [Column], rows: &'a [Vec<Cell>]) -> Self {
        Self {
            columns,
            rows,
            block: None,
            highlight_style: Style::default().add_modifier(Modifier::REVERSED),
            sort: None,
        }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    /// Style applied to the selected row (stateful render). Default: reversed.
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }
    /// Show a ▲/▼ indicator on column `col` (the caller sorts via [`sort_rows`]).
    pub fn sort_indicator(mut self, col: usize, ascending: bool) -> Self {
        self.sort = Some((col, ascending));
        self
    }

    /// Per-column widths that fit within `max_total` (including 1-space gaps).
    /// Natural width = max(title, widest cell); if the row of naturals overflows,
    /// shrink the widest columns first (never below `MIN_COL`).
    pub fn column_widths(&self, max_total: u16) -> Vec<usize> {
        let n = self.columns.len();
        if n == 0 {
            return Vec::new();
        }
        let mut w: Vec<usize> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let cell_max = self
                    .rows
                    .iter()
                    .filter_map(|r| r.get(i))
                    .map(|c| c.text.chars().count())
                    .max()
                    .unwrap_or(0);
                cell_max.max(c.title.chars().count())
            })
            .collect();

        let gaps = COL_GAP * n.saturating_sub(1);
        let budget = max_total as usize;
        // Shrink the current-widest column until we fit (or can't shrink more).
        loop {
            let total: usize = w.iter().sum::<usize>() + gaps;
            if total <= budget {
                break;
            }
            // index of the widest shrinkable column
            let widest = w
                .iter()
                .enumerate()
                .filter(|(_, &x)| x > MIN_COL)
                .max_by_key(|(_, &x)| x)
                .map(|(i, _)| i);
            match widest {
                Some(i) => w[i] -= 1,
                None => break, // everything at the floor; render will clip
            }
        }
        w
    }

    fn render_block(&self, area: Rect, buf: &mut Buffer) -> Rect {
        match &self.block {
            Some(b) => {
                let i = b.inner(area);
                b.clone().render(area, buf);
                i
            }
            None => area,
        }
    }

    /// Draw header + the `[offset, offset+visible)` window of rows into `inner`.
    fn draw(&self, inner: Rect, buf: &mut Buffer, offset: usize, selected: Option<usize>) {
        if inner.width == 0 || inner.height == 0 || self.columns.is_empty() {
            return;
        }
        let widths = self.column_widths(inner.width);

        let put_row = |buf: &mut Buffer, y: u16, cells: &[(String, Style)]| {
            let mut x = inner.x;
            for (i, (s, style)) in cells.iter().enumerate() {
                if x >= inner.right() {
                    break;
                }
                buf.set_string(x, y, s, *style);
                x += widths[i] as u16 + COL_GAP as u16;
            }
        };

        // Header (bold), with a sort indicator on the sorted column.
        let header_style = Style::default().add_modifier(Modifier::BOLD);
        let header: Vec<(String, Style)> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let title = match self.sort {
                    Some((col, asc)) if col == i => {
                        format!("{}{}", c.title, if asc { "▲" } else { "▼" })
                    }
                    _ => c.title.clone(),
                };
                (fit(&title, widths[i], c.align), header_style)
            })
            .collect();
        put_row(buf, inner.y, &header);

        // Data rows: only the visible window (header takes row 0).
        let visible = (inner.height as usize).saturating_sub(1);
        let end = (offset + visible).min(self.rows.len());
        for (vis_i, row_idx) in (offset..end).enumerate() {
            let row = &self.rows[row_idx];
            let y = inner.y + 1 + vis_i as u16;
            let is_sel = selected == Some(row_idx);
            let cells: Vec<(String, Style)> = self
                .columns
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let cell = row.get(i).cloned().unwrap_or_default();
                    let mut style = match cell.color {
                        Some(col) => Style::default().fg(col),
                        None => Style::default(),
                    };
                    if is_sel {
                        style = style.patch(self.highlight_style);
                    }
                    (fit(&cell.text, widths[i], c.align), style)
                })
                .collect();
            put_row(buf, y, &cells);
        }
    }
}

fn fit(text: &str, width: usize, align: Align) -> String {
    let len = text.chars().count();
    if len > width {
        return truncate(text, width);
    }
    let pad = width - len;
    match align {
        Align::Left => format!("{text}{}", " ".repeat(pad)),
        Align::Right => format!("{}{text}", " ".repeat(pad)),
    }
}

impl Widget for DataTable<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = self.render_block(area, buf);
        self.draw(inner, buf, 0, None);
    }
}

impl StatefulWidget for DataTable<'_> {
    type State = TableState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
        let inner = self.render_block(area, buf);
        if inner.width == 0 || inner.height == 0 || self.columns.is_empty() {
            return;
        }
        // Clamp the selection to the data, then scroll it into view.
        state.selected = match state.selected {
            Some(_) if self.rows.is_empty() => None,
            Some(s) => Some(s.min(self.rows.len() - 1)),
            None => None,
        };
        let visible = (inner.height as usize).saturating_sub(1);
        state.offset = scroll_offset(state.offset, state.selected, visible, self.rows.len());
        self.draw(inner, buf, state.offset, state.selected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols() -> Vec<Column> {
        vec![
            Column::left("Symbol"),
            Column::right("Position"),
            Column::right("Notional"),
        ]
    }

    fn rows() -> Vec<Vec<Cell>> {
        vec![
            vec![
                Cell::new("BTCUSDT"),
                Cell::colored("-0.0057", Color::Red),
                Cell::new("418.20"),
            ],
            vec![
                Cell::new("ETHUSDT"),
                Cell::colored("0.1200", Color::Green),
                Cell::new("372.00"),
            ],
        ]
    }

    #[test]
    fn widths_take_max_of_title_and_cells_when_room() {
        let cols = cols();
        let rows = rows();
        let t = DataTable::new(&cols, &rows);
        // Plenty of room → natural widths.
        let w = t.column_widths(100);
        assert_eq!(w[0], "BTCUSDT".len()); // wider than "Symbol"
        assert_eq!(w[1], "Position".len()); // header wider than cells
        assert_eq!(w[2], "Notional".len());
    }

    #[test]
    fn widths_shrink_to_fit_a_narrow_area() {
        let cols = cols();
        let rows = rows();
        let t = DataTable::new(&cols, &rows);
        let budget = 16u16;
        let w = t.column_widths(budget);
        let total: usize = w.iter().sum::<usize>() + COL_GAP * (w.len() - 1);
        assert!(total <= budget as usize, "got {total} > {budget}");
        assert!(w.iter().all(|&x| x >= MIN_COL));
    }

    #[test]
    fn fit_aligns_and_truncates() {
        assert_eq!(fit("abc", 5, Align::Left), "abc  ");
        assert_eq!(fit("abc", 5, Align::Right), "  abc");
        assert_eq!(fit("abcdef", 4, Align::Right), "abcd"); // truncated
    }

    #[test]
    fn renders_into_buffer_without_panicking() {
        let cols = cols();
        let rows = rows();
        let area = Rect::new(0, 0, 30, 6);
        let mut buf = Buffer::empty(area);
        Widget::render(
            DataTable::new(&cols, &rows).block(Block::default().title("Positions")),
            area,
            &mut buf,
        );
        // empty rows + cramped area must not panic
        let mut buf2 = Buffer::empty(Rect::new(0, 0, 8, 2));
        Widget::render(DataTable::new(&cols, &[]), Rect::new(0, 0, 8, 2), &mut buf2);
    }

    #[test]
    fn scroll_offset_keeps_selection_in_view() {
        // 100 rows, 10 visible.
        assert_eq!(scroll_offset(0, Some(0), 10, 100), 0);
        assert_eq!(scroll_offset(0, Some(9), 10, 100), 0);
        assert_eq!(scroll_offset(0, Some(10), 10, 100), 1); // scrolled down one
        assert_eq!(scroll_offset(50, Some(5), 10, 100), 5); // scrolled back up
        assert_eq!(scroll_offset(0, Some(99), 10, 100), 90); // last row
        assert_eq!(scroll_offset(9_999, None, 10, 100), 90); // offset clamped to end
        assert_eq!(scroll_offset(0, Some(0), 0, 100), 0); // no room
        assert_eq!(scroll_offset(0, Some(0), 10, 0), 0); // empty
    }

    #[test]
    fn select_next_prev_clamp_and_handle_empty() {
        let mut s = TableState::default();
        s.select_next(3); // None → 0
        assert_eq!(s.selected, Some(0));
        s.select_next(3);
        s.select_next(3);
        s.select_next(3); // clamps at 2
        assert_eq!(s.selected, Some(2));
        s.select_prev(3);
        assert_eq!(s.selected, Some(1));
        s.select_next(0); // empty → None
        assert_eq!(s.selected, None);
    }

    #[test]
    fn sort_rows_is_numeric_then_lexicographic() {
        let mut r = vec![
            vec![Cell::new("b"), Cell::new("10")],
            vec![Cell::new("a"), Cell::new("2")],
            vec![Cell::new("c"), Cell::new("100")],
        ];
        // numeric ascending on col 1: 2, 10, 100
        sort_rows(&mut r, 1, true);
        assert_eq!(
            r.iter().map(|x| x[1].text.clone()).collect::<Vec<_>>(),
            vec!["2", "10", "100"]
        );
        // descending on col 1
        sort_rows(&mut r, 1, false);
        assert_eq!(r[0][1].text, "100");
        // lexicographic on col 0 (non-numeric)
        sort_rows(&mut r, 0, true);
        assert_eq!(
            r.iter().map(|x| x[0].text.clone()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn renders_10k_rows_drawing_only_the_viewport() {
        let cols = vec![Column::left("id"), Column::right("n")];
        let rows: Vec<Vec<Cell>> = (0..10_000)
            .map(|i| vec![Cell::new(format!("r{i}")), Cell::new(format!("{i}"))])
            .collect();
        let area = Rect::new(0, 0, 20, 12); // 11 data rows visible
        let mut buf = Buffer::empty(area);
        let mut state = TableState {
            offset: 0,
            selected: Some(9_999),
        };
        StatefulWidget::render(DataTable::new(&cols, &rows), area, &mut buf, &mut state);
        // Selecting the last row scrolled the window to the end.
        assert_eq!(state.offset, 9_989);

        let row_text =
            |y: u16| -> String { (0..area.width).map(|x| buf.get(x, y).symbol()).collect() };
        // The last visible data row shows r9999; r0 is nowhere on screen.
        assert!(row_text(area.bottom() - 1).contains("r9999"));
        for y in 0..area.height {
            assert!(!row_text(y).contains("r0 ") && !row_text(y).starts_with("r0"));
        }
    }
}
