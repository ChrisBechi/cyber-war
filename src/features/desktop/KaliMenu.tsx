import { useEffect, useRef, useState } from 'react';
import { apps } from '../../lib/window-store';
import type { BuiltinAppId } from '../../lib/window-store';
import {
  categoryById,
  inCategory,
  launcherList,
  searchSoftware,
  softwareById,
  softwareCatalog,
} from '../../lib/software-catalog';
import type { SoftwareEntry } from '../../lib/software-catalog';
import { AppIcon } from './AppIcon';
import { KaliIcon } from './KaliIcon';
import { useDismissOutside } from '../../lib/use-dismiss-outside';

type Props = {
  settings: Record<string, string>;
  nickname: string;
  onLaunch: (entry: SoftwareEntry) => void;
  onFavorite: (id: string) => void;
  onBuiltin: (id: BuiltinAppId) => void;
  onClose: () => void;
  onLock: () => void;
  onLogout: () => void;
};
const gameApps: BuiltinAppId[] = [
  'messages',
  'forum',
  'missions',
  'codelab',
  'saves',
  'journey',
  'vigilia',
];

export function KaliMenu({
  settings,
  nickname,
  onLaunch,
  onFavorite,
  onBuiltin,
  onClose,
  onLock,
  onLogout,
}: Props) {
  const [query, setQuery] = useState('');
  const [category, setCategory] = useState('favorites');
  const container = useRef<HTMLElement>(null);
  const search = useRef<HTMLInputElement>(null);
  const favorites = launcherList(settings.launcherFavorites, softwareCatalog.favorites);
  const recent = launcherList(settings.launcherRecent);
  const special = [
    { id: 'favorites', name: 'Favorites', icon: 'folder-favorites' },
    { id: 'recent', name: 'Recently Used', icon: 'document-open-recent' },
    { id: 'all', name: 'All Applications', icon: 'applications-other' },
  ];
  const roots = [...special, ...softwareCatalog.categories.filter((item) => !item.parent)];
  const children = query
    ? []
    : softwareCatalog.categories.filter((item) => item.parent === category);
  const entries = query.trim()
    ? searchSoftware(query)
    : category === 'favorites' || category === 'recent'
      ? (category === 'favorites' ? favorites : recent).flatMap((id) => {
          const entry = softwareById.get(id);
          return entry ? [entry] : [];
        })
      : category === 'all'
        ? softwareCatalog.entries
        : softwareCatalog.entries.filter((entry) => entry.categories.includes(category));
  const current = categoryById.get(category);
  const isUsual = !query && (category === 'all' || category === 'kali-usual-applications');
  useDismissOutside(
    true,
    (target) =>
      !!container.current?.contains(target) ||
      (target instanceof Element && !!target.closest('[data-launcher-toggle]')),
    onClose,
  );
  useEffect(() => {
    search.current?.focus();
  }, []);
  const select = (id: string) => {
    setCategory(id);
    setQuery('');
  };
  const focusResult = (index: number) => {
    const buttons = container.current?.querySelectorAll<HTMLButtonElement>('[data-menu-result]');
    if (buttons?.length) {
      buttons[(index + buttons.length) % buttons.length].focus();
    }
  };
  return (
    <nav
      ref={container}
      className="kali-menu"
      aria-label="Aplicativos Kali"
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          event.stopPropagation();
          onClose();
        }
        if (event.key === 'ArrowLeft' && event.target !== search.current) {
          event.preventDefault();
          if (current?.parent) {
            select(current.parent);
          } else {
            container.current
              ?.querySelector<HTMLButtonElement>(`[data-category="${category}"]`)
              ?.focus();
          }
        }
      }}
    >
      <div className="kali-search">
        <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
          <circle cx="10" cy="10" r="6" />
          <path d="m15 15 5 5" />
        </svg>
        <input
          ref={search}
          aria-label="Pesquisar aplicativos"
          autoComplete="off"
          spellCheck={false}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'ArrowDown') {
              event.preventDefault();
              focusResult(0);
            }
            if (event.key === 'Enter' && entries[0]) {
              event.preventDefault();
              onLaunch(entries[0]);
            }
          }}
        />
        {query && (
          <button
            aria-label="Limpar pesquisa"
            onClick={() => {
              setQuery('');
              search.current?.focus();
            }}
          >
            ×
          </button>
        )}
      </div>
      <div className="kali-menu-columns">
        <div
          className="kali-categories"
          role="tablist"
          aria-label="Categorias de aplicativos"
          aria-orientation="vertical"
        >
          {roots.map((item, index) => (
            <button
              key={item.id}
              role="tab"
              aria-selected={!query && category === item.id}
              data-category={item.id}
              className={index === 3 ? 'category-divider' : ''}
              onPointerEnter={(event) => {
                if (event.pointerType === 'mouse' && !query) {
                  select(item.id);
                }
              }}
              onClick={() => select(item.id)}
              onKeyDown={(event) => {
                if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
                  event.preventDefault();
                  const next =
                    roots[
                      (index + (event.key === 'ArrowDown' ? 1 : -1) + roots.length) % roots.length
                    ];
                  select(next.id);
                  container.current
                    ?.querySelector<HTMLButtonElement>(`[data-category="${next.id}"]`)
                    ?.focus();
                }
                if (event.key === 'ArrowRight') {
                  event.preventDefault();
                  focusResult(0);
                }
              }}
            >
              <KaliIcon name={item.icon} size={22} />
              <span>{item.name}</span>
            </button>
          ))}
        </div>
        <div
          className="kali-applications"
          role="tabpanel"
          aria-label={
            query
              ? 'Resultados da pesquisa'
              : (current?.name ?? special.find((item) => item.id === category)?.name)
          }
          onKeyDown={(event) => {
            const buttons = Array.from(
              container.current?.querySelectorAll<HTMLButtonElement>('[data-menu-result]') ?? [],
            );
            const index = buttons.indexOf(event.target as HTMLButtonElement);
            if (index < 0) {
              return;
            }
            if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
              event.preventDefault();
              focusResult(
                event.key === 'Home'
                  ? 0
                  : event.key === 'End'
                    ? buttons.length - 1
                    : index + (event.key === 'ArrowDown' ? 1 : -1),
              );
            }
          }}
        >
          {query && (
            <div className="kali-result-count" role="status">
              {entries.length} resultado{entries.length === 1 ? '' : 's'}
            </div>
          )}
          {current?.parent && !query && (
            <button
              className="kali-category-back"
              data-menu-result
              onClick={() => select(current.parent ?? 'all')}
            >
              ‹ {current.name}
            </button>
          )}
          {children
            .filter((child) => softwareCatalog.entries.some((entry) => inCategory(entry, child.id)))
            .map((child) => (
              <button
                className="kali-app-row"
                data-menu-result
                key={child.id}
                onClick={() => select(child.id)}
              >
                <KaliIcon name={child.icon} size={28} />
                <span>{child.name}</span>
                <span className="submenu-arrow">›</span>
              </button>
            ))}
          {entries.map((entry) => (
            <div className="kali-app-entry" key={entry.id}>
              <button
                className="kali-app-row"
                data-menu-result
                title={`${entry.name} — ${entry.description}`}
                onClick={() => onLaunch(entry)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  onFavorite(entry.id);
                }}
              >
                <KaliIcon name={entry.icon} size={30} />
                <span>{entry.name}</span>
              </button>
              <button
                className="kali-favorite-toggle"
                aria-label={`${favorites.includes(entry.id) ? 'Remover' : 'Adicionar'} ${entry.name} ${favorites.includes(entry.id) ? 'dos' : 'aos'} favoritos`}
                aria-pressed={favorites.includes(entry.id)}
                title="Alternar favorito"
                onClick={() => onFavorite(entry.id)}
              >
                {favorites.includes(entry.id) ? '★' : '☆'}
              </button>
            </div>
          ))}
          {isUsual && (
            <div className="kali-game-apps">
              {gameApps.map((id) => (
                <button
                  className="kali-app-row"
                  data-menu-result
                  key={id}
                  onClick={() => onBuiltin(id)}
                >
                  <AppIcon name={id} size={30} />
                  <span>{apps[id].title}</span>
                </button>
              ))}
            </div>
          )}
          {!entries.length && !children.length && (
            <p className="kali-empty">
              {query
                ? 'Nenhum aplicativo encontrado.'
                : category === 'recent'
                  ? 'Os aplicativos abertos aparecem aqui.'
                  : 'Sem aplicativos nesta categoria da edição padrão.'}
            </p>
          )}
        </div>
      </div>
      <footer className="kali-menu-footer">
        <span>{nickname}</span>
        <button
          aria-label="Configurações"
          title="Configurações"
          onClick={() => onBuiltin('settings')}
        >
          <KaliIcon name="preferences-system" size={22} />
        </button>
        <button aria-label="Bloquear sessão" title="Bloquear sessão" onClick={onLock}>
          <KaliIcon name="system-lock-screen" size={22} />
        </button>
        <button
          aria-label="Encerrar sessão e sair"
          title="Encerrar sessão e sair"
          onClick={onLogout}
        >
          <KaliIcon name="system-log-out" size={22} />
        </button>
      </footer>
    </nav>
  );
}
