import { useContext, useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { importVirtualFile } from '../../lib/import-file';
import { z } from 'zod';
import { nodeSchema, request } from '../../lib/api';
import type { VfsNode } from '../../lib/api';
import { act, useGame } from '../../lib/game-store';
import { apps } from '../../lib/window-store';
import type { AppId } from '../../lib/window-store';
import { isTrashPath, TRASH_FILES_PATH } from '../../lib/vfs-paths';
import { AppIcon } from '../desktop/AppIcon';
import { CompressDialog, ExtractDialog } from './ArchiveDialogs';
import { ArchiveIcon } from './ArchiveIcon';
import { archiveOperation, type ArchiveInspection } from '../../lib/archive';
import { associationForPath } from '../../lib/file-associations';
import { openVfsNode } from '../../lib/file-open';
import { pasteVfs, useVfsClipboard } from '../../lib/vfs-clipboard';
import { dropTargetForNode, startVfsDrag, transferVfsItems } from '../../lib/vfs-drag';
import { useVfsDrop } from '../../lib/use-vfs-drop';
import { fileKeyboardAction } from '../../lib/file-keyboard';
import { FileNameDialog, type FileNameOperation } from './FileNameDialog';
import { WindowHeaderContext } from '../desktop/window-header-context';
import { FilesIcon, FolderIcon, type FilesSymbol } from './FilesIcon';
import './files.css';
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
  const [editingAddress, setEditingAddress] = useState(false);
  const [history, setHistory] = useState({ paths: [path], index: 0 });
  const [refresh, setRefresh] = useState(0);
  const [nodes, setNodes] = useState<VfsNode[]>([]);
  const [selection, setSelection] = useState<VfsNode | null>(null);
  const [destination, setDestination] = useState('/home/kali/Documents');
  const [error, setError] = useState('');
  const drop = useVfsDrop(setError);
  const [importing, setImporting] = useState(false);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [compressPaths, setCompressPaths] = useState<string[] | null>(null);
  const [extractArchive, setExtractArchive] = useState<ArchiveInspection | null>(null);
  const [extractDestination, setExtractDestination] = useState('');
  const [context, setContext] = useState<{ x: number; y: number } | null>(null);
  const [fileOperation, setFileOperation] = useState<FileNameOperation | null>(null);
  const [search, setSearch] = useState('');
  const [showHidden, setShowHidden] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [view, setView] = useState<'grid' | 'list'>('grid');
  const [menu, setMenu] = useState<'actions' | 'view' | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const headerRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const addressRef = useRef<HTMLInputElement>(null);
  const headerSlot = useContext(WindowHeaderContext);
  const importRef = useRef<HTMLInputElement>(null);
  const revision = useGame((s) => s.revision);
  const world = useGame((s) => s.world);
  const clipboard = useVfsClipboard((s) => s.entry);
  const fileAction = (command: string, args: Record<string, unknown> = {}) =>
    act(command, { ...args, asRoot });
  const inTrash = isTrashPath(path);
  const sort = (world?.settings.fileSort as VfsSort | undefined) ?? defaultVfsSort;
  const sortDirection =
    (world?.settings.fileSortDirection as VfsSortDirection | undefined) ?? defaultVfsSortDirection;
  const foldersFirst = world?.settings.fileFoldersFirst !== 'false' && defaultVfsFoldersFirst;
  const displayedNodes = sortVfsNodes(
    (inTrash ? nodes : nodes.filter((node) => showHidden || !node.name.startsWith('.'))).filter(
      (node) => {
        const pattern = search
          .replace(/[.+^${}()|[\]\\]/g, '\\$&')
          .replace(/\*/g, '.*')
          .replace(/\?/g, '.');
        return search.includes('*') || search.includes('?')
          ? new RegExp(`^${pattern}$`, 'i').test(node.name)
          : node.name.toLowerCase().includes(search.toLowerCase());
      },
    ),
    sort,
    sortDirection,
    foldersFirst,
  );
  useEffect(() => {
    if (initialPath) {
      setNodes([]);
      setSelection(null);
      setSelectedPaths([]);
      setPath(initialPath);
      setAddress(initialPath);
      setHistory({ paths: [initialPath], index: 0 });
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
          setSelectedPaths((paths) =>
            paths.filter((path) => data.some((node) => node.id === path)),
          );
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
  }, [path, revision, asRoot, refresh]);
  useEffect(() => {
    if (searchOpen) {
      searchRef.current?.focus();
    }
  }, [searchOpen]);
  useEffect(() => {
    if (!menu && !context) {
      return;
    }
    const dismiss = (event: PointerEvent) => {
      if (!(event.target instanceof Element)) {
        return;
      }
      if (!rootRef.current?.contains(event.target) && !headerRef.current?.contains(event.target)) {
        setMenu(null);
        setContext(null);
      } else if (!event.target.closest('.files-popover, .archive-context, [data-files-menu]')) {
        setMenu(null);
        setContext(null);
      }
    };
    document.addEventListener('pointerdown', dismiss);
    return () => document.removeEventListener('pointerdown', dismiss);
  }, [menu, context]);
  const navigate = (next: string, historyIndex?: number) => {
    if (next === path) {
      return;
    }
    setNodes([]);
    setSelection(null);
    setSelectedPaths([]);
    setContext(null);
    setMenu(null);
    setShowDetails(false);
    setSearch('');
    setEditingAddress(false);
    setHistory((previous) =>
      historyIndex === undefined
        ? {
            paths: [...previous.paths.slice(0, previous.index + 1), next],
            index: previous.index + 1,
          }
        : { ...previous, index: historyIndex },
    );
    setPath(next);
    setAddress(next);
  };
  const startExtract = (node: VfsNode, here: boolean) => {
    setContext(null);
    void archiveOperation({ operation: 'inspect', path: node.id, asRoot })
      .then((result) => {
        if (!result.inspection) {
          return;
        }
        setExtractDestination(here ? path : `${path}/extracted`);
        setExtractArchive(result.inspection);
      })
      .catch((reason: unknown) => setError(String(reason)));
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
  const closeMenus = () => {
    setMenu(null);
    setContext(null);
  };
  const toggleSearch = () => {
    setSearchOpen(!searchOpen);
    if (searchOpen) {
      setSearch('');
    }
  };
  const itemActions = selection && (
    <>
      {inTrash ? (
        <>
          <button
            onClick={() => {
              fileAction('vfs_trash_restore', { path: selection.id });
              closeMenus();
            }}
          >
            Restaurar
          </button>
          <button
            onClick={() => {
              if (window.confirm(`Excluir ${selection.name} permanentemente?`)) {
                fileAction('vfs_remove', { path: selection.id, recursive: true });
              }
              closeMenus();
            }}
          >
            Excluir permanentemente
          </button>
        </>
      ) : (
        <>
          <button
            onClick={() => {
              open(selection);
              closeMenus();
            }}
          >
            Abrir <kbd>Enter</kbd>
          </button>
          <button
            onClick={() => {
              useVfsClipboard
                .getState()
                .copy(selectedPaths.length ? selectedPaths : [selection.id]);
              closeMenus();
            }}
          >
            Copiar <kbd>Ctrl+C</kbd>
          </button>
          <button
            onClick={() => {
              useVfsClipboard
                .getState()
                .copy(selectedPaths.length ? selectedPaths : [selection.id], true);
              closeMenus();
            }}
          >
            Recortar <kbd>Ctrl+X</kbd>
          </button>
          <button
            disabled={selectedPaths.length > 1}
            onClick={() => {
              setFileOperation({ kind: 'rename', node: selection });
              closeMenus();
            }}
          >
            Renomear… <kbd>F2</kbd>
          </button>
          <button
            onClick={() => {
              void transferVfsItems(selectedPaths.length ? selectedPaths : [selection.id], {
                kind: 'trash',
                asRoot,
              }).catch((reason: unknown) => setError(String(reason)));
              closeMenus();
            }}
          >
            Mover para a lixeira <kbd>Delete</kbd>
          </button>
          <hr />
          <button
            onClick={() => {
              setCompressPaths(selectedPaths.length ? selectedPaths : [selection.id]);
              closeMenus();
            }}
          >
            Compactar…
          </button>
          {associationForPath(selection.id, selection).kind === 'archive' && (
            <>
              <button
                onClick={() => {
                  startExtract(selection, true);
                  closeMenus();
                }}
              >
                Extrair aqui
              </button>
              <button
                onClick={() => {
                  startExtract(selection, false);
                  closeMenus();
                }}
              >
                Extrair para…
              </button>
            </>
          )}
        </>
      )}
      <button
        onClick={() => {
          setShowDetails(true);
          closeMenus();
        }}
      >
        Propriedades e ações…
      </button>
    </>
  );
  const folderActions = !inTrash && (
    <>
      <button
        onClick={() => {
          setFileOperation({ kind: 'folder' });
          closeMenus();
        }}
      >
        Nova pasta… <kbd>Ctrl+Shift+N</kbd>
      </button>
      <button
        onClick={() => {
          setFileOperation({ kind: 'file' });
          closeMenus();
        }}
      >
        Novo arquivo… <kbd>Ctrl+N</kbd>
      </button>
      <button
        disabled={!clipboard || !world?.vfs.nodes[clipboard.path]}
        onClick={() => {
          void pasteVfs(path, asRoot).catch((reason: unknown) => setError(String(reason)));
          closeMenus();
        }}
      >
        Colar <kbd>Ctrl+V</kbd>
      </button>
      <button
        disabled={importing}
        onClick={() => {
          importRef.current?.click();
          closeMenus();
        }}
      >
        {importing ? 'Importando…' : 'Importar arquivo…'}
      </button>
    </>
  );
  const header = (
    <div className="files-header" ref={headerRef}>
      <div className="files-sidebar-heading">
        <button
          aria-label="Buscar arquivos"
          title="Buscar arquivos"
          aria-pressed={searchOpen}
          onClick={toggleSearch}
        >
          <FilesIcon name="search" />
        </button>
        <span>Arquivos</span>
        <button
          aria-label="Menu de arquivos"
          title="Menu de arquivos"
          data-files-menu
          aria-expanded={menu === 'actions'}
          onClick={() => {
            setMenu(menu === 'actions' ? null : 'actions');
            setContext(null);
          }}
        >
          <FilesIcon name="more" />
        </button>
      </div>
      <div className="files-navigation">
        <button
          aria-label="Voltar"
          title="Voltar"
          disabled={history.index === 0}
          onClick={() => navigate(history.paths[history.index - 1], history.index - 1)}
        >
          <FilesIcon name="back" />
        </button>
        <button
          aria-label="Avançar"
          title="Avançar"
          disabled={history.index === history.paths.length - 1}
          onClick={() => navigate(history.paths[history.index + 1], history.index + 1)}
        >
          <FilesIcon name="forward" />
        </button>
        <form
          className="files-location"
          onSubmit={(event) => {
            event.preventDefault();
            navigate(address.trim() || '/');
            addressRef.current?.blur();
          }}
        >
          <FilesIcon name={inTrash ? 'trash' : path === '/home/kali' ? 'home' : 'projects'} />
          <input
            ref={addressRef}
            aria-label="Caminho"
            title={path}
            spellCheck={false}
            value={
              editingAddress
                ? address
                : inTrash
                  ? 'Lixeira'
                  : path === '/home/kali'
                    ? 'Início'
                    : path.split('/').pop() || 'Sistema'
            }
            onFocus={(event) => {
              setAddress(path);
              setEditingAddress(true);
              requestAnimationFrame(() => event.target.select());
            }}
            onBlur={() => setEditingAddress(false)}
            onChange={(event) => setAddress(event.target.value)}
          />
          <button
            type="button"
            aria-label="Pasta acima"
            title="Pasta acima"
            disabled={path === '/' || inTrash}
            onClick={() => navigate(path.slice(0, path.lastIndexOf('/')) || '/')}
          >
            <FilesIcon name="up" />
          </button>
        </form>
        <button
          aria-label="Atualizar pasta"
          title="Atualizar pasta"
          onClick={() => setRefresh((value) => value + 1)}
        >
          <FilesIcon name="refresh" />
        </button>
        <button
          aria-label={view === 'grid' ? 'Exibir em lista' : 'Exibir em grade'}
          title={view === 'grid' ? 'Exibir em lista' : 'Exibir em grade'}
          onClick={() => setView(view === 'grid' ? 'list' : 'grid')}
        >
          <FilesIcon name={view === 'grid' ? 'list' : 'grid'} />
        </button>
        <button
          aria-label="Opções de exibição"
          title="Opções de exibição"
          data-files-menu
          aria-expanded={menu === 'view'}
          onClick={() => {
            setMenu(menu === 'view' ? null : 'view');
            setContext(null);
          }}
        >
          <FilesIcon name="down" />
        </button>
      </div>
    </div>
  );
  return (
    <div
      ref={rootRef}
      className="file-manager"
      tabIndex={0}
      aria-label="Gerenciador de arquivos"
      style={{ position: 'relative' }}
      {...drop(inTrash ? { kind: 'trash', asRoot } : { kind: 'folder', path, asRoot })}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          closeMenus();
          if (searchOpen) {
            setSearchOpen(false);
            setSearch('');
          }
          return;
        }
        if (
          fileOperation ||
          compressPaths ||
          extractArchive ||
          context ||
          menu ||
          useGame.getState().sessionPending
        ) {
          return;
        }
        const action = fileKeyboardAction(event);
        if (!action) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        if (event.repeat && !['next', 'previous'].includes(action)) {
          return;
        }
        const selected = selectedPaths.filter((path) => nodes.some((node) => node.id === path));
        if (action === 'all') {
          setSelectedPaths(displayedNodes.map((node) => node.id));
        } else if (action === 'next' || action === 'previous') {
          const index = displayedNodes.findIndex((node) => node.id === selection?.id);
          const next =
            displayedNodes[
              Math.max(0, Math.min(displayedNodes.length - 1, index + (action === 'next' ? 1 : -1)))
            ];
          if (next) {
            setSelection(next);
            setSelectedPaths([next.id]);
            Array.from(
              event.currentTarget.querySelectorAll<HTMLElement>('.file-grid [data-vfs-path]'),
            )
              .find((element) => element.dataset.vfsPath === next.id)
              ?.focus();
          }
        } else if (!inTrash) {
          if (action === 'folder' || action === 'file') {
            setFileOperation({ kind: action });
          } else if (action === 'copy' || action === 'cut') {
            useVfsClipboard.getState().copy(selected, action === 'cut');
          } else if (action === 'paste') {
            void pasteVfs(path, asRoot).catch((reason: unknown) => setError(String(reason)));
          } else if (action === 'rename' && selected.length === 1) {
            setFileOperation({
              kind: 'rename',
              node: nodes.find((node) => node.id === selected[0])!,
            });
          } else if (action === 'trash' && selected.length) {
            void transferVfsItems(selected, { kind: 'trash', asRoot }).catch((reason: unknown) =>
              setError(String(reason)),
            );
          } else if (action === 'open' && selected.length === 1) {
            open(nodes.find((node) => node.id === selected[0])!);
          }
        }
      }}
    >
      {fileOperation && (
        <FileNameDialog
          operation={fileOperation}
          directory={path}
          asRoot={asRoot}
          onClose={() => setFileOperation(null)}
        />
      )}
      {compressPaths && (
        <CompressDialog
          paths={compressPaths}
          directory={path}
          asRoot={asRoot}
          close={() => setCompressPaths(null)}
        />
      )}
      {extractArchive && (
        <ExtractDialog
          archive={extractArchive}
          destination={extractDestination}
          asRoot={asRoot}
          close={() => setExtractArchive(null)}
        />
      )}
      {headerSlot ? createPortal(header, headerSlot) : header}
      {context && (
        <div
          className="archive-context files-popover"
          role="dialog"
          aria-label="Ações do item"
          style={{ left: context.x, top: context.y }}
        >
          {selection ? itemActions : folderActions}
          <button onClick={closeMenus}>Fechar menu</button>
        </div>
      )}
      {menu === 'actions' && (
        <div
          className="files-popover files-actions-menu"
          role="dialog"
          aria-label="Ações de arquivos"
        >
          {folderActions}
          {selection && (
            <>
              <hr />
              {itemActions}
            </>
          )}
          <button onClick={closeMenus}>Fechar menu</button>
        </div>
      )}
      {asRoot && (
        <div className="file-root-banner">
          Administrador root — somente arquivos do computador virtual
        </div>
      )}
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
      {menu === 'view' && (
        <div
          className="files-popover files-view-menu"
          role="dialog"
          aria-label="Opções de exibição de arquivos"
        >
          <strong>Exibição</strong>
          <label>
            <input
              type="checkbox"
              checked={showHidden}
              onChange={(e) => setShowHidden(e.target.checked)}
            />{' '}
            Ocultos
          </label>
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
          <button onClick={closeMenus}>Concluído</button>
        </div>
      )}
      <div className="file-layout">
        <aside aria-label="Locais">
          {(
            [
              ['/home/kali', 'Início', 'home'],
              ['/home/kali/Desktop', 'Área de trabalho', 'desktop'],
              ['/', 'Sistema', 'system'],
              [TRASH_FILES_PATH, 'Lixeira', 'trash'],
              ['/home/kali/Documents', 'Documentos', 'documents'],
              ['/home/kali/Music', 'Música', 'music'],
              ['/home/kali/Pictures', 'Imagens', 'pictures'],
              ['/home/kali/Videos', 'Vídeos', 'videos'],
              ['/home/kali/Downloads', 'Downloads', 'downloads'],
              ['/home/kali/projects', 'Projetos', 'projects'],
            ] as [string, string, FilesSymbol][]
          ).map(([location, label, icon], index) => (
            <button
              key={location}
              title={location}
              aria-label={label}
              aria-current={path === location ? 'page' : undefined}
              className={`${path === location ? 'selected' : ''} ${index === 4 ? 'files-sidebar-divider' : ''}`}
              onClick={() => navigate(location)}
              {...drop(
                location === TRASH_FILES_PATH
                  ? { kind: 'trash', asRoot }
                  : { kind: 'folder', path: location, asRoot },
              )}
            >
              <FilesIcon name={icon} />
              <span>{label}</span>
            </button>
          ))}
        </aside>
        <div className="files-main">
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
          {searchOpen && (
            <div className="files-search-row">
              <FilesIcon name="search" />
              <input
                ref={searchRef}
                aria-label="Buscar arquivos locais"
                placeholder="Buscar nesta pasta · *.zip"
                value={search}
                onChange={(event) => setSearch(event.target.value)}
              />
              <button aria-label="Fechar busca" onClick={toggleSearch}>
                <FilesIcon name="close" />
              </button>
            </div>
          )}
          <div
            className={`file-grid ${view === 'list' ? 'files-list' : ''}`}
            onContextMenu={(event) => {
              if (event.target !== event.currentTarget) {
                return;
              }
              event.preventDefault();
              const bounds = rootRef.current?.getBoundingClientRect();
              setSelection(null);
              setSelectedPaths([]);
              setMenu(null);
              setContext({
                x: Math.max(
                  0,
                  Math.min(event.clientX - (bounds?.left ?? 0), (bounds?.width ?? 400) - 270),
                ),
                y: Math.max(
                  0,
                  Math.min(event.clientY - (bounds?.top ?? 0), (bounds?.height ?? 400) - 200),
                ),
              });
            }}
            onClick={(event) => {
              if (event.target === event.currentTarget) {
                setSelection(null);
                setSelectedPaths([]);
                closeMenus();
              }
            }}
          >
            {displayedNodes.map((node) => (
              <button
                key={node.id}
                data-vfs-path={node.id}
                aria-pressed={selectedPaths.includes(node.id)}
                className={selectedPaths.includes(node.id) ? 'file selected' : 'file'}
                onClick={(e) => {
                  setSelection(node);
                  setContext(null);
                  setSelectedPaths(
                    e.ctrlKey || e.metaKey
                      ? selectedPaths.includes(node.id)
                        ? selectedPaths.filter((p) => p !== node.id)
                        : [...selectedPaths, node.id]
                      : [node.id],
                  );
                }}
                onContextMenu={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  setMenu(null);
                  const bounds = e.currentTarget.closest('.file-manager')?.getBoundingClientRect();
                  setSelection(node);
                  if (!selectedPaths.includes(node.id)) {
                    setSelectedPaths([node.id]);
                  }
                  setContext({
                    x: Math.max(
                      0,
                      Math.min(e.clientX - (bounds?.left ?? 0), (bounds?.width ?? 400) - 270),
                    ),
                    y: Math.max(
                      0,
                      Math.min(e.clientY - (bounds?.top ?? 0), (bounds?.height ?? 400) - 380),
                    ),
                  });
                }}
                onDoubleClick={() => open(node)}
                draggable={!inTrash}
                {...drop(dropTargetForNode(node, asRoot))}
                onDragStart={(e) => {
                  startVfsDrag(
                    e.dataTransfer,
                    selectedPaths.includes(node.id) ? selectedPaths : [node.id],
                    e.currentTarget,
                  );
                }}
              >
                {node.kind === 'directory' ? (
                  <FolderIcon name={node.name} />
                ) : associationForPath(node.id, node).kind === 'archive' ? (
                  <ArchiveIcon
                    size={view === 'grid' ? 64 : 28}
                    format={
                      node.blob?.mime === 'application/gzip'
                        ? 'GZIP'
                        : node.blob?.mime === 'application/x-bzip2'
                          ? 'BZIP2'
                          : node.blob?.mime === 'application/x-xz'
                            ? 'XZ'
                            : node.blob?.mime === 'application/x-tar'
                              ? 'TAR'
                              : node.name.toUpperCase().endsWith('GZ')
                                ? 'GZIP'
                                : node.name.toUpperCase().endsWith('BZ2')
                                  ? 'BZIP2'
                                  : node.name.toUpperCase().endsWith('XZ')
                                    ? 'XZ'
                                    : node.name.toUpperCase().endsWith('TAR')
                                      ? 'TAR'
                                      : 'ZIP'
                    }
                  />
                ) : (
                  <AppIcon
                    name={
                      node.metadata.app && node.metadata.app in apps
                        ? (node.metadata.app as AppId)
                        : associationForPath(node.id, node).application
                    }
                    size={view === 'grid' ? 64 : 28}
                  />
                )}
                <span
                  className="file-name"
                  title={node.kind === 'symlink' ? `Link simbólico → ${node.content}` : node.name}
                >
                  {node.kind === 'symlink' && <span aria-label="Link simbólico">↗ </span>}
                  {node.name}
                </span>
                {view === 'list' && (
                  <small className="files-type">
                    {node.kind === 'directory'
                      ? 'Pasta'
                      : node.kind === 'symlink'
                        ? 'Link simbólico'
                        : node.kind === 'charDevice'
                          ? 'Dispositivo virtual'
                          : associationForPath(node.id, node).label}
                  </small>
                )}
              </button>
            ))}
          </div>
          {displayedNodes.length === 0 && !error && (
            <p className="muted">
              {search
                ? 'Nenhum arquivo encontrado.'
                : inTrash
                  ? 'A lixeira está vazia.'
                  : 'Esta pasta está vazia.'}
            </p>
          )}
        </div>
      </div>
      {selection && showDetails && (
        <div className="file-details">
          <button
            className="files-details-close"
            aria-label="Fechar propriedades"
            onClick={() => setShowDetails(false)}
          >
            <FilesIcon name="close" />
          </button>
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
              <button
                onClick={() =>
                  setCompressPaths(selectedPaths.length ? selectedPaths : [selection.id])
                }
              >
                Compactar…
              </button>
              {associationForPath(selection.id, selection).kind === 'archive' && (
                <>
                  <button onClick={() => startExtract(selection, true)}>Extrair aqui</button>
                  <button onClick={() => startExtract(selection, false)}>Extrair para…</button>
                </>
              )}
              <button
                onClick={() =>
                  useVfsClipboard
                    .getState()
                    .copy(selectedPaths.length ? selectedPaths : selection.id)
                }
              >
                Copiar para área de transferência
              </button>
              <button
                onClick={() =>
                  useVfsClipboard
                    .getState()
                    .copy(selectedPaths.length ? selectedPaths : selection.id, true)
                }
              >
                Recortar
              </button>
              <button onClick={() => setFileOperation({ kind: 'rename', node: selection })}>
                Renomear · F2
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
      <footer className="app-status files-status">
        <span>
          {displayedNodes.length} itens
          {selectedPaths.length > 0 && ` · ${selectedPaths.length} selecionado(s)`}
        </span>
        <span title={path}>{path}</span>
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
