import { useCallback, useEffect, useState } from 'react';
import { act, useGame } from '../../lib/game-store';
import { apps, useWindows, windowApp } from '../../lib/window-store';
import type { AppId, WindowId } from '../../lib/window-store';
import { audioManager } from '../../lib/audio-manager';
import { endSession } from '../../lib/session';
import { isTrashPath, TRASH_FILES_PATH } from '../../lib/vfs-paths';
import { inCategory, softwareById, softwareCatalog } from '../../lib/software-catalog';
import type { SoftwareEntry } from '../../lib/software-catalog';
import { FileManager } from '../files/FileManager';
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
}: {
  onMenu: () => void;
  presentation?: boolean;
}) {
  const { world, missions, error, clearError, sessionPending } = useGame();
  const { windows, workspace, open } = useWindows();
  const [launcher, setLauncher] = useState(false);
  const [locked, setLocked] = useState(false);
  const [trashHover, setTrashHover] = useState(false);
  const [context, setContext] = useState<{ x: number; y: number; path?: string } | null>(null);
  const [creating, setCreating] = useState<DesktopItemKind | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const clipboard = useVfsClipboard((s) => s.entry);
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
  useEffect(() => {
    if (presentation) {
      return;
    }
    const key = (e: KeyboardEvent) => {
      if (locked || creating || sessionPending) {
        return;
      }
      if (e.key === 'Meta' || (e.ctrlKey && e.key === 'Escape')) {
        e.preventDefault();
        setLauncher((value) => !value);
        return;
      }
      if (e.ctrlKey && e.altKey && e.key.toLowerCase() === 't') {
        e.preventDefault();
        useWindows.getState().newTerminal();
      }
      if (e.key === 'Escape') {
        setLauncher(false);
      }
      if (e.altKey && e.key === 'Tab') {
        e.preventDefault();
        const state = useWindows.getState();
        const list = state.windows
          .filter((w) => w.workspace === state.workspace)
          .sort((a, b) => b.z - a.z);
        if (list.length > 1) {
          state.focus(list[1].id);
        }
      }
    };
    window.addEventListener('keydown', key);
    return () => window.removeEventListener('keydown', key);
  }, [open, locked, presentation, creating, sessionPending]);
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
            action: () => useVfsClipboard.getState().copy(context.path!),
          },
          {
            label: 'Recortar',
            icon: 'file' as const,
            action: () => useVfsClipboard.getState().copy(context.path!, true),
          },
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
            initialCommand={path ? scriptCommand(path) : undefined}
          />
        );
      case 'files':
        return <FileManager initialPath={path} asRoot={model?.asRoot} />;
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
        return <Settings />;
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
        inert={locked || sessionPending}
        className={`desktop wallpaper-${world.settings.wallpaper ?? 'waves'}`}
        data-testid="desktop"
        tabIndex={-1}
        onContextMenu={(event) => {
          const target = event.target as HTMLElement;
          if (
            presentation ||
            creating ||
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
          setContext({ x: event.clientX, y: event.clientY, path });
        }}
        onKeyDown={(event) => {
          const target = event.target as HTMLElement;
          if (
            presentation ||
            creating ||
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
          if (event.ctrlKey && ['c', 'x', 'v'].includes(event.key.toLowerCase())) {
            event.preventDefault();
            if (event.key.toLowerCase() === 'v') {
              paste();
            } else if (selected) {
              useVfsClipboard.getState().copy(selected, event.key.toLowerCase() === 'x');
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
              className={selected === node.id ? 'desktop-icon-selected' : undefined}
              onClick={() => setSelected(node.id)}
              title={node.name.replace(/\.desktop$/, '')}
              onDoubleClick={() => openNode(node)}
              draggable
              onDragStart={(e) => {
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.setData('application/x-cyber-war-vfs', node.id);
                e.dataTransfer.setData('text/plain', node.id);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  openNode(node);
                }
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
            className={`desktop-trash${trashHasItems ? ' has-items' : ''}${trashHover ? ' is-over' : ''}`}
            title="Lixeira"
            aria-label="Lixeira"
            onClick={() => open('files', TRASH_FILES_PATH)}
            onDragOver={(e) => {
              const source = e.dataTransfer.types.includes('application/x-cyber-war-vfs');
              if (!source) {
                return;
              }
              e.preventDefault();
              e.dataTransfer.dropEffect = 'move';
              setTrashHover(true);
            }}
            onDragLeave={() => setTrashHover(false)}
            onDrop={(e) => {
              e.preventDefault();
              setTrashHover(false);
              const source =
                e.dataTransfer.getData('application/x-cyber-war-vfs') ||
                e.dataTransfer.getData('text/plain');
              if (source && !isTrashPath(source)) {
                act('vfs_remove', { path: source, recursive: true });
              }
            }}
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
      </main>
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
