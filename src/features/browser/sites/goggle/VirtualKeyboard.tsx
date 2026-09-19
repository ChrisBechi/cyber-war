import { useState } from 'react';
import { GoggleIcon } from './GoggleIcon';

export function VirtualKeyboard({
  insert,
  backspace,
  submit,
  close,
}: {
  insert: (text: string) => void;
  backspace: () => void;
  submit: () => void;
  close: () => void;
}) {
  const [shift, setShift] = useState(false);
  return (
    <section
      aria-label="Teclado virtual QWERTY"
      className="goggle-keyboard"
      onMouseDown={(e) => e.preventDefault()}
    >
      <header>
        <span>Teclado virtual · QWERTY</span>
        <button type="button" aria-label="Fechar teclado" onClick={close}>
          <GoggleIcon name="close" />
        </button>
      </header>
      {['1234567890', 'qwertyuiop', 'asdfghjkl', 'zxcvbnm'].map((row) => (
        <div className="goggle-keyboard-row" key={row}>
          {Array.from(shift ? row.toUpperCase() : row).map((key) => (
            <button type="button" key={key} onClick={() => insert(key)}>
              {key}
            </button>
          ))}
        </div>
      ))}
      <div className="goggle-keyboard-row">
        <button type="button" aria-pressed={shift} onClick={() => setShift(!shift)}>
          Shift
        </button>
        <button type="button" className="goggle-space" onClick={() => insert(' ')}>
          Espaço
        </button>
        <button type="button" aria-label="Backspace" onClick={backspace}>
          <GoggleIcon name="backspace" />
        </button>
        <button type="button" onClick={submit}>
          Enter
        </button>
      </div>
    </section>
  );
}
