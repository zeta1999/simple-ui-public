import { useState, useEffect } from 'react';
import { Layers, FileText, LineChart } from 'lucide-react';
import './index.css';
import { MarkdownRenderer } from './components/MarkdownRenderer';
import { TradingTerminal } from './trading/TradingTerminal';

// The two desktop views: the markdown document, and the trading terminal that
// renders the same widget set as the TUI examples.
type View = 'document' | 'terminal';

// We'll load the WASM dynamically to handle async init
import init, { parse_markdown_js } from 'markdown_engine';

function App() {
  const [view, setView] = useState<View>('terminal');
  const [wasmReady, setWasmReady] = useState(false);
  const [markdown, setMarkdown] = useState('');
  const [parsedAst, setParsedAst] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Initialize WASM module
    async function loadWasm() {
      try {
        await init();
        setWasmReady(true);
        // Load initial demo content
        const demoContent = await fetch('/demo.md').then(res => res.text());
        setMarkdown(demoContent);
      } catch (err) {
        console.error("Failed to initialize WASM", err);
        setError("Failed to initialize core engine.");
      }
    }
    loadWasm();
  }, []);

  useEffect(() => {
    if (wasmReady && markdown) {
      try {
        const ast = parse_markdown_js(markdown);
        setParsedAst(ast);
        setError(null);
      } catch (err) {
        console.error("Parsing error:", err);
        setError(String(err));
      }
    }
  }, [markdown, wasmReady]);

  return (
    <>
      <header className="app-header">
        <div className="app-brand">
          <Layers size={24} style={{ display: 'inline-block', verticalAlign: 'middle', marginRight: '8px' }} />
          Simple UI
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <nav className="view-switch">
            <button
              className={view === 'terminal' ? 'active' : ''}
              onClick={() => setView('terminal')}
            >
              <LineChart size={16} /> Terminal
            </button>
            <button
              className={view === 'document' ? 'active' : ''}
              onClick={() => setView('document')}
            >
              <FileText size={16} /> Document
            </button>
          </nav>
          <span style={{ color: 'var(--text-muted)', fontSize: '0.9rem' }}>
            {wasmReady ? 'Engine Ready' : 'Initializing...'}
          </span>
        </div>
      </header>

      <main className="app-container">
        {error && (
          <div style={{ padding: '1rem', background: 'var(--danger)', color: 'white', borderRadius: 'var(--radius-md)', marginBottom: '1rem' }}>
            {error}
          </div>
        )}
        
        {view === 'terminal' ? (
          // The trading terminal is pure React + the synthetic feed — no WASM needed.
          <TradingTerminal />
        ) : !wasmReady ? (
          <div style={{ textAlign: 'center', padding: '4rem', color: 'var(--text-muted)' }}>
            Loading Core Engine...
          </div>
        ) : (
          <div className="content-area">
            {parsedAst ? (
              <MarkdownRenderer ast={parsedAst} />
            ) : (
              <p>No content</p>
            )}
          </div>
        )}
      </main>
    </>
  );
}

export default App;
