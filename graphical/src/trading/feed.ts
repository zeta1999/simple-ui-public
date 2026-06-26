// A deterministic synthetic market for the desktop trading widgets.
//
// This is a direct port of the Rust example feed (../../examples/widget-examples/
// src/feed.rs): same noise function, same parameters, so the desktop terminal and
// the TUI terminal show the *same* market. No randomness source — every value is
// derived from a tick counter, so it replays identically.

const MASK64 = (1n << 64n) - 1n;

// Deterministic pseudo-random value in [-1, 1] from a counter (SplitMix64-style).
// BigInt keeps the 64-bit wrapping arithmetic identical to the Rust version.
function noise(n: number): number {
  let x = (BigInt(n) * 0x9e3779b97f4a7c15n) & MASK64;
  x ^= x >> 30n;
  x = (x * 0xbf58476d1ce4e5b9n) & MASK64;
  x ^= x >> 27n;
  x = (x * 0x94d049bb133111ebn) & MASK64;
  x ^= x >> 31n;
  const top53 = Number(x >> 11n); // top 53 bits → an exact double
  return (top53 / 2 ** 53) * 2 - 1;
}

export interface DepthLevel {
  price: number;
  size: number;
}

export interface Depth {
  bids: DepthLevel[]; // best first (descending price)
  asks: DepthLevel[]; // best first (ascending price)
}

export interface Trade {
  price: number;
  size: number;
  buy: boolean;
}

export interface Ohlc {
  open: number;
  high: number;
  low: number;
  close: number;
}

export interface Quote {
  bidPrice: number;
  askPrice: number;
}

export interface Position {
  symbol: string;
  qty: number;
  avgPx: number;
  lastPx: number;
}

export function pnl(p: Position): number {
  return (p.lastPx - p.avgPx) * p.qty;
}

const TICKS_PER_CANDLE = 8;

// The whole synthetic market. Read the public fields; call step() to advance.
export class Market {
  tick = 0;
  mid = 27_000;
  depth: Depth = { bids: [], asks: [] };
  trades: Trade[] = []; // newest first, capped
  candles: Ohlc[] = [];
  positions: Position[] = startingPositions();
  quote: Quote = { bidPrice: 0, askPrice: 0 };

  constructor() {
    // Warm up ~200 candles of history so the chart isn't empty.
    for (let i = 0; i < 200 * TICKS_PER_CANDLE; i++) this.step();
  }

  step() {
    this.tick += 1;
    const t = this.tick;

    // Random-walk the mid by up to ~0.15% per tick.
    this.mid *= 1 + noise(t) * 0.0015;

    this.rebuildDepth();
    this.pushTrade();
    this.foldCandle();
    this.markPositions();
    this.refreshQuote();
  }

  private rebuildDepth() {
    const LEVELS = 10;
    const tickSize = this.mid * 0.0001; // 1bp grid
    const bids: DepthLevel[] = [];
    const asks: DepthLevel[] = [];
    for (let i = 0; i < LEVELS; i++) {
      const away = (i + 1) * tickSize;
      const bidSz = 0.2 + Math.abs(noise(this.tick * 31 + i)) * 5;
      const askSz = 0.2 + Math.abs(noise(this.tick * 67 + i)) * 5;
      bids.push({ price: this.mid - away, size: bidSz });
      asks.push({ price: this.mid + away, size: askSz });
    }
    this.depth = { bids, asks };
  }

  private pushTrade() {
    const n = noise(this.tick * 13);
    this.trades.unshift({
      price: this.mid + n * this.mid * 0.0001,
      size: 0.01 + Math.abs(n) * 2,
      buy: n >= 0,
    });
    if (this.trades.length > 200) this.trades.pop();
  }

  private foldCandle() {
    const startNew = this.tick % TICKS_PER_CANDLE === 1 || this.candles.length === 0;
    if (startNew) {
      this.candles.push({ open: this.mid, high: this.mid, low: this.mid, close: this.mid });
    }
    const c = this.candles[this.candles.length - 1];
    c.high = Math.max(c.high, this.mid);
    c.low = Math.min(c.low, this.mid);
    c.close = this.mid;
    if (this.candles.length > 2_000) this.candles.splice(0, 500);
  }

  private markPositions() {
    this.positions = this.positions.map((p, i) => {
      const drift = noise(this.tick * 7 + (i + 1) * 1000) * 0.002;
      return { ...p, lastPx: p.lastPx * (1 + drift) };
    });
  }

  private refreshQuote() {
    const tickSize = this.mid * 0.0001;
    this.quote = {
      bidPrice: this.mid - tickSize * 0.5,
      askPrice: this.mid + tickSize * 0.5,
    };
  }
}

// Simple moving average of closes ending at index `g` over `period` candles.
export function smaAt(candles: Ohlc[], g: number, period: number): number | null {
  if (period <= 0 || g >= candles.length || g + 1 < period) return null;
  let sum = 0;
  for (let i = g + 1 - period; i <= g; i++) sum += candles[i].close;
  return sum / period;
}

function startingPositions(): Position[] {
  return [
    { symbol: 'BTCUSDT', qty: 0.85, avgPx: 26_500, lastPx: 27_000 },
    { symbol: 'ETHUSDT', qty: -4.2, avgPx: 1_650, lastPx: 1_625 },
    { symbol: 'SOLUSDT', qty: 120, avgPx: 22.4, lastPx: 24.1 },
    { symbol: 'ADAUSDT', qty: -5_000, avgPx: 0.38, lastPx: 0.372 },
    { symbol: 'XRPUSDT', qty: 9_000, avgPx: 0.515, lastPx: 0.508 },
    { symbol: 'DOGEUSDT', qty: 80_000, avgPx: 0.072, lastPx: 0.0735 },
    { symbol: 'AVAXUSDT', qty: -75, avgPx: 11.2, lastPx: 10.95 },
    { symbol: 'LINKUSDT', qty: 340, avgPx: 6.4, lastPx: 6.72 },
  ];
}
