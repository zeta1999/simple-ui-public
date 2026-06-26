// Candlestick chart — the desktop twin of simple-ui-widgets' Chart. A small,
// self-contained SVG renderer: one candle (wick + body) per bar, green up / red
// down, with an SMA(period) overlay drawn as a polyline. Kept dependency-free so
// the drawing is fully visible here rather than hidden behind a chart library.

import { smaAt, type Ohlc } from './feed';

interface Props {
  candles: Ohlc[];
  visible?: number; // how many of the most recent candles to show (zoom)
  smaPeriod?: number;
  height?: number;
}

export function Chart({ candles, visible = 80, smaPeriod = 20, height = 360 }: Props) {
  const shown = candles.slice(Math.max(0, candles.length - visible));
  if (shown.length === 0) return <div className="chart-empty">no data</div>;

  // Price range over the shown window, mapped into the SVG's pixel box.
  const lo = Math.min(...shown.map((c) => c.low));
  const hi = Math.max(...shown.map((c) => c.high));
  const pad = 8;
  const W = 1000; // viewBox width; SVG scales to the container via width="100%"
  const H = height;
  const colW = W / shown.length;
  const y = (price: number) =>
    pad + (1 - (price - lo) / Math.max(1e-9, hi - lo)) * (H - 2 * pad);

  const firstShown = candles.length - shown.length;

  return (
    <svg className="chart" viewBox={`0 0 ${W} ${H}`} width="100%" height={H} preserveAspectRatio="none">
      {shown.map((c, i) => {
        const cx = i * colW + colW / 2;
        const up = c.close >= c.open;
        const color = up ? 'var(--success)' : 'var(--danger)';
        const bodyTop = y(Math.max(c.open, c.close));
        const bodyBot = y(Math.min(c.open, c.close));
        const bodyW = Math.max(1, colW * 0.6);
        return (
          <g key={i} stroke={color} fill={color}>
            {/* Wick: high → low */}
            <line x1={cx} y1={y(c.high)} x2={cx} y2={y(c.low)} strokeWidth={1} />
            {/* Body: open ↔ close */}
            <rect
              x={cx - bodyW / 2}
              y={bodyTop}
              width={bodyW}
              height={Math.max(1, bodyBot - bodyTop)}
            />
          </g>
        );
      })}

      {/* SMA overlay as a single polyline over the shown candles. */}
      <polyline
        fill="none"
        stroke="var(--accent-secondary)"
        strokeWidth={1.5}
        points={shown
          .map((_, i) => {
            const v = smaAt(candles, firstShown + i, smaPeriod);
            return v === null ? '' : `${i * colW + colW / 2},${y(v)}`;
          })
          .filter(Boolean)
          .join(' ')}
      />
    </svg>
  );
}
