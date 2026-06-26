//! `simple-ui-widgets` — reusable ratatui widgets for simple-ui consumers.
//!
//! Split out of the `tui` demo app per `../CHANGE_REQUESTS.md` #1 so consumers
//! (the N2 trading terminal, others) can depend on just the widgets + ratatui,
//! not the demo's tokio/syntect/portable-pty/tui-textarea tree. The `tui` demo
//! app should eventually depend on this crate instead of carrying its own copy.
//!
//! Trading-terminal widgets (`depth_ladder`, `time_and_sales`, `data_table`,
//! `chart`) were prototyped in N2's `n2-widgets` and upstreamed here per
//! CHANGE_REQUESTS #2/#3/#4/#6.

pub mod agent_log;
pub mod candlestick;
pub mod chart;
pub mod data_table;
pub mod depth_ladder;
pub mod news_panel;
pub mod time_and_sales;

/// Truncate a string to at most `w` columns (char-wise; ASCII-centric, fine for
/// the numeric rows these widgets render).
pub(crate) fn truncate(s: &str, w: usize) -> String {
    if s.chars().count() <= w {
        s.to_string()
    } else {
        s.chars().take(w).collect()
    }
}
