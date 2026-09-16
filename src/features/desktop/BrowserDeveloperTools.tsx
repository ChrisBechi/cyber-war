import { useEffect, useId, useState, type RefObject } from 'react';
import type { BrowserPreferences, BrowserTab } from './browser-model';
import { requestStatus, serializeBrowserPage, type BrowserRequest } from './browser-inspection';
import { BrowserConsole } from './BrowserConsole';
import './browser-developer-tools.css';

export type DeveloperTab = 'console' | 'network' | 'storage' | 'session' | 'source';
const panels: { id: DeveloperTab; label: string }[] = [
  { id: 'console', label: 'Console' },
  { id: 'network', label: 'Rede' },
  { id: 'storage', label: 'Storage' },
  { id: 'session', label: 'Sessão' },
  { id: 'source', label: 'Código fonte' },
];

function PageSource({
  pageElement,
  tab,
}: {
  pageElement: RefObject<HTMLDivElement | null>;
  tab: BrowserTab;
}) {
  const [source, setSource] = useState('');
  useEffect(() => {
    const element = pageElement.current;
    if (!element) {
      setSource('');
      return;
    }
    const update = () =>
      setSource(
        serializeBrowserPage(
          element,
          tab.page?.title || (tab.error ? 'Servidor não encontrado' : 'Nova aba'),
        ),
      );
    update();
    const observer = new MutationObserver(update);
    observer.observe(element, {
      childList: true,
      subtree: true,
      characterData: true,
      attributes: true,
    });
    return () => observer.disconnect();
  }, [pageElement, tab.id, tab.request, tab.page, tab.error, tab.loading]);
  return (
    <>
      <p className="browser-developer-muted">
        HTML da página exibida · {tab.address || 'Nova aba'}
      </p>
      {tab.loading ? (
        <p role="status">Aguardando a página…</p>
      ) : (
        <textarea
          className="browser-developer-source"
          aria-label="Código fonte da página"
          readOnly
          spellCheck={false}
          value={source}
        />
      )}
    </>
  );
}

export function BrowserDeveloperTools({
  activeTab,
  setActiveTab,
  tab,
  tabs,
  history,
  preferences,
  savedPreferences,
  clearHistory,
  requests,
  clearRequests,
  pageElement,
  onClose,
}: {
  activeTab: DeveloperTab;
  setActiveTab: (tab: DeveloperTab) => void;
  tab: BrowserTab;
  tabs: BrowserTab[];
  history: { address: string; title: string }[];
  preferences: BrowserPreferences;
  savedPreferences?: string;
  clearHistory: () => void;
  requests: BrowserRequest[];
  clearRequests: () => void;
  pageElement: RefObject<HTMLDivElement | null>;
  onClose: () => void;
}) {
  const id = useId();
  const [filter, setFilter] = useState('');
  const [selectedRequest, setSelectedRequest] = useState<number | null>(null);
  const visibleRequests = requests.filter((request) => request.tabId === tab.id);
  const selected = visibleRequests.find((request) => request.id === selectedRequest);
  const records = visibleRequests.filter((request) =>
    `${request.address} ${request.operation} ${request.error}`
      .toLowerCase()
      .includes(filter.toLowerCase()),
  );
  return (
    <section className="browser-developer-tools" aria-label="Modo desenvolvedor">
      <header className="browser-developer-heading">
        <strong>Modo desenvolvedor</strong>
        <button type="button" aria-label="Fechar modo desenvolvedor" onClick={onClose}>
          ×
        </button>
      </header>
      <div className="browser-developer-tabs" role="tablist" aria-label="Painéis do desenvolvedor">
        {panels.map((item, index) => (
          <button
            key={item.id}
            role="tab"
            id={`${id}-${item.id}`}
            aria-controls={`${id}-panel`}
            tabIndex={activeTab === item.id ? 0 : -1}
            aria-selected={activeTab === item.id}
            onClick={() => setActiveTab(item.id)}
            onKeyDown={(event) => {
              if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
                return;
              }
              event.preventDefault();
              const next =
                event.key === 'Home'
                  ? 0
                  : event.key === 'End'
                    ? panels.length - 1
                    : (index + (event.key === 'ArrowRight' ? 1 : -1) + panels.length) %
                      panels.length;
              setActiveTab(panels[next].id);
              document.getElementById(`${id}-${panels[next].id}`)?.focus();
            }}
          >
            {item.label}
          </button>
        ))}
      </div>
      <div
        id={`${id}-panel`}
        className="browser-developer-panel"
        role="tabpanel"
        aria-labelledby={`${id}-${activeTab}`}
      >
        {tabs.map((item) => (
          <div
            key={item.id}
            className="browser-console-panel"
            hidden={activeTab !== 'console' || item.id !== tab.id}
          >
            <BrowserConsole
              tab={item}
              pageElement={pageElement}
              requests={requests.filter((request) => request.tabId === item.id)}
            />
          </div>
        ))}
        {activeTab === 'network' && (
          <>
            <div className="browser-developer-toolbar">
              <input
                aria-label="Filtrar requisições"
                placeholder="Filtrar URL ou operação"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
              />
              <button onClick={clearRequests}>Limpar registros</button>
            </div>
            <p className="browser-developer-muted">
              Requisições da aba atual · até 100 registros por sessão
            </p>
            <table className="browser-developer-network">
              <thead>
                <tr>
                  <th>Endereço</th>
                  <th>Operação</th>
                  <th>Resultado</th>
                  <th>Tempo</th>
                </tr>
              </thead>
              <tbody>
                {records.map((request) => (
                  <tr key={request.id}>
                    <td>
                      <button
                        className="browser-developer-request"
                        onClick={() => setSelectedRequest(request.id)}
                      >
                        {request.address}
                      </button>
                    </td>
                    <td>{request.operation}</td>
                    <td>{requestStatus[request.status]}</td>
                    <td>{request.durationMs === null ? '—' : `${request.durationMs} ms`}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {!records.length && <p>Nenhuma requisição encontrada.</p>}
            {selected && (
              <dl className="browser-developer-storage">
                <div>
                  <dt>Endereço</dt>
                  <dd>{selected.address}</dd>
                </div>
                <div>
                  <dt>Resultado</dt>
                  <dd>{requestStatus[selected.status]}</dd>
                </div>
                {selected.error && (
                  <div>
                    <dt>Erro</dt>
                    <dd>{selected.error}</dd>
                  </div>
                )}
              </dl>
            )}
          </>
        )}
        {activeTab === 'storage' && (
          <>
            <h3>Preferências salvas na campanha</h3>
            <p className="browser-developer-muted">
              {savedPreferences
                ? 'browserPreferences · persistido no save'
                : 'Configuração padrão · ainda não personalizada'}
            </p>
            <dl className="browser-developer-storage">
              <div>
                <dt>home</dt>
                <dd>{preferences.home}</dd>
              </div>
              <div>
                <dt>zoom</dt>
                <dd>{preferences.zoom}%</dd>
              </div>
              <div>
                <dt>showBookmarks</dt>
                <dd>{String(preferences.showBookmarks)}</dd>
              </div>
              <div>
                <dt>bookmarks</dt>
                <dd>{preferences.bookmarks.length} favoritos</dd>
              </div>
            </dl>
            <details>
              <summary>Valores dos favoritos</summary>
              <pre>{JSON.stringify(preferences.bookmarks, null, 2)}</pre>
            </details>
            <h3>Armazenamento por site</h3>
            <p>
              As páginas atuais não utilizam cookies, localStorage ou sessionStorage por domínio.
            </p>
          </>
        )}
        {activeTab === 'session' && (
          <>
            <h3>Sessão do navegador</h3>
            <dl className="browser-developer-storage">
              <div>
                <dt>Aba ativa</dt>
                <dd>{tab.address || 'Nova aba'}</dd>
              </div>
              <div>
                <dt>Abas abertas</dt>
                <dd>{tabs.length}</dd>
              </div>
              <div>
                <dt>Histórico</dt>
                <dd>{history.length} página(s) nesta sessão</dd>
              </div>
              <div>
                <dt>Navegação da aba</dt>
                <dd>
                  {tab.index} anterior(es) · {tab.history.length - tab.index - 1} seguinte(s)
                </dd>
              </div>
            </dl>
            <ul>
              {tabs.map((item) => (
                <li key={item.id}>
                  {item.page?.title || item.address || 'Nova aba'}
                  {item.id === tab.id ? ' · ativa' : ''}
                </li>
              ))}
            </ul>
            <button onClick={clearHistory}>Limpar histórico da sessão</button>
            <p className="browser-developer-muted">
              Abas e histórico são encerrados ao fechar o navegador. Favoritos e preferências
              permanecem no save.
            </p>
          </>
        )}
        {activeTab === 'source' && <PageSource pageElement={pageElement} tab={tab} />}
      </div>
    </section>
  );
}
