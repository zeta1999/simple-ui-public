// Depth-of-book ladder — the desktop twin of simple-ui-widgets' DepthLadder.
// Asks above the mid (red), bids below (green), a size bar per level, and the
// bot's own resting quote marked with a ◀ on the nearest level.

import type { Depth, Quote } from './feed';

// Index of the level whose price is closest to `price` (-1 if empty).
function nearestIndex(levels: { price: number }[], price: number): number {
  let best = -1;
  let bestDist = Infinity;
  levels.forEach((l, i) => {
    const d = Math.abs(l.price - price);
    if (d < bestDist) {
      bestDist = d;
      best = i;
    }
  });
  return best;
}

interface Props {
  depth: Depth;
  quote: Quote;
  levels?: number;
}

export function DepthLadder({ depth, quote, levels = 8 }: Props) {
  const bids = depth.bids.slice(0, levels);
  const asks = depth.asks.slice(0, levels);
  const maxSize = Math.max(1e-9, ...bids.map((l) => l.size), ...asks.map((l) => l.size));

  const bestBid = bids[0]?.price ?? 0;
  const bestAsk = asks[0]?.price ?? 0;
  const mid = (bestBid + bestAsk) / 2;
  const spread = bestAsk - bestBid;

  const markedAsk = nearestIndex(asks, quote.askPrice);
  const markedBid = nearestIndex(bids, quote.bidPrice);

  // One ladder row: a size bar (width ∝ size) behind the price and size text.
  const row = (l: { price: number; size: number }, side: 'ask' | 'bid', marked: boolean) => (
    <div className={`ladder-row ${side}`} key={`${side}-${l.price}`}>
      <span className="ladder-bar" style={{ width: `${(l.size / maxSize) * 100}%` }} />
      <span className="ladder-mark">{marked ? '◀' : ''}</span>
      <span className="ladder-price">{l.price.toFixed(2)}</span>
      <span className="ladder-size">{l.size.toFixed(4)}</span>
    </div>
  );

  return (
    <div className="ladder">
      {/* Asks worst→best so the best ask sits just above the mid row. */}
      {asks.map((l, i) => row(l, 'ask', i === markedAsk)).reverse()}
      <div className="ladder-mid">
        ── mid {mid.toFixed(2)} · spr {spread.toFixed(4)} · mine{' '}
        {quote.bidPrice.toFixed(2)}/{quote.askPrice.toFixed(2)} ──
      </div>
      {bids.map((l, i) => row(l, 'bid', i === markedBid))}
    </div>
  );
}
