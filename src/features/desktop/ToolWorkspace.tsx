import { useState } from 'react';
import { z } from 'zod';
import type { SoftwareEntry } from '../../lib/software-catalog';
import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { KaliIcon } from './KaliIcon';
import { TrafficInspector, WirelessInspector } from './InvestigationTools';

const reportSchema = z.object({
  title: z.string(),
  body: z.string(),
  savedPath: z.string().nullable(),
});
const operations: Record<string, { name: string; hint: string; target: string }> = {
  host: { name: 'Inspeção de host virtual', hint: 'Host do mundo do jogo', target: 'vex.local' },
  web: {
    name: 'Inspeção de resposta HTTP',
    hint: 'URL de um serviço virtual',
    target: 'https://www.archive.org',
  },
  file: {
    name: 'Análise de arquivo e SHA-256',
    hint: 'Caminho no computador virtual',
    target: '/home/kali/Documents/primeiros-passos.txt',
  },
  wifi: { name: 'Levantamento de redes Wi-Fi', hint: '', target: '' },
  system: { name: 'Inventário do sistema e processos', hint: '', target: '' },
};

export function ToolWorkspace({
  entry,
  initialPath,
}: {
  entry: SoftwareEntry;
  initialPath?: string;
}) {
  const operation = operations[entry.operation];
  const [target, setTarget] = useState(operation.target);
  const [report, setReport] = useState<z.infer<typeof reportSchema> | null>(null);
  const [error, setError] = useState('');
  const [tab, setTab] = useState<'workspace' | 'reference'>(
    entry.resource ? 'reference' : 'workspace',
  );
  const [savePath, setSavePath] = useState(`/home/kali/Documents/${entry.id}-report.txt`);
  const busy = useGame((s) => s.busy);
  const run = (save: boolean) => {
    setError('');
    void perform(
      'tool_run',
      { id: entry.id, target, savePath: save ? savePath : null },
      reportSchema,
    )
      .then(setReport)
      .catch((e: unknown) => setError(String(e)));
  };
  if (entry.operation === 'wifi') {
    return <WirelessInspector />;
  }
  if (entry.package === 'wireshark') {
    return <TrafficInspector initialPath={initialPath} />;
  }
  return (
    <div className="tool-workspace">
      <div className="tool-heading">
        <KaliIcon name={entry.icon} size={48} />
        <div>
          <h2>{entry.name}</h2>
          <p>{entry.description || entry.package}</p>
        </div>
        <span className="tool-edition">Kali default</span>
      </div>
      <nav className="tool-tabs" aria-label="Seções da ferramenta">
        {!entry.resource && (
          <button
            className={tab === 'workspace' ? 'active' : ''}
            onClick={() => setTab('workspace')}
          >
            Laboratório
          </button>
        )}
        <button className={tab === 'reference' ? 'active' : ''} onClick={() => setTab('reference')}>
          Referência
        </button>
      </nav>
      <div className="app-scroll">
        {tab === 'reference' ? (
          <div className="tool-reference">
            <h3>{entry.resource ? entry.name : 'Sobre esta ferramenta'}</h3>
            <p>{entry.description}</p>
            <label>
              Referência oficial
              <input
                readOnly
                value={entry.resource ?? entry.source}
                onFocus={(event) => event.target.select()}
              />
            </label>
            <p className="muted">
              Referência local do catálogo Kali. A navegação e os instrumentos do LifeOS usam o
              mundo simulado do jogo.
            </p>
            {!entry.resource && (
              <>
                <h3>Comandos do pacote {entry.package}</h3>
                <div className="tool-commands">
                  {(entry.commands.length ? entry.commands : [entry.name]).map((command) => (
                    <code key={command}>{command}</code>
                  ))}
                </div>
                <p>
                  O laboratório oferece {operation.name.toLowerCase()}. Os demais módulos e opções
                  do programa original não são emulados.
                </p>
              </>
            )}
            <button onClick={() => useWindows.getState().open('browser')}>
              Abrir navegador do LifeOS
            </button>
          </div>
        ) : (
          <>
            <form
              onSubmit={(event) => {
                event.preventDefault();
                run(false);
              }}
            >
              <h3>{operation.name}</h3>
              {operation.hint && (
                <label>
                  {operation.hint}
                  <input
                    aria-label="Alvo virtual"
                    value={target}
                    onChange={(event) => setTarget(event.target.value)}
                  />
                </label>
              )}
              <div className="toolbar">
                <button className="primary" disabled={busy}>
                  {busy ? 'Inspecionando…' : 'Executar inspeção'}
                </button>
                <span className="muted">Laboratório virtual · {entry.name}</span>
              </div>
            </form>
            {error && (
              <p role="alert" className="error">
                {error}
              </p>
            )}
            {report ? (
              <>
                <pre className="tool-output" aria-label="Resultado da inspeção">
                  {report.body}
                </pre>
                <form
                  className="tool-save"
                  onSubmit={(event) => {
                    event.preventDefault();
                    run(true);
                  }}
                >
                  <label>
                    Salvar relatório no computador virtual
                    <input
                      aria-label="Caminho do relatório"
                      value={savePath}
                      onChange={(event) => setSavePath(event.target.value)}
                    />
                  </label>
                  <button disabled={busy}>Inspecionar e salvar</button>
                </form>
                {report.savedPath && (
                  <p role="status">
                    Salvo em {report.savedPath}{' '}
                    <button
                      onClick={() =>
                        useWindows.getState().open('editor', report.savedPath ?? undefined)
                      }
                    >
                      Abrir relatório
                    </button>
                  </p>
                )}
              </>
            ) : (
              <div className="tool-empty">
                <KaliIcon name={entry.icon} size={64} />
                <p>Execute uma inspeção para consultar o estado atual do laboratório.</p>
                <small>Os resultados usam os hosts, arquivos e processos da campanha.</small>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
