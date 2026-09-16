import { useState } from 'react';
import {
  startArchiveOperation,
  archiveFormatLabel,
  cancelArchiveJob,
  type ArchiveJob,
  type ArchiveFormat,
  type ArchiveInspection,
  type ArchiveRequest,
} from '../../lib/archive';
import './archive.css';

function useArchiveTask(close: () => void) {
  const [job, setJob] = useState<ArchiveJob | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const run = async (request: ArchiveRequest) => {
    setBusy(true);
    setError('');
    try {
      await startArchiveOperation(request, setJob);
      close();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
      setJob(null);
    }
  };
  const cancel = () => {
    if (job && busy) {
      void cancelArchiveJob(job.id);
    } else {
      close();
    }
  };
  return { job, busy, error, setError, run, cancel };
}

export function CompressDialog({
  paths,
  directory,
  asRoot,
  close,
}: {
  paths: string[];
  directory: string;
  asRoot: boolean;
  close: () => void;
}) {
  const [name, setName] = useState('documentos');
  const [format, setFormat] = useState<ArchiveFormat>('ZIP');
  const [password, setPassword] = useState('');
  const task = useArchiveTask(close);
  const extensions: Partial<Record<ArchiveFormat, string>> = {
    ZIP: '.zip',
    TAR: '.tar',
    TAR_GZIP: '.tar.gz',
    TAR_BZIP2: '.tar.bz2',
    TAR_XZ: '.tar.xz',
  };
  return (
    <div className="archive-overlay">
      <form
        className="archive-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Compactar arquivos"
        onSubmit={(e) => {
          e.preventDefault();
          if (!name.trim() || /[\\/:]/.test(name) || name === '.' || name === '..') {
            task.setError('Digite um nome de arquivo válido.');
            return;
          }
          const suffix = extensions[format] ?? '';
          void task
            .run({
              operation: 'create',
              path: `${directory}/${name.endsWith(suffix) ? name : name + suffix}`,
              format,
              inputs: paths,
              password: format === 'ZIP' && password ? password : undefined,
              asRoot,
            })
            .finally(() => setPassword(''));
        }}
      >
        <h2>Compactar arquivos</h2>
        <p>{paths.length} item(ns) selecionado(s)</p>
        <label>
          Nome do archive
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            disabled={task.busy}
          />
        </label>
        <label>
          Formato
          <select
            value={format}
            onChange={(e) => setFormat(e.target.value as ArchiveFormat)}
            disabled={task.busy}
          >
            {Object.keys(extensions).map((f) => (
              <option key={f} value={f}>
                {archiveFormatLabel(f as ArchiveFormat)}
              </option>
            ))}
          </select>
        </label>
        {format === 'ZIP' && (
          <label>
            Senha (opcional)
            <input
              type="password"
              autoComplete="new-password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={task.busy}
            />
          </label>
        )}
        {task.error && <p role="alert">{task.error}</p>}
        {task.busy && <p role="status">Compactando… {task.job && `${task.job.progress}%`}</p>}
        <footer>
          <button type="button" onClick={task.cancel} disabled={task.busy && !task.job}>
            Cancelar
          </button>
          <button disabled={task.busy}>Criar</button>
        </footer>
      </form>
    </div>
  );
}

export function ExtractDialog({
  archive,
  selected = [],
  destination: initialDestination,
  asRoot,
  close,
}: {
  archive: ArchiveInspection;
  selected?: string[];
  destination: string;
  asRoot: boolean;
  close: () => void;
}) {
  const [destination, setDestination] = useState(initialDestination);
  const [overwrite, setOverwrite] = useState<'ask' | 'skip' | 'replace'>('ask');
  const [password, setPassword] = useState('');
  const task = useArchiveTask(close);
  const stream = ['GZIP', 'BZIP2', 'XZ'].includes(archive.format);
  return (
    <div className="archive-overlay">
      <form
        className="archive-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Extrair archive"
        onSubmit={(e) => {
          e.preventDefault();
          void task
            .run({
              operation: stream ? 'decompress' : 'extract',
              path: archive.path,
              password: password || undefined,
              asRoot,
              options: { destination, overwrite, selected },
            })
            .finally(() => setPassword(''));
        }}
      >
        <h2>Extrair archive</h2>
        <p>
          {archive.path.split('/').pop()} ·{' '}
          {selected.length ? `${selected.length} selecionado(s)` : 'Todos os arquivos'}
        </p>
        <label>
          Destino
          <input
            autoFocus
            value={destination}
            onChange={(e) => setDestination(e.target.value)}
            required
            disabled={task.busy}
          />
        </label>
        <label>
          Se o arquivo já existir
          <select
            value={overwrite}
            onChange={(e) => setOverwrite(e.target.value as typeof overwrite)}
            disabled={task.busy}
          >
            <option value="ask">Avisar e interromper</option>
            {!stream && <option value="skip">Ignorar existentes</option>}
            <option value="replace">Substituir existentes</option>
          </select>
        </label>
        {archive.encrypted && (
          <label>
            Senha
            <input
              type="password"
              autoComplete="off"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              disabled={task.busy}
            />
          </label>
        )}
        {task.error && <p role="alert">{task.error}</p>}
        {task.busy && <p role="status">Extraindo… {task.job && `${task.job.progress}%`}</p>}
        <footer>
          <button type="button" onClick={task.cancel} disabled={task.busy && !task.job}>
            Cancelar
          </button>
          <button disabled={task.busy}>Extrair</button>
        </footer>
      </form>
    </div>
  );
}
