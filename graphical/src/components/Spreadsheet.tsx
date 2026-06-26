import React, { useState, useEffect, useRef } from 'react';
import { EvaluatorWrapper } from 'markdown_engine';

interface SpreadsheetProps {
  data: (string | number)[][];
}

export function Spreadsheet({ data }: SpreadsheetProps) {
  const [gridData, setGridData] = useState<(string | number)[][]>(data);
  const [evaluatedData, setEvaluatedData] = useState<string[][]>([]);
  const [activeCell, setActiveCell] = useState<{r: number, c: number} | null>(null);
  const [editValue, setEditValue] = useState<string>("");
  const inputRef = useRef<HTMLInputElement>(null);
  const evaluatorRef = useRef<EvaluatorWrapper | null>(null);

  useEffect(() => {
    // Initialize evaluator
    try {
      evaluatorRef.current = new EvaluatorWrapper();
    } catch (e) {
      console.error("Failed to init EvaluatorWrapper", e);
    }
  }, []);

  useEffect(() => {
    evaluateGrid();
  }, [gridData]);

  useEffect(() => {
    if (activeCell && inputRef.current) {
      inputRef.current.focus();
    }
  }, [activeCell]);

  const indexToColName = (index: number) => {
    return String.fromCharCode(65 + index); // 0 -> A, 1 -> B
  };

  const evaluateGrid = () => {
    const newEvaluated = Array(gridData.length).fill(null).map(() => Array(gridData[0].length).fill(""));
    const context: Record<string, number> = {};

    // First pass: extract literal values for context
    for (let r = 0; r < gridData.length; r++) {
      for (let c = 0; c < gridData[r].length; c++) {
        const val = gridData[r][c];
        const cellName = `${indexToColName(c)}${r + 1}`;
        if (typeof val === 'number') {
          context[cellName] = val;
          newEvaluated[r][c] = val.toString();
        } else if (typeof val === 'string' && !val.startsWith('=')) {
          const numVal = parseFloat(val);
          if (!isNaN(numVal)) {
            context[cellName] = numVal;
          }
          newEvaluated[r][c] = val;
        }
      }
    }

    // Second pass: evaluate formulas
    for (let r = 0; r < gridData.length; r++) {
      for (let c = 0; c < gridData[r].length; c++) {
        const val = gridData[r][c];
        if (typeof val === 'string' && val.startsWith('=')) {
          if (evaluatorRef.current) {
            const expression = val.substring(1);
            try {
              // Convert Rhai expression e.g. SUM(A1:B2) is not natively supported by Rhai default eval
              // The rust evaluator should support B2 * C2.
              const result = evaluatorRef.current.eval(expression, context);
              newEvaluated[r][c] = result.toFixed(2);
            } catch (e) {
              console.warn(`Evaluation failed for ${expression}`, e);
              newEvaluated[r][c] = "#ERROR";
            }
          } else {
            newEvaluated[r][c] = "#LOADING";
          }
        }
      }
    }

    setEvaluatedData(newEvaluated);
  };

  const handleCellClick = (r: number, c: number) => {
    setActiveCell({ r, c });
    setEditValue(gridData[r][c].toString());
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      finishEditing();
    } else if (e.key === 'Escape') {
      setActiveCell(null);
    }
  };

  const finishEditing = () => {
    if (activeCell) {
      const newGrid = [...gridData];
      const { r, c } = activeCell;
      
      // Try to parse as number if it looks like one and isn't a formula
      let finalVal: string | number = editValue;
      if (!editValue.startsWith('=')) {
        const numVal = parseFloat(editValue);
        if (!isNaN(numVal) && numVal.toString() === editValue.trim()) {
          finalVal = numVal;
        }
      }
      
      newGrid[r][c] = finalVal;
      setGridData(newGrid);
      setActiveCell(null);
    }
  };

  if (evaluatedData.length === 0) return null;

  return (
    <div style={{ overflowX: 'auto', borderRadius: 'var(--radius-md)', border: '1px solid var(--border-strong)' }}>
      <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
        <thead>
          <tr style={{ background: 'var(--bg-tertiary)' }}>
            <th style={{ padding: '0.5rem', borderBottom: '1px solid var(--border-strong)', borderRight: '1px solid var(--border-strong)', width: '40px' }}></th>
            {gridData[0].map((_, c) => (
              <th key={c} style={{ padding: '0.5rem', borderBottom: '1px solid var(--border-strong)', borderRight: '1px solid var(--border-strong)' }}>
                {indexToColName(c)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {evaluatedData.map((row, r) => (
            <tr key={r} style={{ borderBottom: '1px solid var(--border-strong)' }}>
              <td style={{ padding: '0.5rem', background: 'var(--bg-tertiary)', borderRight: '1px solid var(--border-strong)', fontWeight: 'bold' }}>
                {r + 1}
              </td>
              {row.map((val, c) => (
                <td 
                  key={c} 
                  onClick={() => handleCellClick(r, c)}
                  style={{ 
                    padding: '0.5rem', 
                    borderRight: '1px solid var(--border-strong)',
                    cursor: 'cell',
                    background: activeCell?.r === r && activeCell?.c === c ? 'var(--accent-glass)' : 'transparent',
                    boxShadow: activeCell?.r === r && activeCell?.c === c ? 'inset 0 0 0 2px var(--accent-primary)' : 'none'
                  }}
                >
                  {activeCell?.r === r && activeCell?.c === c ? (
                    <input
                      ref={inputRef}
                      type="text"
                      value={editValue}
                      onChange={e => setEditValue(e.target.value)}
                      onKeyDown={handleKeyDown}
                      onBlur={finishEditing}
                      style={{ 
                        width: '100%', 
                        background: 'transparent', 
                        border: 'none', 
                        outline: 'none', 
                        padding: 0,
                        margin: 0,
                        color: 'var(--text-primary)'
                      }}
                    />
                  ) : (
                    <span>{val}</span>
                  )}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
