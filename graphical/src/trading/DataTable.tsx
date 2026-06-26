// DataTable — the desktop twin of simple-ui-widgets' DataTable. A column/row
// grid with click-to-sort headers (▲/▼) and a click-to-select highlighted row.
// Generic over the row type so it backs any blotter/watchlist.

import { useState } from 'react';

export interface ColumnDef<T> {
  title: string;
  align?: 'left' | 'right';
  // The sortable value and the displayed cell (with optional color).
  value: (row: T) => number | string;
  render?: (row: T) => { text: string; color?: string };
}

interface Props<T> {
  columns: ColumnDef<T>[];
  rows: T[];
  rowKey: (row: T) => string;
}

export function DataTable<T>({ columns, rows, rowKey }: Props<T>) {
  const [sortCol, setSortCol] = useState<number | null>(null);
  const [ascending, setAscending] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);

  // Clicking a header sorts by it; clicking the active header flips direction.
  const onHeaderClick = (i: number) => {
    if (sortCol === i) setAscending((a) => !a);
    else {
      setSortCol(i);
      setAscending(true);
    }
  };

  const sorted = [...rows];
  if (sortCol !== null) {
    const col = columns[sortCol];
    sorted.sort((a, b) => {
      const av = col.value(a);
      const bv = col.value(b);
      const cmp = typeof av === 'number' && typeof bv === 'number'
        ? av - bv
        : String(av).localeCompare(String(bv));
      return ascending ? cmp : -cmp;
    });
  }

  return (
    <table className="grid">
      <thead>
        <tr>
          {columns.map((c, i) => (
            <th
              key={c.title}
              className={c.align === 'right' ? 'right' : 'left'}
              onClick={() => onHeaderClick(i)}
            >
              {c.title}
              {sortCol === i ? (ascending ? ' ▲' : ' ▼') : ''}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {sorted.map((row) => {
          const key = rowKey(row);
          return (
            <tr
              key={key}
              className={selected === key ? 'selected' : ''}
              onClick={() => setSelected(key)}
            >
              {columns.map((c) => {
                const cell = c.render ? c.render(row) : { text: String(c.value(row)) };
                return (
                  <td
                    key={c.title}
                    className={c.align === 'right' ? 'right' : 'left'}
                    style={cell.color ? { color: cell.color } : undefined}
                  >
                    {cell.text}
                  </td>
                );
              })}
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
