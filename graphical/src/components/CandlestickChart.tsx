import {
  ComposedChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer
} from 'recharts';

interface CandlestickData {
  dates: string[];
  open: number[];
  high: number[];
  low: number[];
  close: number[];
  title?: string;
}

interface ChartProps {
  data: CandlestickData;
}

export function CandlestickChart({ data }: ChartProps) {
  // Transform the separate arrays into a single array of objects for Recharts
  const chartData = data.dates.map((date, i) => {
    const open = data.open[i];
    const close = data.close[i];
    const high = data.high[i];
    const low = data.low[i];
    
    return {
      date,
      open,
      close,
      high,
      low,
      // For a simple bar representation of candlestick body
      bodyBottom: Math.min(open, close),
      bodyTop: Math.max(open, close),
      isGreen: close >= open
    };
  });

  const CustomCandlestick = (props: any) => {
    const { x, y, width, height, isGreen } = props;
    const color = isGreen ? 'var(--success)' : 'var(--danger)';
    
    // We get bodyBottom and bodyTop from the Bar's domain logic, 
    // but we need to render the high/low wicks manually.
    // Recharts doesn't have a native Candlestick, so this is a simplified composite.
    
    return (
      <g>
        <rect x={x} y={y} width={width} height={Math.max(height, 2)} fill={color} rx={2} />
      </g>
    );
  };

  return (
    <div style={{ width: '100%', height: 400 }}>
      <ResponsiveContainer>
        <ComposedChart data={chartData} margin={{ top: 20, right: 30, left: 20, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border-strong)" vertical={false} />
          <XAxis dataKey="date" stroke="var(--text-muted)" />
          <YAxis domain={['auto', 'auto']} stroke="var(--text-muted)" />
          <Tooltip 
            contentStyle={{ backgroundColor: 'var(--bg-secondary)', border: '1px solid var(--border-strong)', borderRadius: 'var(--radius-md)' }}
            formatter={(value: any, name: any) => [value, name === 'bodyTop' ? 'Close/Open' : name]}
          />
          <Bar dataKey="bodyTop" shape={<CustomCandlestick />} />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}
