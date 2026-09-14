import { useState } from 'react';

import { commandSchema } from '../../lib/api';

import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';

export function CodeLab() {
  const world = useGame((s) => s.world);
  const [value, setValue] = useState('100');
  const [output, setOutput] = useState('Conecte ideias. Observe o que mudou.');
  const busy = useGame((s) => s.busy);
  const run = (command: string) => {
    void perform('execute_terminal', { command }, commandSchema)
      .then((r) => setOutput(r.stderr || r.stdout))
      .catch((e: unknown) => setOutput(String(e)));
  };
  return (
    <div className="app-scroll codelab">
      <div className="section-heading">
        <span className="eyebrow">CODELAB / LABORATÓRIO PESSOAL</span>
        <h2>O valor não é o endereço.</h2>
        <p className="muted">
          Compare a memória antes e depois de jogar. Elimine os candidatos que não acompanharam a
          mudança.
        </p>
      </div>
      <div className="lab-panels">
        <section>
          <small>CYBER SIEGE / PROCESSO VIRTUAL</small>
          <strong className="memory-value">{world?.memoryValue}</strong>
          <span>VIDA</span>
          <button onClick={() => run('lab play')} disabled={busy}>
            Jogar um turno (−1)
          </button>
        </section>
        <section>
          <small>MEMORY SCANNER</small>
          <form
            className="toolbar"
            onSubmit={(e) => {
              e.preventDefault();
              if (/^\d+$/.test(value)) {
                run(`lab scan ${value}`);
              }
            }}
          >
            <input
              type="number"
              aria-label="Valor para escanear"
              value={value}
              onChange={(e) => setValue(e.target.value)}
            />
            <button disabled={busy}>Escanear</button>
          </form>
          <div className="addresses">
            {world?.memoryCandidates.map((n) => (
              <code key={n}>0x{n.toString(16).toUpperCase()}</code>
            ))}
          </div>
        </section>
      </div>
      <div className="toolbar">
        <button disabled={busy} onClick={() => run('lab build-v1')}>
          Construir V1
        </button>
        <button disabled={busy} onClick={() => run('lab inspect')}>
          Inspecionar runtime
        </button>
        <button disabled={busy} onClick={() => run('lab build-v2')}>
          Construir V2
        </button>
      </div>
      <pre className="lab-output" role="status">
        {output}
      </pre>
      <p className="muted">
        Outra hipótese? Os arquivos de perfil estão em{' '}
        <button
          className="text-button"
          onClick={() => useWindows.getState().open('files', '/home/kali/projects')}
        >
          ~/projects
        </button>
        .
      </p>
    </div>
  );
}
