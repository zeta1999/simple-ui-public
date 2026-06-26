// Time & sales tape — the desktop twin of simple-ui-widgets' TimeAndSales.
// Streaming trade prints, newest at the top, green for buyer-aggressed and red
// for seller-aggressed. The feed keeps `trades` newest-first and bounded.

import type { Trade } from './feed';

interface Props {
  trades: Trade[];
  rows?: number;
}

export function TimeAndSales({ trades, rows = 18 }: Props) {
  return (
    <div className="tape">
      {trades.slice(0, rows).map((t, i) => (
        <div className={`tape-row ${t.buy ? 'buy' : 'sell'}`} key={i}>
          <span className="tape-side">{t.buy ? 'B' : 'S'}</span>
          <span className="tape-price">{t.price.toFixed(2)}</span>
          <span className="tape-size">{t.size.toFixed(6)}</span>
        </div>
      ))}
    </div>
  );
}
