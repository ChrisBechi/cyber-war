import { useCallback, useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { emptySchema, request } from '../../lib/api';
import { perform } from '../../lib/game-store';

export function Editor({
  initialPath,
  asRoot = false,
}: {
  initialPath?: string;
  asRoot?: boolean;
}) {
  const [path, setPath] = useState(initialPath ?? '/home/kali/Documents/notes.txt');
  const [content, setContent] = useState('');
  const [loaded, setLoaded] = useState<{ path: string; content: string } | null>(null);
  const [dirty, setDirty] = useState(false);
  const [status, setStatus] = useState('');
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<string | null>(null);
  const dirtyRef = useRef(false);
  const requestId = useRef(0);
  const open = useCallback(
    (next: string) => {
      const id = ++requestId.current;
      setBusy(true);
      void request('vfs_read', { path: next, asRoot }, z.string())
        .then((text) => {
          if (id !== requestId.current) {
            return;
          }
          setPath(next);
          setContent(text);
          setLoaded({ path: next, content: text });
          setDirty(false);
          dirtyRef.current = false;
          setStatus('Arquivo aberto');
        })
        .catch((error: unknown) => {
          if (id === requestId.current) {
            setStatus(String(error));
          }
        })
        .finally(() => {
          if (id === requestId.current) {
            setBusy(false);
          }
        });
    },
    [asRoot],
  );
  useEffect(() => {
    const next = initialPath ?? '/home/kali/Documents/notes.txt';
    if (dirtyRef.current) {
      setPending(next);
    } else {
      open(next);
    }
    return () => {
      requestId.current += 1;
    };
  }, [initialPath, open]);
  const save = () => {
    if (busy) {
      return;
    }
    setBusy(true);
    void perform(
      'vfs_write',
      { path, content, asRoot, expectedContent: loaded?.path === path ? loaded.content : null },
      emptySchema,
    )
      .then(() => {
        setLoaded({ path, content });
        setDirty(false);
        dirtyRef.current = false;
        setStatus('Salvo no computador virtual');
      })
      .catch(() => undefined)
      .finally(() => setBusy(false));
  };
  return (
    <div
      className="editor-app"
      onKeyDown={(e) => {
        if (e.ctrlKey && e.key === 's') {
          e.preventDefault();
          save();
        }
      }}
    >
      <div className="toolbar">
        <input
          aria-label="Arquivo no HackPad"
          value={path}
          disabled={busy}
          onChange={(e) => setPath(e.target.value)}
        />
        <button
          disabled={busy}
          onClick={() => {
            if (dirty) {
              setPending(path);
            } else {
              open(path);
            }
          }}
        >
          Abrir
        </button>
        <button className="primary" disabled={busy} onClick={save}>
          Salvar{dirty ? ' *' : ''}
        </button>
      </div>
      {pending && (
        <div className="confirm-panel">
          <p>Abrir {pending} e descartar o texto não salvo?</p>
          <button onClick={() => setPending(null)}>Manter texto</button>
          <button
            onClick={() => {
              open(pending);
              setPending(null);
            }}
          >
            Abrir arquivo
          </button>
        </div>
      )}
      <textarea
        aria-label="Conteúdo do arquivo"
        spellCheck={false}
        value={content}
        disabled={busy}
        onChange={(e) => {
          setContent(e.target.value);
          setDirty(true);
          dirtyRef.current = true;
        }}
      />
      <footer className="app-status">
        {dirty ? 'Alterações não salvas — salve antes de fechar' : status}
        <span>UTF-8 · {content.split('\n').length} linhas</span>
      </footer>
    </div>
  );
}
