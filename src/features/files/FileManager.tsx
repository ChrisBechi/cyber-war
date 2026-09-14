import { useEffect, useRef, useState } from 'react';
import { importVirtualFile } from '../../lib/import-file';
import { z } from 'zod';
import { nodeSchema, request } from '../../lib/api';
import type { VfsNode } from '../../lib/api';
import { act, useGame } from '../../lib/game-store';
import { apps } from '../../lib/window-store';
import type { AppId } from '../../lib/window-store';
import { isTrashPath, TRASH_FILES_PATH } from '../../lib/vfs-paths';
import { AppIcon } from '../desktop/AppIcon';
import { associationForPath } from '../../lib/file-associations';
import { openVfsNode } from '../../lib/file-open';
import { pasteVfs, useVfsClipboard } from '../../lib/vfs-clipboard';
import {
  defaultVfsFoldersFirst,
  defaultVfsSort,
  defaultVfsSortDirection,
  sortVfsNodes,
  type VfsSort,
  type VfsSortDirection,
} from '../../lib/vfs-order';

export function FileManager({
  initialPath,
  asRoot = false,
}: {
  initialPath?: string;
  asRoot?: boolean;
}) {
  const [path, setPath] = useState(initialPath ?? '/home/kali');
  const [address, setAddress] = useState(path);
  const [nodes, setNodes] = useState<VfsNode[]>([]);
  const [name, setName] = useState('notes.txt');
  const [selection, setSelection] = useState<VfsNode | null>(null);
  const [destination, setDestination] = useState('/home/kali/Documents');
  const [error, setError] = useState('');
  const [importing, setImporting] = useState(false);
  const importRef = useRef<HTMLInputElement>(null);
  const revision = useGame((s) => s.revision);
  const world = useGame((s) => s.world);
  const clipboard = useVfsClipboard((s) => s.entry);
  const fileAction = (command: string, args: Record<string, unknown> = {}) =>
    act(command, { ...args, asRoot });
  const inTrash = isTrashPath(path);
  const trashHasItems = Object.values(world?.vfs.nodes ?? {}).some(
    (node) => node.parentId === TRASH_FILES_PATH,
  );
  const sort = (world?.settings.fileSort as VfsSort | undefined) ?? defaultVfsSort;
  const sortDirection =
    (world?.settings.fileSortDirection as VfsSortDirection | undefined) ?? defaultVfsSortDirection;
  const foldersFirst = world?.settings.fileFoldersFirst !== 'false' && defaultVfsFoldersFirst;
  const displayedNodes = sortVfsNodes(
    inTrash ? nodes : nodes.filter((node) => !node.name.startsWith('.')),
    sort,
    sortDirection,
    foldersFirst,
  );
  useEffect(() => {
    if (initialPath) {
      setPath(initialPath);
      setAddress(initialPath);
    }
  }, [initialPath]);
  useEffect(() => {
    let current = true;
    void request('vfs_list', { path, asRoot }, z.array(nodeSchema))
      .then((data) => {
        if (current) {
          setNodes(data);
          setError('');
          setSelection(null);
        }
      })
      .catch((e: unknown) => {
        if (current) {
          setError(String(e));
          setNodes([]);
        }
      });
    return () => {
      current = false;
    };
  }, [path, revision, asRoot]);
  const navigate = (next: string) => {
    setPath(next);
    setAddress(next);
  };
  const open = (node: VfsNode) => {
    if (inTrash || isTrashPath(node.id)) {
      setError('Itens na lixeira não podem ser abertos. Restaure o item primeiro.');
      return;
    }
    if (node.kind === 'directory') {
      navigate(node.id);
    } else {
      void openVfsNode(node, asRoot).catch((reason: unknown) => setError(String(reason)));
    }
  };
  return (
    <div className="file-manager">
      {asRoot && (
        <div className="file-root-banner">
          Administrador root — somente arquivos do computador virtual
        </div>
      )}
      <form
        className="toolbar"
        onSubmit={(e) => {
          e.preventDefault();
          navigate(address);
        }}
      >
        <button type="button" onClick={() => navigate(path.slice(0, path.lastIndexOf('/')) || '/')}>
          ↑
        </button>
        <input aria-label="Caminho" value={address} onChange={(e) => setAddress(e.target.value)} />
        <button>Abrir</button>
        <button
          type="button"
          disabled={inTrash || importing}
          onClick={() => importRef.current?.click()}
        >
          {importing ? 'Importando…' : 'Importar arquivo…'}
        </button>
        <input
          ref={importRef}
          type="file"
          hidden
          aria-label="Importar arquivo para o sistema virtual"
          onChange={(event) => {
            const file = event.target.files?.[0];
            event.target.value = '';
            if (!file) {
              return;
            }
            setImporting(true);
            setError('');
            void importVirtualFile(file, path, asRoot)
              .catch((reason: unknown) => setError(String(reason)))
              .finally(() => setImporting(false));
          }}
        />
      </form>
      <div className="file-layout">
        <aside>
          {[
            '/home/kali',
            '/home/kali/Desktop',
            '/home/kali/Documents',
            '/home/kali/Downloads',
            '/home/kali/projects',
            '/',
          ].map((p) => (
            <button key={p} onClick={() => navigate(p)} className={path === p ? 'selected' : ''}>
              <AppIcon name="folder" /> {p.split('/').pop() || 'Sistema'}
            </button>
          ))}
          <button
            onClick={() => navigate(TRASH_FILES_PATH)}
            className={inTrash ? 'selected' : ''}
            aria-label="Lixeira"
          >
            <AppIcon name="trash" trashFull={trashHasItems} /> Lixeira
          </button>
        </aside>
        <div className="files-main">
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
          <div className="toolbar file-sort-toolbar">
            <label>
              Ordenar por
              <select
                aria-label="Ordenar arquivos por"
                value={sort}
                onChange={(event) =>
                  act('setting_update', { key: 'fileSort', value: event.target.value })
                }
              >
                <option value="name">Nome</option>
                <option value="modified">Modificação</option>
                <option value="kind">Tipo</option>
              </select>
            </label>
            <button
              type="button"
              aria-label={sortDirection === 'asc' ? 'Ordem crescente' : 'Ordem decrescente'}
              onClick={() =>
                act('setting_update', {
                  key: 'fileSortDirection',
                  value: sortDirection === 'asc' ? 'desc' : 'asc',
                })
              }
            >
              {sortDirection === 'asc' ? '↑' : '↓'}
            </button>
            <label>
              <input
                type="checkbox"
                checked={foldersFirst}
                onChange={(event) =>
                  act('setting_update', {
                    key: 'fileFoldersFirst',
                    value: String(event.target.checked),
                  })
                }
              />{' '}
              Pastas primeiro
            </label>
          </div>
          <div className="file-grid">
            {displayedNodes.map((node) => (
              <button
                key={node.id}
                className={selection?.id === node.id ? 'file selected' : 'file'}
                onClick={() => setSelection(node)}
                onDoubleClick={() => open(node)}
                draggable={!inTrash}
                onDragStart={(e) => {
                  e.dataTransfer.effectAllowed = 'move';
                  e.dataTransfer.setData('application/x-cyber-war-vfs', node.id);
                  e.dataTransfer.setData('text/plain', node.id);
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    open(node);
                  }
                }}
              >
                <AppIcon
                  name={
                    node.metadata.app && node.metadata.app in apps
                      ? (node.metadata.app as AppId)
                      : node.kind === 'directory'
                        ? 'folder'
                        : associationForPath(node.id, node).application
                  }
                  size={40}
                />
                <span>{node.name}</span>
              </button>
            ))}
          </div>
          {displayedNodes.length === 0 && !error && (
            <p className="muted">{inTrash ? 'A lixeira está vazia.' : 'Esta pasta está vazia.'}</p>
          )}
        </div>
      </div>
      {!inTrash && (
        <div className="toolbar">
          <input
            aria-label="Nome do novo arquivo ou pasta"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <button onClick={() => fileAction('vfs_create_file', { path: `${path}/${name}` })}>
            Novo arquivo
          </button>
          <button onClick={() => fileAction('vfs_create_directory', { path: `${path}/${name}` })}>
            Nova pasta
          </button>
          <button
            disabled={!clipboard || !world?.vfs.nodes[clipboard.path]}
            onClick={() => {
              void pasteVfs(path, asRoot).catch((reason: unknown) => setError(String(reason)));
            }}
          >
            Colar
          </button>
        </div>
      )}
      {selection && (
        <div className="file-details">
          <span>
            {selection.name} · {selection.owner} · {selection.mode.toString(8)} ·{' '}
            {selection.blob
              ? `${selection.blob.size} bytes`
              : `${selection.content.length} caracteres`}
          </span>
          {selection.kind === 'file' && (
            <small className="file-association">
              Abre com: {associationForPath(selection.id, selection).label}
            </small>
          )}
          {inTrash ? (
            <div className="toolbar">
              <small>
                Local original: {selection.metadata.trashOriginalPath ?? 'desconhecido'}
              </small>
              <button onClick={() => fileAction('vfs_trash_restore', { path: selection.id })}>
                Restaurar
              </button>
              <button
                onClick={() => {
                  if (window.confirm(`Excluir ${selection.name} permanentemente?`)) {
                    fileAction('vfs_remove', { path: selection.id, recursive: true });
                  }
                }}
              >
                Excluir permanentemente
              </button>
            </div>
          ) : (
            <div className="toolbar">
              <button onClick={() => open(selection)}>Abrir</button>
              <button onClick={() => useVfsClipboard.getState().copy(selection.id)}>
                Copiar para área de transferência
              </button>
              <button onClick={() => useVfsClipboard.getState().copy(selection.id, true)}>
                Recortar
              </button>
              <input
                aria-label="Destino para mover ou renomear"
                value={destination}
                onChange={(e) => setDestination(e.target.value)}
              />
              <button onClick={() => fileAction('vfs_move', { source: selection.id, destination })}>
                Mover / renomear
              </button>
              <button onClick={() => fileAction('vfs_copy', { source: selection.id, destination })}>
                Copiar
              </button>
              <button
                onClick={() => {
                  if (window.confirm(`Enviar ${selection.name} para a lixeira?`)) {
                    fileAction('vfs_remove', { path: selection.id, recursive: true });
                  }
                }}
              >
                Apagar
              </button>
            </div>
          )}
        </div>
      )}
      <footer className="app-status">
        {displayedNodes.length} itens <span>Computador / {path}</span>
        {inTrash && displayedNodes.length > 0 && (
          <button
            onClick={() => {
              if (window.confirm('Esvaziar a lixeira permanentemente?')) {
                fileAction('vfs_trash_empty');
              }
            }}
          >
            Esvaziar lixeira
          </button>
        )}
      </footer>
    </div>
  );
}
