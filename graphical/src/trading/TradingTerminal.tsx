// The desktop trading terminal: the same four widgets as the TUI example
// (chart, depth ladder, time & sales, positions blotter), driven by the same
// synthetic feed (feed.ts) on a timer. This is the desktop counterpart of
// `examples/widget-examples/src/bin/trading_terminal.rs`.

import { useEffect, useRef, useState } from 'react';
import { Market, pnl, type Position } from './feed';
import { DepthLadder } from './DepthLadder';
import { TimeAndSales } from './TimeAndSales';
import { Chart } from './Chart';
import { DataTable, type ColumnDef } from './DataTable';
import './terminal.css';

const signed = (v: number) => (v >= 0 ? 'var(--success)' : 'var(--danger)');

// Positions-blotter columns — value() drives sorting, render() the displayed cell.
const blotterColumns: ColumnDef<Position>[] = [
  { title: 'Symbol', align: 'left', value: (p) => p.symbol },
  { title: 'Qty', align: 'right', value: (p) => p.qty,
    render: (p) => ({ text: p.qty.toFixed(4), color: signed(p.qty) }) },
  { title: 'Avg Px', align: 'right', value: (p) => p.avgPx,
    render: (p) => ({ text: p.avgPx.toFixed(2) }) },
  { title: 'Last', align: 'right', value: (p) => p.lastPx,
    render: (p) => ({ text: p.lastPx.toFixed(2) }) },
  { title: 'PnL', align: 'right', value: (p) => pnl(p),
    render: (p) => ({ text: pnl(p).toFixed(2), color: signed(pnl(p)) }) },
];

export function TradingTerminal() {
  // The market lives in a ref (mutated in place); a tick counter triggers redraws.
  const market = useRef(new Market());
  const [, setTick] = useState(0);

  useEffect(() => {
    // Advance the feed ~12×/sec, then ask React to repaint.
    const id = setInterval(() => {
      market.current.step();
      setTick((t) => t + 1);
    }, 1000 / 12);
    return () => clearInterval(id);
  }, []);

  const m = market.current;
  return (
    // Two columns: chart + blotter on the left, ladder + tape on the right —
    // the same arrangement as the TUI example, laid out with flexbox so each
    // panel fits or scrolls rather than overflowing.
    <div className="terminal">
      <div className="terminal-col left">
        <Panel title="BTCUSDT — Chart" grow>
          <Chart candles={m.candles} visible={80} smaPeriod={20} />
        </Panel>
        <Panel title="Positions — click a header to sort, a row to select" scroll>
          <DataTable columns={blotterColumns} rows={m.positions} rowKey={(p) => p.symbol} />
        </Panel>
      </div>
      <div className="terminal-col right">
        <Panel title="Depth">
          <DepthLadder depth={m.depth} quote={m.quote} levels={6} />
        </Panel>
        <Panel title="Time & Sales" grow scroll>
          <TimeAndSales trades={m.trades} rows={40} />
        </Panel>
      </div>
    </div>
  );
}

// A titled, bordered panel — the desktop equivalent of a ratatui Block.
// `grow` makes it expand to fill the column; `scroll` lets its body scroll.
function Panel({
  title,
  grow,
  scroll,
  children,
}: {
  title: string;
  grow?: boolean;
  scroll?: boolean;
  children: React.ReactNode;
}) {
  return (
    <section className={`panel ${grow ? 'grow' : ''}`}>
      <header className="panel-title">{title}</header>
      <div className={`panel-body ${scroll ? 'scroll' : ''}`}>{children}</div>
    </section>
  );
}
