import { useEffect, useRef, useState } from 'react';
import { emptySchema, type VfsNode } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import '../desktop/desktop-context.css';

export type FileNameOperation = { kind: 'file' | 'folder' } | { kind: 'rename'; node: VfsNode };
export function FileNameDialog({
  operation,
  directory,
  asRoot = false,
  onClose,
}: {
  operation: FileNameOperation;
  directory: string;
  asRoot?: boolean;
  onClose: () => void;
}) {
  const [name, setName] = useState(
    operation.kind === 'rename'
      ? operation.node.name
      : operation.kind === 'folder'
        ? 'Nova pasta'
        : 'Documento.txt',
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const input = useRef<HTMLInputElement>(null);
  const previousFocus = useRef(document.activeElement);
  const title =
    operation.kind === 'rename'
      ? 'Renomear item'
      : operation.kind === 'folder'
        ? 'Criar pasta'
        : 'Criar arquivo';
  useEffect(() => {
    const previous = previousFocus.current;
    input.current?.focus();
    input.current?.select();
    return () => {
      if (previous instanceof HTMLElement && previous.isConnected) {
        previous.focus();
      }
    };
  }, []);
  const submit = async () => {
    if (busy) {
      return;
    }
    const trimmed = name.trim();
    if (
      !trimmed ||
      ['.', '..'].includes(trimmed) ||
      /[\\/]/.test(trimmed) ||
      [...trimmed].some((char) => char.charCodeAt(0) < 32)
    ) {
      setError('Informe um nome sem barras ou caracteres de controle.');
      return;
    }
    if (operation.kind === 'rename' && trimmed === operation.node.name) {
      onClose();
      return;
    }
    setBusy(true);
    setError('');
    const path = `${directory.replace(/\/$/, '')}/${trimmed}`;
    if (useGame.getState().world?.vfs.nodes[path]) {
      setError('Já existe um item com esse nome.');
      setBusy(false);
      return;
    }
    try {
      await perform(
        operation.kind === 'rename'
          ? 'vfs_move'
          : operation.kind === 'folder'
            ? 'vfs_create_directory'
            : 'vfs_create_file',
        operation.kind === 'rename'
          ? { source: operation.node.id, destination: path, asRoot }
          : { path, asRoot },
        emptySchema,
      );
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="desktop-create-backdrop">
      <form
        className="desktop-create-dialog"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
        onKeyDown={(event) => {
          event.stopPropagation();
          if (event.key === 'Escape' && !busy) {
            event.preventDefault();
            onClose();
          }
          if (event.key === 'Tab') {
            const fields = Array.from(
              event.currentTarget.querySelectorAll<HTMLElement>(
                'input:not(:disabled),button:not(:disabled)',
              ),
            );
            if (event.shiftKey && document.activeElement === fields[0]) {
              event.preventDefault();
              fields.at(-1)?.focus();
            } else if (!event.shiftKey && document.activeElement === fields.at(-1)) {
              event.preventDefault();
              fields[0]?.focus();
            }
          }
        }}
      >
        <h2>{title}</h2>
        <p>{directory}</p>
        <label>
          Nome
          <input
            ref={input}
            value={name}
            maxLength={200}
            disabled={busy}
            onChange={(event) => setName(event.target.value)}
          />
        </label>
        {error && <p role="alert">{error}</p>}
        <footer>
          <button type="button" disabled={busy} onClick={onClose}>
            Cancelar
          </button>
          <button className="primary" type="submit" disabled={busy}>
            {busy ? 'Salvando…' : operation.kind === 'rename' ? 'Renomear' : 'Criar'}
          </button>
        </footer>
      </form>
    </div>
  );
}
