import { CandlestickChart } from './CandlestickChart';
import { Spreadsheet } from './Spreadsheet';
import { Question } from './Question';

interface MarkdownRendererProps {
  ast: {
    blocks: any[];
  };
}

export function MarkdownRenderer({ ast }: MarkdownRendererProps) {
  if (!ast || !ast.blocks) return null;

  return (
    <div className="md-container">
      {ast.blocks.map((block, index) => {
        // block is an object where the key is the block type enum variant
        
        if (block.Markdown) {
          // Simple parsing of text block into paragraphs/headers
          const lines = block.Markdown.split('\n');
          return (
            <div key={index} className="md-block">
              {lines.map((line: string, i: number) => {
                if (line.startsWith('# ')) return <h1 key={i}>{line.substring(2)}</h1>;
                if (line.startsWith('## ')) return <h2 key={i}>{line.substring(3)}</h2>;
                if (line.startsWith('### ')) return <h3 key={i}>{line.substring(4)}</h3>;
                if (line.trim() === '') return <br key={i} />;
                return <p key={i}>{line}</p>;
              })}
            </div>
          );
        }
        
        if (block.Plot) {
          const plotData = block.Plot;
          if (plotData.type === 'candlestick') {
            return (
              <div key={index} className="interactive-block">
                <h3>{plotData.title}</h3>
                <CandlestickChart data={plotData} />
              </div>
            );
          }
          return <div key={index}>Unsupported Plot Type</div>;
        }

        if (block.Spreadsheet) {
          return (
            <div key={index} className="interactive-block">
              <h3>Interactive Spreadsheet</h3>
              <Spreadsheet data={block.Spreadsheet.data} />
            </div>
          );
        }

        if (block.Question) {
          return (
            <div key={index} className="interactive-block">
              <Question data={block.Question} />
            </div>
          );
        }

        return <div key={index}>Unknown Block Type</div>;
      })}
    </div>
  );
}
