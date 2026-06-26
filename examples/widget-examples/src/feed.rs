//! A deterministic synthetic market — enough to drive every widget.
//!
//! There is no real exchange here and no randomness source: every value is
//! derived from a tick counter through [`noise`]. That means `step()` replays
//! identically run-to-run (so `--headless` snapshots never flake) while still
//! looking lively when animated in the interactive TUI.

use simple_ui_widgets::{
    chart::Ohlc,
    data_table::{Cell, Column},
    depth_ladder::{DepthData, DepthLevel, LadderQuote},
    time_and_sales::{TapeBuffer, TradePrint},
};

/// Deterministic pseudo-random value in `[-1.0, 1.0]` from a counter.
///
/// A SplitMix64-style scramble — not cryptographic, just a cheap way to turn
/// "tick 41" into a stable, well-spread number so the feed looks random but
/// replays exactly.
fn noise(n: u64) -> f64 {
    let mut x = n.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    // Top 53 bits → a double in [0, 1), then rescaled to [-1, 1).
    ((x >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
}

/// One line of the positions blotter.
pub struct Position {
    pub symbol: &'static str,
    pub qty: f64,
    pub avg_px: f64,
    pub last_px: f64,
}

impl Position {
    /// Mark-to-market profit/loss at the current price.
    pub fn pnl(&self) -> f64 {
        (self.last_px - self.avg_px) * self.qty
    }
}

/// How many ticks make up one candle on the chart.
const TICKS_PER_CANDLE: u64 = 8;

/// The whole synthetic market. Widgets read these fields; `step()` advances them.
pub struct Market {
    tick: u64,
    /// Current mid price of the charted/laddered instrument.
    pub mid: f64,
    pub depth: DepthData,
    pub tape: TapeBuffer,
    pub candles: Vec<Ohlc>,
    pub positions: Vec<Position>,
    /// The bot's own resting quote, overlaid on the ladder.
    pub quote: LadderQuote,
}

impl Market {
    /// A fresh market pre-seeded with some candle history so charts aren't empty
    /// on the first frame.
    pub fn new() -> Self {
        let mut m = Market {
            tick: 0,
            mid: 27_000.0,
            depth: DepthData::default(),
            tape: TapeBuffer::new(500),
            candles: Vec::new(),
            positions: starting_positions(),
            quote: LadderQuote {
                bid_price: 0.0,
                bid_size: 0.0,
                ask_price: 0.0,
                ask_size: 0.0,
            },
        };
        // Warm up ~200 candles of history.
        for _ in 0..(200 * TICKS_PER_CANDLE) {
            m.step();
        }
        m
    }

    /// Advance the market one tick: nudge the price, then rebuild the order book,
    /// append a trade print, fold the price into the current candle, and re-mark
    /// the positions.
    pub fn step(&mut self) {
        self.tick += 1;
        let t = self.tick;

        // Random-walk the mid by up to ~0.15% per tick.
        self.mid *= 1.0 + noise(t) * 0.0015;

        self.rebuild_depth();
        self.push_trade();
        self.fold_candle();
        self.mark_positions();
        self.refresh_quote();
    }

    /// A 10-level book straddling the mid. Sizes vary per level/tick so the
    /// ladder's size bars move.
    fn rebuild_depth(&mut self) {
        const LEVELS: usize = 10;
        let tick_size = self.mid * 0.0001; // 1bp grid
        let mut bids = Vec::with_capacity(LEVELS);
        let mut asks = Vec::with_capacity(LEVELS);
        for i in 0..LEVELS {
            let away = (i as f64 + 1.0) * tick_size;
            // size in [0.2, 5.2], distinct per side/level via the noise counter
            let bid_sz = 0.2 + (noise(self.tick * 31 + i as u64).abs()) * 5.0;
            let ask_sz = 0.2 + (noise(self.tick * 67 + i as u64).abs()) * 5.0;
            bids.push(DepthLevel {
                price: self.mid - away,
                size: bid_sz,
            });
            asks.push(DepthLevel {
                price: self.mid + away,
                size: ask_sz,
            });
        }
        self.depth = DepthData { bids, asks };
    }

    /// Append one trade print, side and size driven by the noise stream.
    fn push_trade(&mut self) {
        let n = noise(self.tick * 13);
        self.tape.push(TradePrint {
            ts_ns: self.tick as i64,
            price: self.mid + n * self.mid * 0.0001,
            size: 0.01 + n.abs() * 2.0,
            buy: n >= 0.0,
        });
    }

    /// Fold the current mid into the forming candle, rolling a new one every
    /// [`TICKS_PER_CANDLE`] ticks.
    fn fold_candle(&mut self) {
        let starting_new = self.tick % TICKS_PER_CANDLE == 1 || self.candles.is_empty();
        if starting_new {
            self.candles.push(Ohlc {
                open: self.mid,
                high: self.mid,
                low: self.mid,
                close: self.mid,
            });
        }
        let c = self.candles.last_mut().expect("just pushed if empty");
        c.high = c.high.max(self.mid);
        c.low = c.low.min(self.mid);
        c.close = self.mid;

        // Keep memory bounded for a long-running session.
        if self.candles.len() > 5_000 {
            self.candles.drain(0..1_000);
        }
    }

    /// Re-mark every position's last price off a per-symbol synthetic walk.
    fn mark_positions(&mut self) {
        for (i, p) in self.positions.iter_mut().enumerate() {
            // Each symbol gets its own drift via a per-symbol offset into noise.
            let drift = noise(self.tick * 7 + (i as u64 + 1) * 1000) * 0.002;
            p.last_px *= 1.0 + drift;
        }
    }

    /// Place the bot's quote one tick inside the top of book.
    fn refresh_quote(&mut self) {
        let tick_size = self.mid * 0.0001;
        self.quote = LadderQuote {
            bid_price: self.mid - tick_size * 0.5,
            bid_size: 0.5,
            ask_price: self.mid + tick_size * 0.5,
            ask_size: 0.5,
        };
    }
}

impl Default for Market {
    fn default() -> Self {
        Self::new()
    }
}

/// The blotter's starting book of positions.
fn starting_positions() -> Vec<Position> {
    vec![
        Position {
            symbol: "BTCUSDT",
            qty: 0.85,
            avg_px: 26_500.0,
            last_px: 27_000.0,
        },
        Position {
            symbol: "ETHUSDT",
            qty: -4.20,
            avg_px: 1_650.0,
            last_px: 1_625.0,
        },
        Position {
            symbol: "SOLUSDT",
            qty: 120.0,
            avg_px: 22.40,
            last_px: 24.10,
        },
        Position {
            symbol: "ADAUSDT",
            qty: -5_000.0,
            avg_px: 0.380,
            last_px: 0.372,
        },
        Position {
            symbol: "XRPUSDT",
            qty: 9_000.0,
            avg_px: 0.515,
            last_px: 0.508,
        },
        Position {
            symbol: "DOGEUSDT",
            qty: 80_000.0,
            avg_px: 0.0720,
            last_px: 0.0735,
        },
        Position {
            symbol: "AVAXUSDT",
            qty: -75.0,
            avg_px: 11.20,
            last_px: 10.95,
        },
        Position {
            symbol: "LINKUSDT",
            qty: 340.0,
            avg_px: 6.40,
            last_px: 6.72,
        },
    ]
}

// --- Blotter column/row helpers (shared by the data_table + terminal examples) ---

/// The positions-blotter column layout (also defines the sort columns).
pub fn blotter_columns() -> Vec<Column> {
    vec![
        Column::left("Symbol"),
        Column::right("Qty"),
        Column::right("Avg Px"),
        Column::right("Last"),
        Column::right("PnL"),
    ]
}

/// Render the live positions into table rows, coloring Qty and PnL by sign.
pub fn blotter_rows(positions: &[Position]) -> Vec<Vec<Cell>> {
    use ratatui::style::Color;
    let signed = |v: f64| if v >= 0.0 { Color::Green } else { Color::Red };
    positions
        .iter()
        .map(|p| {
            vec![
                Cell::new(p.symbol),
                Cell::colored(format!("{:.4}", p.qty), signed(p.qty)),
                Cell::new(format!("{:.2}", p.avg_px)),
                Cell::new(format!("{:.2}", p.last_px)),
                Cell::colored(format!("{:+.2}", p.pnl()), signed(p.pnl())),
            ]
        })
        .collect()
}

/// Index of the PnL column — the interesting one to sort by.
pub const BLOTTER_PNL_COL: usize = 4;
