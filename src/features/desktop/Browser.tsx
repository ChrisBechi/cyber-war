import { useEffect, useRef, useState } from 'react';
import { emptySchema } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { useDismissOutside } from '../../lib/use-dismiss-outside';
import { ChromeIcon } from './ChromeIcon';
import { KaliIcon } from './KaliIcon';
import { BrowserContent } from './BrowserContent';
import { addressKey, canonicalAddress, isOnionAddress, readPreferences } from './browser-model';
import type { BrowserPreferences } from './browser-model';
import { useBrowserTabs } from './use-browser-tabs';
import './browser-chrome.css';

type Panel =
  | ''
  | 'menu'
  | 'bookmarks'
  | 'history'
  | 'settings'
  | 'tabs'
  | 'security'
  | 'extensions'
  | 'downloads';
export function Browser({
  initialAddress,
  navigationId,
  torOnly = false,
}: {
  initialAddress?: string;
  navigationId?: number;
  torOnly?: boolean;
}) {
  const browser = useBrowserTabs(initialAddress, navigationId, torOnly);
  const { tab, tabs, navigate, select, close, add, change } = browser;
  const rawPreferences = useGame((state) => state.world?.settings.browserPreferences);
  const files = useGame((state) => state.world?.vfs.nodes);
  const [preferences, setPreferences] = useState(() => readPreferences(rawPreferences));
  const [saving, setSaving] = useState(false);
  const savingRef = useRef(false);
  const [preferenceError, setPreferenceError] = useState('');
  const [actionPending, setActionPending] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [panel, setPanel] = useState<Panel>('');
  const [suggesting, setSuggesting] = useState(false);
  const [bookmarkDraft, setBookmarkDraft] = useState<{
    index: number;
    title: string;
    address: string;
  } | null>(null);
  const [homeDraft, setHomeDraft] = useState(preferences.home);
  const root = useRef<HTMLDivElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const addressInput = useRef<HTMLInputElement>(null);
  const addressBox = useRef<HTMLDivElement>(null);
  const mounted = useRef(true);
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null);
  useEffect(() => {
    setPreferences(readPreferences(rawPreferences));
  }, [rawPreferences]);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useDismissOutside(
    !!panel,
    (target) =>
      !!popup.current?.contains(target) ||
      !!root.current?.querySelector(`[data-browser-popup="${panel}"]`)?.contains(target),
    () => setPanel(''),
  );
  useDismissOutside(
    suggesting,
    (target) => !!addressBox.current?.contains(target),
    () => setSuggesting(false),
  );

  const savePreferences = async (next: BrowserPreferences) => {
    if (savingRef.current) {
      return false;
    }
    const canonical = {
      ...next,
      home: canonicalAddress(next.home),
      bookmarks: next.bookmarks.map((bookmark) => ({
        ...bookmark,
        address: canonicalAddress(bookmark.address),
      })),
    };
    savingRef.current = true;
    setSaving(true);
    setPreferenceError('');
    try {
      await perform('browser_preferences_save', { preferences: canonical }, emptySchema);
      if (mounted.current) {
        setPreferences(canonical);
      }
      return true;
    } catch (error) {
      if (mounted.current) {
        setPreferenceError(String(error));
      }
      return false;
    } finally {
      savingRef.current = false;
      if (mounted.current) {
        setSaving(false);
      }
    }
  };
  const openPanel = (value: Panel) => {
    setSuggesting(false);
    setPreferenceError('');
    setBookmarkDraft(null);
    if (value === 'settings') {
      setHomeDraft(preferences.home);
    }
    setPanel((old) => (old === value ? '' : value));
  };
  const go = (address: string, tabId = tab.id, index?: number) => {
    setPanel('');
    setSuggesting(false);
    navigate(address, tabId, index);
  };
  const bookmarkIndex = preferences.bookmarks.findIndex(
    (item) => addressKey(item.address) === addressKey(tab.address),
  );
  const star = () => {
    if (!tab.address || !tab.page) {
      return;
    }
    if (bookmarkIndex >= 0) {
      setBookmarkDraft({ index: bookmarkIndex, ...preferences.bookmarks[bookmarkIndex] });
      setPanel('bookmarks');
    } else {
      setBookmarkDraft({ index: -1, title: tab.page.title.slice(0, 100), address: tab.address });
      setPanel('bookmarks');
    }
  };
  const addTab = () => {
    setPanel('');
    setSuggesting(false);
    add();
    addressInput.current?.focus();
  };
  const action = async () => {
    if (!tab.page?.action || actionPending) {
      return;
    }
    const captured = tab;
    setActionPending(true);
    try {
      await perform(
        'browser_action',
        { action: captured.page!.action, value: captured.value },
        emptySchema,
      );
      // An action started in one tab must not replace a later navigation or another tab.
      if (mounted.current) {
        let reload = false;
        change((items) => {
          reload = items.some(
            (item) => item.id === captured.id && item.request === captured.request,
          );
          return items;
        });
        if (reload) {
          navigate(captured.address, captured.id, captured.index);
        }
      }
    } catch (error) {
      if (mounted.current) {
        change((items) =>
          items.map((item) =>
            item.id === captured.id && item.request === captured.request
              ? { ...item, error: String(error) }
              : item,
          ),
        );
      }
    } finally {
      if (mounted.current) {
        setActionPending(false);
      }
    }
  };
  const suggestions = [
    ...preferences.bookmarks,
    ...browser.history.map((item) => ({ title: item.title, address: item.address })),
  ]
    .filter(
      (item, index, items) =>
        items.findIndex((other) => addressKey(other.address) === addressKey(item.address)) ===
          index &&
        (item.address + ' ' + item.title).toLowerCase().includes(tab.draft.toLowerCase()),
    )
    .slice(0, 7);
  const downloads = Object.values(files ?? {}).filter(
    (file) => file.parentId === '/home/kali/Downloads',
  );
  const torAddress = isOnionAddress(tab.address);
  const torMode = torOnly || torAddress;
  const secureAddress = /^https:\/\//i.test(tab.address);
  return (
    <div
      className={`browser browser-shell${torMode ? ' tor-browser' : ''}`}
      ref={root}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          setPanel('');
          setSuggesting(false);
        }
        if (event.ctrlKey && event.key.toLowerCase() === 'l') {
          event.preventDefault();
          addressInput.current?.focus();
          addressInput.current?.select();
        }
        if (event.ctrlKey && event.key.toLowerCase() === 't') {
          event.preventDefault();
          addTab();
        }
        if (event.ctrlKey && event.key.toLowerCase() === 'w') {
          event.preventDefault();
          close(tab.id);
        }
        if (event.ctrlKey && event.key.toLowerCase() === 'd') {
          event.preventDefault();
          star();
        }
        if (event.altKey && event.key === 'ArrowLeft' && tab.index > 0) {
          event.preventDefault();
          go(tab.history[tab.index - 1], tab.id, tab.index - 1);
        }
        if (event.altKey && event.key === 'ArrowRight' && tab.index < tab.history.length - 1) {
          event.preventDefault();
          go(tab.history[tab.index + 1], tab.id, tab.index + 1);
        }
      }}
    >
      <div
        className={`browser-tabstrip${dragging ? ' is-dragging' : ''}`}
        onDoubleClick={(event) => {
          if (!(event.target as HTMLElement).closest('button')) {
            const model = useWindows.getState().windows.find((item) => item.id === 'browser');
            if (model) {
              useWindows.getState().update('browser', { maximized: !model.maximized });
            }
          }
        }}
        onPointerDown={(event) => {
          const model = useWindows.getState().windows.find((item) => item.id === 'browser');
          if (
            event.button !== 0 ||
            !model ||
            model.maximized ||
            (event.target as HTMLElement).closest('button')
          ) {
            return;
          }
          event.currentTarget.setPointerCapture(event.pointerId);
          setDragging(true);
          drag.current = { x: event.clientX, y: event.clientY, left: model.x, top: model.y };
        }}
        onPointerMove={(event) => {
          if (drag.current) {
            useWindows.getState().update('browser', {
              x: Math.max(
                0,
                Math.min(
                  window.innerWidth - 160,
                  drag.current.left + event.clientX - drag.current.x,
                ),
              ),
              y: Math.max(
                0,
                Math.min(
                  window.innerHeight - 120,
                  drag.current.top + event.clientY - drag.current.y,
                ),
              ),
            });
          }
        }}
        onPointerUp={() => {
          drag.current = null;
          setDragging(false);
        }}
        onPointerCancel={() => {
          drag.current = null;
          setDragging(false);
        }}
        onLostPointerCapture={() => {
          drag.current = null;
          setDragging(false);
        }}
      >
        <button
          className="browser-icon-button"
          aria-label="Listar abas"
          title="Listar abas"
          data-browser-popup="tabs"
          aria-expanded={panel === 'tabs'}
          onClick={() => openPanel('tabs')}
        >
          <ChromeIcon name="down" />
        </button>
        <div className="browser-tabs" role="tablist" aria-label="Abas do navegador">
          {tabs.map((item) => (
            <div
              className={item.id === tab.id ? 'browser-tab is-active' : 'browser-tab'}
              key={item.id}
            >
              <button
                role="tab"
                id={`browser-tab-${item.id}`}
                aria-selected={item.id === tab.id}
                aria-controls="browser-current-page"
                tabIndex={item.id === tab.id ? 0 : -1}
                title={item.page?.title ?? item.address ?? 'Nova aba'}
                onClick={() => {
                  select(item.id);
                  setPanel('');
                  setSuggesting(false);
                }}
                onKeyDown={(event) => {
                  if (['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
                    event.preventDefault();
                    const index = tabs.findIndex((candidate) => candidate.id === item.id);
                    const next =
                      event.key === 'Home'
                        ? 0
                        : event.key === 'End'
                          ? tabs.length - 1
                          : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) %
                            tabs.length;
                    select(tabs[next].id);
                    document.getElementById(`browser-tab-${tabs[next].id}`)?.focus();
                  }
                }}
              >
                <KaliIcon name="firefox" size={14} />
                <span>
                  {item.loading ? 'Carregando…' : item.page?.title || item.address || 'Nova aba'}
                </span>
              </button>
              <button
                className="browser-tab-close"
                aria-label={`Fechar aba ${item.page?.title || item.address || 'Nova aba'}`}
                onClick={() => close(item.id)}
              >
                <ChromeIcon name="close" />
              </button>
            </div>
          ))}
        </div>
        <button
          className="browser-icon-button"
          aria-label="Nova aba"
          title="Nova aba (Ctrl+T)"
          disabled={tabs.length >= 16}
          onClick={addTab}
        >
          <ChromeIcon name="plus" />
        </button>
      </div>
      <div className="browser-navigation">
        <button
          className="browser-icon-button"
          aria-label="Voltar"
          disabled={tab.index <= 0}
          onClick={() => go(tab.history[tab.index - 1], tab.id, tab.index - 1)}
        >
          <ChromeIcon name="back" />
        </button>
        <button
          className="browser-icon-button"
          aria-label="Avançar"
          disabled={tab.index >= tab.history.length - 1}
          onClick={() => go(tab.history[tab.index + 1], tab.id, tab.index + 1)}
        >
          <ChromeIcon name="forward" />
        </button>
        <button
          className="browser-icon-button"
          aria-label="Recarregar página"
          disabled={!tab.address || tab.loading}
          onClick={() => go(tab.address, tab.id, tab.index)}
        >
          <ChromeIcon name="reload" />
        </button>
        <button
          className="browser-icon-button"
          aria-label="Página inicial"
          onClick={() => go(preferences.home)}
        >
          <ChromeIcon name="home" />
        </button>
        <div className="browser-address-box" ref={addressBox}>
          <form
            className="browser-url"
            onSubmit={(event) => {
              event.preventDefault();
              go(tab.draft);
            }}
          >
            <button
              type="button"
              className={`browser-icon-button browser-security-button ${torAddress ? 'is-tor' : secureAddress ? 'is-secure' : 'is-insecure'}`}
              aria-label={
                torAddress
                  ? 'Onion Service v3 na rede Tor'
                  : secureAddress
                    ? 'Conexão HTTPS segura'
                    : 'Site sem HTTPS: conexão não segura'
              }
              title={
                torAddress
                  ? 'Tor · Onion Service v3 · sem DNS tradicional'
                  : secureAddress
                    ? 'HTTPS · certificado virtual válido'
                    : 'Não seguro · este site não usa HTTPS'
              }
              data-browser-popup="security"
              onClick={() => openPanel('security')}
            >
              <ChromeIcon name="shield" />
            </button>
            {torMode && <span className="browser-tor-badge">TOR</span>}
            <ChromeIcon name={secureAddress ? 'lock' : 'shield'} />
            {tab.address && !secureAddress && !torAddress && (
              <span className="browser-insecure-label">Não seguro</span>
            )}
            <input
              ref={addressInput}
              aria-label="Endereço"
              autoComplete="off"
              spellCheck={false}
              placeholder="Pesquisar nos favoritos ou digitar endereço virtual"
              value={tab.draft}
              onFocus={() => setSuggesting(true)}
              onChange={(event) => {
                const draft = event.target.value;
                change((items) =>
                  items.map((item) => (item.id === tab.id ? { ...item, draft } : item)),
                );
                setSuggesting(true);
              }}
            />
            {preferences.zoom !== 100 && (
              <button
                type="button"
                className="browser-zoom-badge"
                title="Restaurar zoom"
                disabled={saving}
                onClick={() => {
                  void savePreferences({ ...preferences, zoom: 100 });
                }}
              >
                {preferences.zoom}%
              </button>
            )}
            <button
              type="button"
              className={`browser-icon-button browser-star ${bookmarkIndex >= 0 && tab.address ? 'is-bookmarked' : ''}`}
              aria-label={
                bookmarkIndex >= 0 && tab.address ? 'Editar favorito' : 'Adicionar aos favoritos'
              }
              disabled={!tab.page || saving}
              onClick={star}
            >
              <ChromeIcon name="star" />
            </button>
          </form>
          {suggesting && suggestions.length > 0 && (
            <div className="browser-suggestions" aria-label="Sugestões de endereço">
              {suggestions.map((item) => (
                <button key={item.address} onClick={() => go(item.address)}>
                  <ChromeIcon name="history" />
                  <span>
                    {item.title}
                    <small>{item.address}</small>
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>
        <button
          className="browser-icon-button browser-secondary-control"
          aria-label="Downloads"
          data-browser-popup="downloads"
          aria-expanded={panel === 'downloads'}
          onClick={() => openPanel('downloads')}
        >
          <ChromeIcon name="downloads" />
        </button>
        <button
          className="browser-icon-button browser-secondary-control"
          aria-label="Extensões"
          data-browser-popup="extensions"
          aria-expanded={panel === 'extensions'}
          onClick={() => openPanel('extensions')}
        >
          <ChromeIcon name="extensions" />
        </button>
        <button
          className="browser-icon-button"
          aria-label="Menu do navegador"
          data-browser-popup="menu"
          aria-expanded={panel === 'menu'}
          onClick={() => openPanel('menu')}
        >
          <ChromeIcon name="menu" />
        </button>
      </div>
      {preferences.showBookmarks && (
        <nav className="browser-bookmarks" aria-label="Barra de favoritos">
          {preferences.bookmarks.map((bookmark, index) => (
            <button
              key={`${bookmark.address}:${index}`}
              title={bookmark.address}
              onClick={() => go(bookmark.address)}
            >
              <KaliIcon name="firefox" size={12} />
              <span>{bookmark.title}</span>
            </button>
          ))}
        </nav>
      )}
      {panel && (
        <div
          ref={popup}
          className={`browser-popup browser-popup-${panel}`}
          role="dialog"
          aria-label={
            panel === 'settings'
              ? 'Configurações do navegador'
              : panel === 'bookmarks'
                ? 'Favoritos'
                : 'Painel do navegador'
          }
        >
          {panel === 'menu' && (
            <>
              <strong>Navegador</strong>
              <button onClick={addTab} disabled={tabs.length >= 16}>
                <ChromeIcon name="plus" />
                Nova aba<kbd>Ctrl+T</kbd>
              </button>
              <hr />
              <button onClick={() => openPanel('bookmarks')}>
                <ChromeIcon name="star" />
                Favoritos
              </button>
              <button onClick={() => openPanel('history')}>
                <ChromeIcon name="history" />
                Histórico
              </button>
              <button onClick={() => openPanel('downloads')}>
                <ChromeIcon name="downloads" />
                Downloads
              </button>
              <hr />
              <div className="browser-zoom-controls">
                <span>Zoom</span>
                <button
                  aria-label="Diminuir zoom"
                  disabled={saving || preferences.zoom <= 50}
                  onClick={() => {
                    void savePreferences({
                      ...preferences,
                      zoom: Math.max(50, preferences.zoom - 10),
                    });
                  }}
                >
                  <ChromeIcon name="minus" />
                </button>
                <button
                  aria-label="Restaurar zoom"
                  disabled={saving}
                  onClick={() => {
                    void savePreferences({ ...preferences, zoom: 100 });
                  }}
                >
                  {preferences.zoom}%
                </button>
                <button
                  aria-label="Aumentar zoom"
                  disabled={saving || preferences.zoom >= 200}
                  onClick={() => {
                    void savePreferences({
                      ...preferences,
                      zoom: Math.min(200, preferences.zoom + 10),
                    });
                  }}
                >
                  <ChromeIcon name="plus" />
                </button>
              </div>
              <hr />
              <button onClick={() => openPanel('settings')}>
                <ChromeIcon name="settings" />
                Configurações
              </button>
              <button onClick={() => openPanel('extensions')}>
                <ChromeIcon name="extensions" />
                Extensões
              </button>
            </>
          )}
          {panel === 'tabs' && (
            <>
              <strong>Abas abertas</strong>
              {tabs.map((item) => (
                <button
                  key={item.id}
                  onClick={() => {
                    select(item.id);
                    setPanel('');
                  }}
                >
                  {item.page?.title || item.address || 'Nova aba'}
                </button>
              ))}
            </>
          )}
          {panel === 'bookmarks' && (
            <>
              <strong>Favoritos</strong>
              {bookmarkDraft ? (
                <form
                  onSubmit={(event) => {
                    event.preventDefault();
                    const bookmarks = [...preferences.bookmarks];
                    const item = {
                      title: bookmarkDraft.title.trim(),
                      address: bookmarkDraft.address.trim(),
                    };
                    if (bookmarkDraft.index < 0) {
                      bookmarks.push(item);
                    } else {
                      bookmarks[bookmarkDraft.index] = item;
                    }
                    void savePreferences({ ...preferences, bookmarks }).then((saved) => {
                      if (saved && mounted.current) {
                        setBookmarkDraft(null);
                      }
                    });
                  }}
                >
                  <label>
                    Nome
                    <input
                      aria-label="Nome do favorito"
                      value={bookmarkDraft.title}
                      maxLength={100}
                      required
                      onChange={(e) =>
                        setBookmarkDraft({ ...bookmarkDraft, title: e.target.value })
                      }
                    />
                  </label>
                  <label>
                    Endereço
                    <input
                      aria-label="Endereço do favorito"
                      value={bookmarkDraft.address}
                      maxLength={512}
                      required
                      onChange={(e) =>
                        setBookmarkDraft({ ...bookmarkDraft, address: e.target.value })
                      }
                    />
                  </label>
                  <div className="browser-form-actions">
                    <button type="button" disabled={saving} onClick={() => setBookmarkDraft(null)}>
                      Cancelar
                    </button>
                    <button disabled={saving}>Salvar favorito</button>
                  </div>
                </form>
              ) : (
                <>
                  {preferences.bookmarks.map((item, index) => (
                    <div className="browser-bookmark-row" key={index}>
                      <button title={item.address} onClick={() => go(item.address)}>
                        {item.title}
                      </button>
                      <button
                        aria-label={`Editar ${item.title}`}
                        onClick={() => setBookmarkDraft({ index, ...item })}
                      >
                        <ChromeIcon name="settings" />
                      </button>
                      <button
                        aria-label={`Remover ${item.title}`}
                        disabled={saving}
                        onClick={() => {
                          void savePreferences({
                            ...preferences,
                            bookmarks: preferences.bookmarks.filter((_, i) => i !== index),
                          });
                        }}
                      >
                        <ChromeIcon name="close" />
                      </button>
                    </div>
                  ))}
                  {!preferences.bookmarks.length && <p>Nenhum favorito salvo.</p>}
                  <button
                    disabled={preferences.bookmarks.length >= 50 || saving}
                    onClick={() =>
                      setBookmarkDraft({
                        index: -1,
                        title: tab.page?.title ?? '',
                        address: tab.address,
                      })
                    }
                  >
                    Adicionar favorito
                  </button>
                </>
              )}
            </>
          )}
          {panel === 'history' && (
            <>
              <strong>Histórico desta sessão</strong>
              {browser.history.map((item) => (
                <button key={item.address} title={item.address} onClick={() => go(item.address)}>
                  {item.title}
                </button>
              ))}
              {!browser.history.length && <p>Nenhuma página visitada.</p>}
              <button onClick={browser.clearHistory}>Limpar histórico da sessão</button>
            </>
          )}
          {panel === 'settings' && (
            <>
              <strong>Configurações</strong>
              <form
                onSubmit={(event) => {
                  event.preventDefault();
                  void savePreferences({ ...preferences, home: homeDraft.trim() });
                }}
              >
                <label>
                  Página inicial
                  <input
                    aria-label="Endereço da página inicial"
                    required
                    maxLength={512}
                    value={homeDraft}
                    onChange={(e) => setHomeDraft(e.target.value)}
                  />
                </label>
                <button disabled={saving}>Salvar página inicial</button>
              </form>
              <label className="browser-checkbox">
                <input
                  type="checkbox"
                  checked={preferences.showBookmarks}
                  disabled={saving}
                  onChange={(e) => {
                    void savePreferences({ ...preferences, showBookmarks: e.target.checked });
                  }}
                />
                Mostrar barra de favoritos
              </label>
              <p>
                Tema escuro. Favoritos, zoom e página inicial são salvos na campanha. Abas e
                histórico são limpos ao encerrar o navegador.
              </p>
            </>
          )}
          {panel === 'security' && (
            <>
              <strong>
                {torAddress
                  ? 'Tor · Onion Service v3'
                  : secureAddress
                    ? 'Conexão HTTPS segura'
                    : 'Site não seguro'}
              </strong>
              <p>{tab.address || 'Nova aba'}</p>
              <p>
                {torAddress
                  ? 'Este endereço foi identificado automaticamente como um Onion Service v3. Ele usa a rede Tor, não participa do DNS tradicional e não publica um IP convencional.'
                  : secureAddress
                    ? 'Certificado virtual válido para a internet do jogo. A conexão é simulada e não acessa a internet do Windows.'
                    : 'Alerta: este endereço foi aberto sem HTTPS e não possui certificado TLS virtual. Não envie credenciais ou dados sensíveis.'}
              </p>
            </>
          )}
          {panel === 'extensions' && (
            <>
              <strong>Extensões</strong>
              <p>
                Nenhuma extensão instalada. Extensões do Firefox e programas do Windows não são
                executados neste navegador virtual.
              </p>
            </>
          )}
          {panel === 'downloads' && (
            <>
              <strong>Downloads</strong>
              {downloads.length ? (
                downloads.map((file) => (
                  <button
                    key={file.id}
                    onClick={() => {
                      useWindows.getState().open('files', '/home/kali/Downloads');
                      setPanel('');
                    }}
                  >
                    {file.name}
                  </button>
                ))
              ) : (
                <p>A pasta Downloads está vazia.</p>
              )}
              <button
                onClick={() => {
                  useWindows.getState().open('files', '/home/kali/Downloads');
                  setPanel('');
                }}
              >
                Abrir pasta Downloads
              </button>
            </>
          )}
          {preferenceError && (
            <p role="alert" className="browser-preference-error">
              {preferenceError}
            </p>
          )}
        </div>
      )}
      <div
        id="browser-current-page"
        role="tabpanel"
        aria-labelledby={`browser-tab-${tab.id}`}
        className="browser-viewport"
      >
        {tab.loading ? (
          <div className="browser-loading" role="status">
            Carregando página virtual…
          </div>
        ) : (
          <div className="browser-zoom-page" style={{ zoom: `${preferences.zoom}%` }}>
            <BrowserContent
              address={addressKey(tab.address)}
              page={tab.page}
              error={tab.error}
              value={tab.value}
              busy={actionPending}
              navigate={go}
              setValue={(value) =>
                change((items) =>
                  items.map((item) => (item.id === tab.id ? { ...item, value } : item)),
                )
              }
              action={() => {
                void action();
              }}
            />
          </div>
        )}
      </div>
    </div>
  );
}
