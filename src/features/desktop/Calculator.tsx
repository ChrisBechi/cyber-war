import { useState } from 'react';

export function Calculator() {
  const [display, setDisplay] = useState('0');
  const [previous, setPrevious] = useState<number | null>(null);
  const [operator, setOperator] = useState('');
  const [replace, setReplace] = useState(false);
  const operate = (left: number, right: number, op: string) =>
    op === '+'
      ? left + right
      : op === '−'
        ? left - right
        : op === '×'
          ? left * right
          : left / right;
  const press = (key: string) => {
    if (key === 'C') {
      setDisplay('0');
      setPrevious(null);
      setOperator('');
      setReplace(false);
      return;
    }
    if (/^\d$/.test(key)) {
      setDisplay((value) =>
        replace || value === '0' ? key : value.length < 16 ? value + key : value,
      );
      setReplace(false);
      return;
    }
    if (key === '.') {
      if (replace) {
        setDisplay('0.');
      } else if (!display.includes('.')) {
        setDisplay(display + '.');
      }
      setReplace(false);
      return;
    }
    if (key === '⌫') {
      setDisplay((value) => (value.length > 1 ? value.slice(0, -1) : '0'));
      return;
    }
    if (key === '±') {
      setDisplay(String(-Number(display)));
      return;
    }
    if (key === '%') {
      setDisplay(String(Number(display) / 100));
      return;
    }
    if (['+', '−', '×', '÷', '='].includes(key)) {
      const result =
        previous !== null && operator && !replace
          ? operate(previous, Number(display), operator)
          : Number(display);
      setDisplay(Number.isFinite(result) ? String(Number(result.toPrecision(12))) : 'Erro');
      setPrevious(key === '=' ? null : result);
      setOperator(key === '=' ? '' : key);
      setReplace(true);
    }
  };
  return (
    <div
      className="calculator"
      onKeyDown={(event) => {
        const key =
          (
            {
              Enter: '=',
              Escape: 'C',
              Backspace: '⌫',
              '*': '×',
              '/': '÷',
              '-': '−',
              ',': '.',
            } as Record<string, string>
          )[event.key] ?? event.key;
        if (/^[0-9.+%=]$/.test(key) || ['C', '⌫', '×', '÷', '−'].includes(key)) {
          event.preventDefault();
          press(key);
        }
      }}
    >
      <p className="muted">Calculadora</p>
      <div className="calculator-display">
        <small>{previous !== null ? `${previous} ${operator}` : '\u00a0'}</small>
        <output aria-label="Resultado">{display}</output>
      </div>
      <div className="calculator-keys">
        {[
          'C',
          '⌫',
          '%',
          '÷',
          '7',
          '8',
          '9',
          '×',
          '4',
          '5',
          '6',
          '−',
          '1',
          '2',
          '3',
          '+',
          '±',
          '0',
          '.',
          '=',
        ].map((key) => (
          <button
            autoFocus={key === '7'}
            key={key}
            className={key === '=' ? 'primary' : ''}
            onClick={() => press(key)}
          >
            {key}
          </button>
        ))}
      </div>
    </div>
  );
}
