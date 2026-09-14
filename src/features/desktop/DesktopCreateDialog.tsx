import { useEffect, useRef, useState } from 'react';
import { emptySchema } from '../../lib/api';
import { perform } from '../../lib/game-store';
import { softwareCatalog } from '../../lib/software-catalog';

export type DesktopItemKind = 'folder' | 'file' | 'shell' | 'launcher' | 'url';
const titles: Record<DesktopItemKind, string> = {
  folder: 'Criar pasta',
  file: 'Criar documento',
  shell: 'Criar script Shell',
  launcher: 'Criar lançador',
  url: 'Criar link de URL',
};

export function DesktopCreateDialog({
  kind,
  onClose,
}: {
  kind: DesktopItemKind;
  onClose: () => void;
}) {
  const [name, setName] = useState(
    kind === 'folder'
      ? 'Nova pasta'
      : kind === 'shell'
        ? 'script.sh'
        : kind === 'file'
          ? 'Documento.txt'
          : 'Novo atalho',
  );
  const [target, setTarget] = useState(
    kind === 'launcher' ? 'terminal' : 'https://www.wipedia.org',
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);
  return (
    <div className="desktop-create-backdrop" onContextMenu={(e) => e.preventDefault()}>
      <form
        className="desktop-create-dialog"
        role="dialog"
        aria-modal="true"
        aria-label={titles[kind]}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === 'Escape' && !busy) {
            e.preventDefault();
            onClose();
          }
          if (e.key === 'Tab') {
            const fields = Array.from(
              e.currentTarget.querySelectorAll<HTMLElement>(
                'input:not(:disabled),select:not(:disabled),button:not(:disabled)',
              ),
            );
            if (e.shiftKey && document.activeElement === fields[0]) {
              e.preventDefault();
              fields.at(-1)?.focus();
            } else if (!e.shiftKey && document.activeElement === fields.at(-1)) {
              e.preventDefault();
              fields[0]?.focus();
            }
          }
        }}
        onSubmit={(e) => {
          e.preventDefault();
          if (busy) {
            return;
          }
          setBusy(true);
          setError('');
          void perform('desktop_create', { kind, name, target }, emptySchema)
            .then(onClose)
            .catch((reason: unknown) => setError(String(reason)))
            .finally(() => setBusy(false));
        }}
      >
        <h2>{titles[kind]}</h2>
        <p>/home/kali/Desktop</p>
        <label>
          Nome
          <input
            ref={input}
            required
            maxLength={200}
            value={name}
            disabled={busy}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        {kind === 'launcher' && (
          <label>
            Aplicativo
            <select value={target} disabled={busy} onChange={(e) => setTarget(e.target.value)}>
              {softwareCatalog.entries.map((entry) => (
                <option key={entry.id} value={entry.id}>
                  {entry.name}
                </option>
              ))}
            </select>
          </label>
        )}
        {kind === 'url' && (
          <>
            <label>
              Endereço
              <input
                required
                value={target}
                maxLength={2048}
                disabled={busy}
                onChange={(e) => setTarget(e.target.value)}
              />
            </label>
            <p>
              O link abre no navegador do jogo. Apenas páginas da internet virtual são resolvidas.
            </p>
          </>
        )}
        {error && <p role="alert">{error}</p>}
        <footer>
          <button type="button" disabled={busy} onClick={onClose}>
            Cancelar
          </button>
          <button disabled={busy || !name.trim()}>{busy ? 'Criando…' : 'Criar'}</button>
        </footer>
      </form>
    </div>
  );
}
