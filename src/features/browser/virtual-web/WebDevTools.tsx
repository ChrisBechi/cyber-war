import { useState } from 'react';
import { z } from 'zod';
import { request } from '../../../lib/api';

const named = z.object({ id: z.string(), name: z.string().optional() }).passthrough();
const diagnosticSchema = z.object({
  pageStatus: z.number().nullable(),
  documentId: z.string().nullable(),
  seed: z.number(),
  pack: z.number(),
  worldSeconds: z.number(),
  resolveMicros: z.number(),
  materializationCache: z.number(),
  cacheBefore: z.unknown(),
  cacheAfter: z.unknown(),
  brands: z.array(named),
  entities: z.array(named),
  events: z.array(named),
  activeEvents: z.array(z.string()),
  documents: z.array(
    z.object({
      id: z.string(),
      title: z.string(),
      url: z.string(),
      visible: z.boolean(),
      source: z.string(),
      entities: z.array(z.string()),
    }),
  ),
  results: z.array(
    z.object({ id: z.string(), title: z.string(), url: z.string(), components: z.unknown() }),
  ),
  totalResults: z.number(),
});
type Report = z.infer<typeof diagnosticSchema>;
type Panel = 'documents' | 'results' | 'brands' | 'entities' | 'events';

export function WebDevTools({
  address,
  navigate,
}: {
  address: string;
  navigate: (url: string) => void;
}) {
  const [query, setQuery] = useState('');
  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState('');
  const [panel, setPanel] = useState<Panel>('documents');
  const [filter, setFilter] = useState('');
  const [pending, setPending] = useState(false);
  if (!import.meta.env.DEV) {
    return null;
  }
  return (
    <details className="web-devtools">
      <summary>Ferramentas de desenvolvimento</summary>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          setPending(true);
          setError('');
          void request('web_debug', { query, address }, diagnosticSchema)
            .then(setReport)
            .catch((e: unknown) => setError(String(e)))
            .finally(() => setPending(false));
        }}
      >
        <label>
          Consulta ou documento
          <input value={query} maxLength={200} onChange={(e) => setQuery(e.target.value)} />
        </label>
        <button disabled={pending}>Inspecionar</button>
      </form>
      {error && <p role="alert">{error}</p>}
      {report ? (
        <>
          <p>
            HTTP {report.pageStatus ?? '—'} · {report.documentId ?? 'listagem'} · pack {report.pack}{' '}
            · seed {report.seed} · tempo {report.worldSeconds}s · resolução{' '}
            {(report.resolveMicros / 1000).toFixed(2)}ms
          </p>
          <details>
            <summary>Cache e eventos ativos</summary>
            <pre>
              {JSON.stringify(
                {
                  before: report.cacheBefore,
                  after: report.cacheAfter,
                  materializations: report.materializationCache,
                  activeEvents: report.activeEvents,
                },
                null,
                2,
              )}
            </pre>
          </details>
          <div className="web-debug-tabs">
            {(['documents', 'results', 'brands', 'entities', 'events'] as const).map((key) => (
              <button key={key} aria-pressed={panel === key} onClick={() => setPanel(key)}>
                {
                  {
                    documents: 'Documentos',
                    results: 'Ranking',
                    brands: 'Marcas',
                    entities: 'Entidades',
                    events: 'Eventos',
                  }[key]
                }{' '}
                ({key === 'results' ? report.totalResults : report[key].length})
              </button>
            ))}
          </div>
          <label>
            Filtrar painel <input value={filter} onChange={(e) => setFilter(e.target.value)} />
          </label>
          <div className="web-debug-records">
            {report[panel]
              .filter((row) =>
                JSON.stringify(row)
                  .toLocaleLowerCase('pt-BR')
                  .includes(filter.toLocaleLowerCase('pt-BR')),
              )
              .map((row) => (
                <details key={row.id}>
                  <summary>
                    {'title' in row && typeof row.title === 'string'
                      ? row.title
                      : 'name' in row && typeof row.name === 'string'
                        ? row.name
                        : row.id}
                    {'visible' in row && row.visible === false ? ' · oculto' : ''}
                  </summary>
                  {'url' in row && typeof row.url === 'string' && (
                    <button onClick={() => navigate(row.url as string)}>Abrir endereço</button>
                  )}
                  <pre>{JSON.stringify(row, null, 2)}</pre>
                </details>
              ))}
          </div>
          {panel === 'documents' && (
            <small>
              Até 100 documentos por consulta. A busca e a abertura mantêm as regras normais de
              visibilidade.
            </small>
          )}
        </>
      ) : (
        <p>Marcas, entidades, eventos, visibilidade e pontuação dos resultados.</p>
      )}
    </details>
  );
}
