//! Shared plumbing for the `simple-ui-widgets` examples.
//!
//! Two pieces, kept deliberately small so the example binaries in `src/bin/`
//! read top-to-bottom with no surprises:
//!
//! - [`feed`] — a deterministic synthetic market (prices, depth, trades,
//!   candles, positions). Same input → same output, so the headless snapshots
//!   are stable and diffable.
//! - [`harness`] — runs an [`Example`] either interactively (a live ratatui TUI)
//!   or headless (`--headless`: advance the feed N frames, render one frame to an
//!   off-screen buffer, print it as text, exit — no terminal required).

pub mod feed;
pub mod harness;

pub use harness::Example;
