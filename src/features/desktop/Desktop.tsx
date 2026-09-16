import { useCallback, useEffect, useState } from 'react';
import { act, useGame } from '../../lib/game-store';
import { apps, useWindows, windowApp } from '../../lib/window-store';
import type { AppId, WindowId } from '../../lib/window-store';
import { audioManager } from '../../lib/audio-manager';
import { endSession } from '../../lib/session';
import { TRASH_FILES_PATH } from '../../lib/vfs-paths';
import { inCategory, softwareById, softwareCatalog } from '../../lib/software-catalog';
import type { SoftwareEntry } from '../../lib/software-catalog';
import { FileManager } from '../files/FileManager';
import { ArchiveViewer } from '../files/ArchiveViewer';
import { PackageInstaller } from '../files/PackageInstaller';
import { pollArchiveJobs } from '../../lib/archive';
import { desktopRuntime } from '../../lib/api';
import { Editor } from '../files/Editor';
import { ImageViewer } from '../media/ImageViewer';
import { MediaPlayer } from '../media/MediaPlayer';
import { TerminalWindow } from '../terminal/TerminalWindow';
import { AppWindow } from './AppWindow';
import { AppIcon } from './AppIcon';
import { KaliPanel } from './KaliPanel';
import { KaliMenu } from './KaliMenu';
import { ToolWorkspace } from './ToolWorkspace';
import { Calculator } from './Calculator';
import { DesktopContextMenu } from './DesktopContextMenu';
import type { ContextItem } from './DesktopContextMenu';
import { DesktopCreateDialog } from './DesktopCreateDialog';
import { DesktopKeyboardHelp } from './DesktopKeyboardHelp';
import { useDesktopKeyboard } from './use-desktop-keyboard';
import { dropTargetForNode, startVfsDrag, transferVfsItems } from '../../lib/vfs-drag';
import { useVfsDrop } from '../../lib/use-vfs-drop';
import { fileKeyboardAction } from '../../lib/file-keyboard';
import { FileNameDialog, type FileNameOperation } from '../files/FileNameDialog';
import type { DesktopItemKind } from './DesktopCreateDialog';
import { pasteVfs, useVfsClipboard } from '../../lib/vfs-clipboard';
import { sortVfsNodes } from '../../lib/vfs-order';
import type { VfsSort } from '../../lib/vfs-order';
import { launchSoftware, openVfsNode, scriptCommand } from '../../lib/file-open';
import {
  Browser,
  CodeLab,
  Forum,
  Journey,
  Messages,
  Missions,
  Processes,
  Saves,
  Settings,
} from './Applications';
import { VigiliaGame } from './VigiliaGame';

export function Desktop({
  onMenu,
  presentation = false,
  inputBlocked = false,
}: {
  onMenu: () => void;
  presentation?: boolean;
  inputBlocked?: boolean;
}) {
  const { world, missions, error, clearError, sessionPending } = useGame();
  const { windows, workspace, open } = useWindows();
  const [launcher, setLauncher] = useState(false);
  const [locked, setLocked] = useState(false);
  const [keyboardHelp, setKeyboardHelp] = useState(false);
  const drop = useVfsDrop();
  const [context, setContext] = useState<{ x: number; y: number; path?: string } | null>(null);
  const [creating, setCreating] = useState<DesktopItemKind | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [fileOperation, setFileOperation] = useState<FileNameOperation | null>(null);
  const clipboard = useVfsClipboard((s) => s.entry);
  useEffect(() => {
    if (!desktopRuntime || presentation) {
      return;
    }
    let running = false;
    let completed = '';
    const timer = setInterval(() => {
      if (running) {
        return;
      }
      running = true;
      void pollArchiveJobs()
        .then((jobs) => {
          const next = jobs
            .filter((j) => j.status !== 'Running')
            .map((j) => `${j.id}:${j.status}`)
            .join(',');
          if (next !== completed) {
            completed = next;
            void useGame.getState().refresh();
          }
        })
        .catch(() => undefined)
        .finally(() => {
          running = false;
        });
    }, 500);
    return () => clearInterval(timer);
  }, [presentation]);
  const closeContext = useCallback(() => setContext(null), []);
  const closeLauncher = useCallback(() => setLauncher(false), []);
  const lock = () => {
    setLauncher(false);
    setContext(null);
    setLocked(true);
  };
  const logout = () => {
    void endSession(onMenu).catch(() => undefined);
  };
  const launch = (entry: SoftwareEntry) => {
    setLauncher(false);
    void launchSoftware(entry).catch(() => undefined);
  };
  useEffect(() => {
    if (presentation) {
      return;
    }
    const save = window.setInterval(() => act('autosave'), 120000);
    return () => {
      window.clearInterval(save);
    };
  }, [presentation]);
  useEffect(
    () =>
      useWindows.subscribe((next, previous) => {
        if (next.windows.length !== previous.windows.length) {
          audioManager.play('ui-window');
        } else if (
          next.windows.some(
            (item, i) =>
              item.maximized !== previous.windows[i]?.maximized ||
              item.minimized !== previous.windows[i]?.minimized,
          )
        ) {
          audioManager.play('ui-window');
        }
      }),
    [],
  );
  useEffect(
    () =>
      useGame.subscribe((next, previous) => {
        if ((next.world?.messages.length ?? 0) > (previous.world?.messages.length ?? 0)) {
          audioManager.play('ui-notification');
        }
      }),
    [],
  );
  useDesktopKeyboard({
    disabled:
      inputBlocked ||
      locked ||
      !!creating ||
      !!fileOperation ||
      sessionPending ||
      keyboardHelp ||
      !world,
    presentation,
    onMenu: () => {
      setContext(null);
      setLauncher((value) => !value);
    },
    onDismiss: () => {
      setLauncher(false);
      setContext(null);
    },
    onHelp: () => setKeyboardHelp(true),
    onLock: lock,
  });
  if (!world) {
    return null;
  }
  const unread = world.messages.filter((m) => !m.read).length;
  const active = missions.filter((m) => m.status === 'active');
  const desktopPath = '/home/kali/Desktop';
  const shortcuts = Object.values(world.vfs.nodes).filter(
    (node) => node.parentId === desktopPath && !node.name.startsWith('.'),
  );
  const displayedShortcuts = world.settings.desktopSort
    ? sortVfsNodes(shortcuts, world.settings.desktopSort as VfsSort)
    : shortcuts;
  const trashHasItems = Object.values(world.vfs.nodes).some(
    (node) => node.parentId === TRASH_FILES_PATH,
  );
  const openNode = (node: (typeof shortcuts)[number]) => {
    void openVfsNode(node).catch((reason: unknown) => useGame.setState({ error: String(reason) }));
  };
  const paste = () => {
    void pasteVfs(desktopPath).catch(() => undefined);
  };
  const selectedItems = selectedPaths.filter((path) => !!world.vfs.nodes[path]);
  const trashSelection = () => {
    void transferVfsItems(selectedItems, { kind: 'trash' }).catch((error: unknown) =>
      useGame.setState({ error: String(error) }),
    );
  };
  const menuItems: ContextItem[] = [
    ...(context?.path
      ? [
          {
            label: 'Abrir',
            icon: 'files' as const,
            action: () => {
              const node = world.vfs.nodes[context.path!];
              if (node) {
                openNode(node);
              }
            },
          },
          {
            label: 'Copiar',
            icon: 'file' as const,
            action: () =>
              useVfsClipboard
                .getState()
                .copy(selectedItems.includes(context.path!) ? selectedItems : context.path!),
          },
          {
            label: 'Recortar',
            icon: 'file' as const,
            action: () =>
              useVfsClipboard
                .getState()
                .copy(selectedItems.includes(context.path!) ? selectedItems : context.path!, true),
          },
          {
            label: 'Renomear…',
            icon: 'editor' as const,
            action: () =>
              setFileOperation({ kind: 'rename', node: world.vfs.nodes[context.path!] }),
          },
          { label: 'Enviar à lixeira', icon: 'trash' as const, action: trashSelection },
          null,
        ]
      : []),
    { label: 'Criar lançador…', icon: 'settings', action: () => setCreating('launcher') },
    { label: 'Criar link de URL…', icon: 'browser', action: () => setCreating('url') },
    { label: 'Criar pasta…', icon: 'folder', action: () => setCreating('folder') },
    {
      label: 'Criar documento',
      icon: 'file',
      children: [
        { label: 'Arquivo vazio', icon: 'file', action: () => setCreating('file') },
        { label: 'Script Shell (.sh)', icon: 'terminal', action: () => setCreating('shell') },
      ],
    },
    {
      label: 'Colar',
      icon: 'editor',
      disabled: !clipboard || !world.vfs.nodes[clipboard.path],
      action: paste,
    },
    null,
    {
      label: 'Abrir terminal aqui',
      icon: 'terminal',
      action: () => useWindows.getState().newTerminal({ cwd: desktopPath }),
    },
    {
      label: 'Abrir como root',
      icon: 'folder',
      action: () => useWindows.getState().newInstance('files', desktopPath, true),
    },
    {
      label: 'Abrir em nova janela',
      icon: 'files',
      action: () => useWindows.getState().newInstance('files', desktopPath),
    },
    {
      label: 'Organizar ícones da área de trabalho',
      icon: 'files',
      action: () => act('setting_update', { key: 'desktopSort', value: 'name' }),
    },
    null,
    {
      label: 'Configurações da área de trabalho…',
      icon: 'settings',
      action: () => open('settings'),
    },
    null,
    {
      label: 'Aplicativos',
      icon: 'settings',
      children: [
        ...(['messages', 'forum', 'missions', 'saves'] as const).map((id) => ({
          label: apps[id].title,
          icon: id,
          action: () => open(id),
        })),
        null,
        ...softwareCatalog.categories
          .filter((category) => !category.parent)
          .map((category) => ({
            label: category.name,
            icon: 'folder' as const,
            children: softwareCatalog.entries
              .filter((entry) => inCategory(entry, category.id))
              .map((entry) => ({
                label: entry.name,
                icon: `tool:${entry.id}` as const,
                action: () => launch(entry),
              })),
          })),
      ],
    },
  ];
  const renderApp = (id: WindowId, path?: string) => {
    const model = windows.find((window) => window.id === id);
    switch (windowApp(id)) {
      case 'terminal':
        return (
          <TerminalWindow
            sessionId={id}
            initialCwd={model?.cwd}
            asRoot={model?.asRoot}
            initialCommand={model?.initialCommand ?? (path ? scriptCommand(path) : undefined)}
          />
        );
      case 'files':
        return <FileManager initialPath={path} asRoot={model?.asRoot} />;
      case 'archive-viewer':
        return <ArchiveViewer initialPath={path} asRoot={model?.asRoot} />;
      case 'package-installer':
        return <PackageInstaller initialPath={path} />;
      case 'editor':
        return <Editor initialPath={path} asRoot={model?.asRoot} />;
      case 'media-player':
        return <MediaPlayer initialPath={path} />;
      case 'image-viewer':
        return <ImageViewer initialPath={path} />;
      case 'browser':
        return <Browser initialAddress={path} navigationId={model?.navigationId} />;
      case 'tor-browser':
        return <Browser torOnly initialAddress={path} navigationId={model?.navigationId} />;
      case 'messages':
        return <Messages />;
      case 'forum':
        return <Forum />;
      case 'missions':
        return <Missions />;
      case 'codelab':
        return <CodeLab />;
      case 'settings':
        return <Settings onShowKeyboardHelp={() => setKeyboardHelp(true)} />;
      case 'saves':
        return <Saves onMenu={onMenu} />;
      case 'processes':
        return <Processes />;
      case 'journey':
        return <Journey />;
      case 'vigilia':
        return <VigiliaGame />;
      default: {
        const entry = softwareById.get(id.slice(5));
        if (entry?.builtin === 'calculator') {
          return <Calculator />;
        }
        return entry ? <ToolWorkspace entry={entry} initialPath={path} /> : null;
      }
    }
  };
  return (
    <>
      <main
        inert={inputBlocked || locked || sessionPending || keyboardHelp}
        className={`desktop wallpaper-${world.settings.wallpaper ?? 'waves'}`}
        data-testid="desktop"
        {...drop({ kind: 'folder', path: desktopPath })}
        tabIndex={-1}
        onContextMenu={(event) => {
          const target = event.target as HTMLElement;
          if (
            presentation ||
            creating ||
            fileOperation ||
            target.closest(
              '.window, .top-panel, .kali-menu, .desktop-note, .desktop-create-backdrop, .desktop-context-menu',
            )
          ) {
            return;
          }
          if (target.closest('button') && !target.closest('[data-vfs-path]')) {
            return;
          }
          event.preventDefault();
          setLauncher(false);
          const path = target.closest<HTMLElement>('[data-vfs-path]')?.dataset.vfsPath;
          setSelected(path ?? null);
          if (!path || !selectedPaths.includes(path)) {
            setSelectedPaths(path ? [path] : []);
          }
          setContext({ x: event.clientX, y: event.clientY, path });
        }}
        onKeyDown={(event) => {
          const target = event.target as HTMLElement;
          if (
            presentation ||
            creating ||
            fileOperation ||
            locked ||
            sessionPending ||
            keyboardHelp ||
            target.closest(
              '.window, .top-panel, .kali-menu, .desktop-context-menu, .desktop-create-backdrop',
            )
          ) {
            return;
          }
          if (event.key === 'ContextMenu' || (event.shiftKey && event.key === 'F10')) {
            event.preventDefault();
            const rect = target.getBoundingClientRect();
            setContext({
              x: rect.left + 20,
              y: rect.top + 36,
              path: target.closest<HTMLElement>('[data-vfs-path]')?.dataset.vfsPath,
            });
          }
          const action = fileKeyboardAction(event);
          if (action) {
            event.preventDefault();
            event.stopPropagation();
            if (event.repeat && !['next', 'previous'].includes(action)) {
              return;
            }
            if (action === 'paste') {
              paste();
            } else if (action === 'folder' || action === 'file') {
              setFileOperation({ kind: action });
            } else if (action === 'all') {
              setSelectedPaths(displayedShortcuts.map((item) => item.id));
            } else if (action === 'copy' || action === 'cut') {
              useVfsClipboard.getState().copy(selectedItems, action === 'cut');
            } else if (action === 'trash' && selectedItems.length) {
              trashSelection();
            } else if (action === 'rename' && selectedItems.length === 1) {
              setFileOperation({ kind: 'rename', node: world.vfs.nodes[selectedItems[0]] });
            } else if (action === 'open' && selectedItems.length === 1) {
              openNode(world.vfs.nodes[selectedItems[0]]);
            } else if (action === 'next' || action === 'previous') {
              const index = displayedShortcuts.findIndex((item) => item.id === selected);
              const next =
                displayedShortcuts[
                  Math.max(
                    0,
                    Math.min(displayedShortcuts.length - 1, index + (action === 'next' ? 1 : -1)),
                  )
                ];
              if (next) {
                setSelected(next.id);
                setSelectedPaths([next.id]);
                Array.from(
                  event.currentTarget.querySelectorAll<HTMLElement>(
                    '.desktop-shortcuts [data-vfs-path]',
                  ),
                )
                  .find((item) => item.dataset.vfsPath === next.id)
                  ?.focus();
              }
            }
          }
        }}
      >
        <KaliPanel
          launcher={launcher}
          onToggle={() => setLauncher((value) => !value)}
          onLaunch={launch}
          onLock={lock}
          onLogout={logout}
        />
        {launcher && (
          <KaliMenu
            settings={world.settings}
            nickname={world.nickname}
            onLaunch={launch}
            onFavorite={(id) => act('launcher_favorite', { id })}
            onBuiltin={(id) => {
              open(id);
              setLauncher(false);
            }}
            onClose={closeLauncher}
            onLock={lock}
            onLogout={logout}
          />
        )}
        <div className="desktop-shortcuts">
          {displayedShortcuts.map((node) => (
            <button
              key={node.id}
              data-app={node.metadata.app}
              data-vfs-path={node.id}
              className={selectedPaths.includes(node.id) ? 'desktop-icon-selected' : undefined}
              aria-pressed={selectedPaths.includes(node.id)}
              onClick={(event) => {
                setSelected(node.id);
                setSelectedPaths(
                  event.ctrlKey
                    ? selectedPaths.includes(node.id)
                      ? selectedPaths.filter((path) => path !== node.id)
                      : [...selectedPaths, node.id]
                    : [node.id],
                );
              }}
              title={node.name.replace(/\.desktop$/, '')}
              onDoubleClick={() => openNode(node)}
              draggable
              {...drop(dropTargetForNode(node))}
              onDragStart={(e) => {
                startVfsDrag(
                  e.dataTransfer,
                  selectedPaths.includes(node.id) ? selectedItems : [node.id],
                  e.currentTarget,
                );
              }}
            >
              <AppIcon
                name={
                  node.metadata.launcherId && softwareById.has(node.metadata.launcherId)
                    ? `tool:${node.metadata.launcherId}`
                    : node.metadata.url
                      ? 'browser'
                      : node.metadata.app && node.metadata.app in apps
                        ? (node.metadata.app as AppId)
                        : node.kind === 'directory'
                          ? 'folder'
                          : 'file'
                }
                size={40}
              />
              <span className="desktop-shortcut-name">{node.name.replace(/\.desktop$/, '')}</span>
            </button>
          ))}
          <button
            className={`desktop-trash${trashHasItems ? ' has-items' : ''}`}
            title="Lixeira"
            aria-label="Lixeira"
            onClick={() => open('files', TRASH_FILES_PATH)}
            {...drop({ kind: 'trash' })}
          >
            <AppIcon name="trash" size={40} trashFull={trashHasItems} />
            <span className="desktop-shortcut-name">Lixeira</span>
          </button>
        </div>
        <aside className="desktop-note">
          <span className="eyebrow">SESSÃO {world.session.toString().padStart(2, '0')}</span>
          <h2>
            {world.flags.includes('SESSION_1_COMPLETE')
              ? 'O primeiro vestígio.'
              : 'Tudo começa pequeno.'}
          </h2>
          <p>{active[0]?.objective || 'Um novo contato pode ter o próximo trabalho.'}</p>
          <button onClick={() => open('missions')}>
            Ver trabalhos <span>↗</span>
          </button>
          <div>
            <span>R$ {world.money}</span>
            <span>REP {world.reputation}</span>
          </div>
        </aside>
        <div className="window-layer">
          {windows.map((w) => (
            <AppWindow key={w.id} model={w}>
              {renderApp(w.id, w.path)}
            </AppWindow>
          ))}
        </div>
        {unread > 0 &&
          !windows.some(
            (w) => w.id === 'messages' && !w.minimized && w.workspace === workspace,
          ) && (
            <button className="notification" onClick={() => open('messages')}>
              <AppIcon name="messages" size={32} />
              <span>
                <b>{world.messages.filter((m) => !m.read).at(-1)?.contact}</b>
                <small>{world.messages.filter((m) => !m.read).at(-1)?.text}</small>
              </span>
            </button>
          )}
        {context && (
          <DesktopContextMenu
            key={`${context.x}:${context.y}:${context.path ?? ''}`}
            items={menuItems}
            x={context.x}
            y={context.y}
            onClose={closeContext}
          />
        )}
        {creating && <DesktopCreateDialog kind={creating} onClose={() => setCreating(null)} />}
        {fileOperation && (
          <FileNameDialog
            operation={fileOperation}
            directory={desktopPath}
            onClose={() => setFileOperation(null)}
          />
        )}
      </main>
      {keyboardHelp && <DesktopKeyboardHelp onClose={() => setKeyboardHelp(false)} />}
      {locked && (
        <section
          className="session-lock"
          role="dialog"
          aria-modal="true"
          aria-label="Sessão bloqueada"
        >
          <div>
            <AppIcon name="settings" size={64} />
            <h1>{world.nickname}</h1>
            <p>Sessão bloqueada</p>
            <button autoFocus className="primary" onClick={() => setLocked(false)}>
              Desbloquear
            </button>
            <button disabled={sessionPending} onClick={logout}>
              Encerrar sessão
            </button>
            {error && <p role="alert">{error}</p>}
          </div>
        </section>
      )}
      {error && !locked && (
        <div className="error-toast" role="alert">
          <span>{error}</span>
          <button onClick={clearError} aria-label="Fechar erro">
            ×
          </button>
        </div>
      )}
    </>
  );
}
