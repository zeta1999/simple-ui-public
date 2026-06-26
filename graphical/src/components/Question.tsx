import { useState } from 'react';

interface QuestionProps {
  data: {
    id: string;
    question: string;
    options: string[];
    allowOther?: boolean;
  };
}

export function Question({ data }: QuestionProps) {
  const [selected, setSelected] = useState<string | null>(null);
  const [otherText, setOtherText] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const handleSubmit = () => {
    if (selected) {
      setSubmitted(true);
      // In a real app, this would emit via IPC to the daemon
      console.log(`Submitted answer for ${data.id}:`, selected === 'other' ? otherText : selected);
    }
  };

  if (submitted) {
    return (
      <div style={{ textAlign: 'center', padding: '2rem' }}>
        <div style={{ color: 'var(--success)', fontSize: '2rem', marginBottom: '1rem' }}>✓</div>
        <h3>Response Recorded</h3>
        <p>Thank you for your input.</p>
        <button className="btn btn-secondary" onClick={() => setSubmitted(false)}>Reset</button>
      </div>
    );
  }

  return (
    <div>
      <h3 style={{ marginBottom: '1.5rem', marginTop: 0 }}>{data.question}</h3>
      
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', marginBottom: '1.5rem' }}>
        {Array.isArray(data.options) ? data.options.map((opt, i) => (
          <label 
            key={i} 
            style={{ 
              display: 'flex', 
              alignItems: 'center', 
              gap: '0.75rem',
              padding: '1rem',
              background: selected === opt ? 'var(--accent-glass)' : 'var(--bg-secondary)',
              border: `1px solid ${selected === opt ? 'var(--accent-primary)' : 'var(--border-strong)'}`,
              borderRadius: 'var(--radius-md)',
              cursor: 'pointer',
              transition: 'all var(--transition-fast)'
            }}
          >
            <input 
              type="radio" 
              name={`q-${data.id}`} 
              value={opt}
              checked={selected === opt}
              onChange={() => setSelected(opt)}
              style={{ width: 'auto' }}
            />
            {opt}
          </label>
        )) : (
          <div style={{ color: 'var(--warning)', padding: '1rem', background: 'var(--bg-secondary)', borderRadius: 'var(--radius-md)' }}>
            Warning: Invalid options format provided in Markdown.
          </div>
        )}
        
        {data.allowOther && (
          <label 
            style={{ 
              display: 'flex', 
              alignItems: 'center', 
              gap: '0.75rem',
              padding: '1rem',
              background: selected === 'other' ? 'var(--accent-glass)' : 'var(--bg-secondary)',
              border: `1px solid ${selected === 'other' ? 'var(--accent-primary)' : 'var(--border-strong)'}`,
              borderRadius: 'var(--radius-md)',
              cursor: 'pointer',
              transition: 'all var(--transition-fast)'
            }}
          >
            <input 
              type="radio" 
              name={`q-${data.id}`} 
              value="other"
              checked={selected === 'other'}
              onChange={() => setSelected('other')}
              style={{ width: 'auto' }}
            />
            Other
            {selected === 'other' && (
              <input 
                type="text" 
                value={otherText}
                onChange={e => setOtherText(e.target.value)}
                placeholder="Please specify..."
                style={{ marginLeft: '1rem', width: '200px' }}
                autoFocus
              />
            )}
          </label>
        )}
      </div>
      
      <button 
        className="btn btn-primary" 
        onClick={handleSubmit}
        disabled={!selected || (selected === 'other' && !otherText)}
      >
        Submit Response
      </button>
    </div>
  );
}
