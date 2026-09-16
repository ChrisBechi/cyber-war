import { useEffect, useRef, useState, type RefObject } from 'react';
import type { BrowserTab } from './browser-model';
import { evaluateBrowserConsole, requestStatus, type BrowserRequest } from './browser-inspection';

type Entry = { command: string; result: string; failed: boolean };
export function BrowserConsole({
  tab,
  pageElement,
  requests,
}: {
  tab: BrowserTab;
  pageElement: RefObject<HTMLDivElement | null>;
  requests: BrowserRequest[];
}) {
  const [draft, setDraft] = useState('');
  const [entries, setEntries] = useState<Entry[]>([]);
  const [commands, setCommands] = useState<string[]>([]);
  const [cursor, setCursor] = useState<number | null>(null);
  const [clearedThrough, setClearedThrough] = useState(0);
  const output = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (output.current) {
      output.current.scrollTop = output.current.scrollHeight;
    }
  }, [entries, requests]);
  const clear = () => {
    setEntries([]);
    setClearedThrough(Math.max(clearedThrough, ...requests.map((request) => request.id)));
  };
  const run = () => {
    const command = draft.trim();
    if (!command) {
      return;
    }
    setDraft('');
    setCursor(null);
    setCommands((items) => [...items, command].slice(-100));
    if (/^(?:clear|console\.clear\(\));?$/.test(command)) {
      clear();
      return;
    }
    let result: string;
    let failed = false;
    try {
      result = evaluateBrowserConsole(command, {
        title: tab.page?.title || (tab.error ? 'Servidor não encontrado' : 'Nova aba'),
        address: tab.address,
        element: pageElement.current,
      });
    } catch (error) {
      result = String(error);
      failed = true;
    }
    setEntries((items) =>
      [...items, { command, result: result.slice(0, 64000), failed }].slice(-100),
    );
  };
  return (
    <>
      <div className="browser-developer-toolbar">
        <span>Console da aba atual · digite help para ver os comandos</span>
        <button onClick={clear}>Limpar console</button>
      </div>
      <div className="browser-console-output" ref={output}>
        <ol className="browser-developer-console" aria-label="Eventos de navegação">
          {requests
            .filter((request) => request.id > clearedThrough)
            .map((request) => (
              <li key={request.id} data-status={request.status}>
                <time>{new Date(request.startedAt).toLocaleTimeString('pt-BR')}</time>{' '}
                {request.operation} · {request.address} · {requestStatus[request.status]}
                {request.error && <pre>{request.error}</pre>}
              </li>
            ))}
        </ol>
        <ol
          className="browser-developer-console"
          aria-label="Resultados do console"
          aria-live="polite"
        >
          {entries.map((entry, index) => (
            <li key={index} data-status={entry.failed ? 'error' : 'success'}>
              <div>› {entry.command}</div>
              <pre>{entry.result}</pre>
            </li>
          ))}
        </ol>
      </div>
      <form
        className="browser-console-form"
        onSubmit={(event) => {
          event.preventDefault();
          run();
        }}
      >
        <span aria-hidden="true">›</span>
        <input
          aria-label="Comando do console"
          autoComplete="off"
          spellCheck={false}
          placeholder="document.title"
          value={draft}
          onChange={(event) => {
            setDraft(event.target.value);
            setCursor(null);
          }}
          onKeyDown={(event) => {
            if (event.key === 'ArrowUp' && commands.length) {
              event.preventDefault();
              const next = Math.max(0, (cursor ?? commands.length) - 1);
              setCursor(next);
              setDraft(commands[next]);
            }
            if (event.key === 'ArrowDown' && cursor !== null) {
              event.preventDefault();
              const next = cursor + 1;
              setCursor(next < commands.length ? next : null);
              setDraft(commands[next] ?? '');
            }
          }}
        />
        <button type="submit">Executar</button>
      </form>
    </>
  );
}
