import { useEffect, useState } from 'react';
import {
  archiveChildren,
  archiveOperation,
  archiveSize,
  archiveFormatLabel,
  type ArchiveInspection,
} from '../../lib/archive';
import { ExtractDialog } from './ArchiveDialogs';
import { ArchiveIcon } from './ArchiveIcon';
import './archive.css';

export function ArchiveViewer({
  initialPath,
  asRoot = false,
}: {
  initialPath?: string;
  asRoot?: boolean;
}) {
  const [archive, setArchive] = useState<ArchiveInspection | null>(null);
  const [directory, setDirectory] = useState('');
  const [selected, setSelected] = useState<string[]>([]);
  const [extract, setExtract] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState('');
  const [busy, setBusy] = useState(false);
  const [password, setPassword] = useState('');
  const [testing, setTesting] = useState(false);
  const test = () => {
    if (!archive) {
      return;
    }
    setBusy(true);
    setError('');
    void archiveOperation({
      operation: 'test',
      path: archive.path,
      password: password || undefined,
      asRoot,
    })
      .then(() => {
        setStatus('Nenhum erro de integridade detectado.');
        setArchive((info) => (info ? { ...info, integrity: 'VALID' } : info));
        setTesting(false);
      })
      .catch((e: unknown) => setError(String(e)))
      .finally(() => {
        setBusy(false);
        setPassword('');
      });
  };
  useEffect(() => {
    let current = true;
    setArchive(null);
    setError('');
    setDirectory('');
    setSelected([]);
    setExtract(false);
    if (initialPath) {
      void archiveOperation({ operation: 'inspect', path: initialPath, asRoot })
        .then((result) => {
          if (current) {
            setArchive(result.inspection);
          }
        })
        .catch((e: unknown) => {
          if (current) {
            setError(String(e));
          }
        });
    }
    return () => {
      current = false;
    };
  }, [initialPath, asRoot]);
  const stream = archive && ['GZIP', 'BZIP2', 'XZ'].includes(archive.format);
  return (
    <section className="archive-viewer" aria-label="Archive Viewer">
      <header className="archive-heading">
        <ArchiveIcon format={archive?.format ?? 'ZIP'} />
        <div>
          <h2>{initialPath?.split('/').pop() ?? 'Archive Viewer'}</h2>
          <small>{initialPath ?? 'Abra um archive pelo gerenciador de arquivos.'}</small>
        </div>
      </header>
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      {archive && (
        <>
          <div className="toolbar">
            <button
              disabled={busy}
              onClick={() => {
                setExtract(true);
              }}
            >
              {stream ? 'Descompactar' : selected.length ? 'Extrair seleção…' : 'Extrair…'}
            </button>
            <button
              disabled={busy}
              onClick={() => {
                if (archive.encrypted) {
                  setTesting(true);
                } else {
                  test();
                }
              }}
            >
              Testar integridade
            </button>
            <button
              onClick={() => {
                setDirectory(directory.split('/').slice(0, -1).join('/'));
                setSelected([]);
              }}
              disabled={!directory}
            >
              ↑
            </button>
            <span>/{directory}</span>
          </div>
          <div className="archive-table-scroll">
            <table>
              <thead>
                <tr>
                  <th aria-label="Selecionar" />
                  <th>Nome</th>
                  <th>Tamanho</th>
                  <th>Compactado</th>
                  <th>Tipo</th>
                  <th>Modificado</th>
                </tr>
              </thead>
              <tbody>
                {archiveChildren(archive.entries, directory).map((entry) => (
                  <tr
                    key={entry.path}
                    onDoubleClick={() => {
                      if (entry.type === 'directory') {
                        setDirectory(entry.path);
                        setSelected([]);
                      }
                    }}
                  >
                    <td>
                      <input
                        type="checkbox"
                        aria-label={`Selecionar ${entry.path}`}
                        checked={selected.includes(entry.path)}
                        onChange={(e) =>
                          setSelected(
                            e.target.checked
                              ? [...selected, entry.path]
                              : selected.filter((p) => p !== entry.path),
                          )
                        }
                      />
                    </td>
                    <td>
                      <button
                        className="archive-entry-name"
                        onClick={() => {
                          if (entry.type === 'directory') {
                            setDirectory(entry.path);
                            setSelected([]);
                          } else {
                            setSelected([entry.path]);
                          }
                        }}
                      >
                        {entry.type === 'directory' ? '▸ ' : ''}
                        {entry.path.split('/').pop()}
                        {entry.linkTarget && ` → ${entry.linkTarget}`}
                      </button>
                    </td>
                    <td>{archiveSize(entry.originalSize)}</td>
                    <td>
                      {entry.compressedSize === null ? '—' : archiveSize(entry.compressedSize)}
                    </td>
                    <td>{entry.type}</td>
                    <td>{entry.modifiedAt}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {stream && <p>Um único arquivo comprimido.</p>}
          </div>
          <details className="archive-properties">
            <summary>Propriedades</summary>
            <dl>
              <dt>Tipo</dt>
              <dd>{archiveFormatLabel(archive.format)}</dd>
              <dt>Local</dt>
              <dd>{archive.path}</dd>
              <dt>Tamanho compactado</dt>
              <dd>{archiveSize(archive.compressedSize)}</dd>
              <dt>Tamanho original</dt>
              <dd>{archive.originalSize ? archiveSize(archive.originalSize) : '—'}</dd>
              <dt>Redução</dt>
              <dd>
                {archive.originalSize
                  ? `${Math.max(0, 100 - (archive.compressedSize / archive.originalSize) * 100).toFixed(1)}%`
                  : '—'}
              </dd>
              <dt>Arquivos / pastas</dt>
              <dd>
                {archive.entries.filter((e) => e.type !== 'directory').length} /{' '}
                {archive.entries.filter((e) => e.type === 'directory').length}
              </dd>
              <dt>Criptografado</dt>
              <dd>{archive.encrypted ? 'Sim' : 'Não'}</dd>
              <dt>Integridade</dt>
              <dd>
                {
                  {
                    VALID: 'Verificada',
                    PARTIAL: 'Índice lido · conteúdo não verificado',
                    CORRUPTED: 'Corrompido',
                    ENCRYPTED: 'Senha necessária para verificar',
                    UNSUPPORTED: 'Formato não suportado',
                  }[archive.integrity]
                }
              </dd>
              <dt>Criado</dt>
              <dd>{archive.createdAt ?? '—'}</dd>
              <dt>Modificado</dt>
              <dd>{archive.modifiedAt}</dd>
              {selected.length === 1 && (
                <>
                  <dt>Permissões da seleção</dt>
                  <dd>
                    {archive.entries.find((e) => e.path === selected[0])?.permissions.toString(8)}
                  </dd>
                </>
              )}
            </dl>
          </details>
          {extract && (
            <ExtractDialog
              archive={archive}
              selected={selected}
              destination={archive.path.slice(0, archive.path.lastIndexOf('/')) || '/'}
              asRoot={asRoot}
              close={() => setExtract(false)}
            />
          )}
          {testing && (
            <div className="archive-overlay">
              <form
                className="archive-dialog"
                role="dialog"
                aria-modal="true"
                aria-label="Testar archive protegido"
                onSubmit={(e) => {
                  e.preventDefault();
                  test();
                }}
              >
                <h2>Testar archive protegido</h2>
                <label>
                  Senha
                  <input
                    autoFocus
                    type="password"
                    autoComplete="off"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    disabled={busy}
                  />
                </label>
                {error && <p role="alert">{error}</p>}
                <footer>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => {
                      setTesting(false);
                      setPassword('');
                    }}
                  >
                    Cancelar
                  </button>
                  <button disabled={busy}>Testar</button>
                </footer>
              </form>
            </div>
          )}
        </>
      )}
      {(busy || status) && <p role="status">{busy ? 'Processando…' : status}</p>}
    </section>
  );
}
