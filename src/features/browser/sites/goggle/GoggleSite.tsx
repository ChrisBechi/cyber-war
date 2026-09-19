import { useEffect, useRef, useState } from 'react';
import { emptySchema, request } from '../../../../lib/api';
import type { BrowserPage } from '../../../../lib/api';
import { perform, useGame } from '../../../../lib/game-store';
import { GoggleHeader } from './GoggleHeader';
import { GoggleHome } from './GoggleHome';
import { GoggleFooter } from './GoggleFooter';
import { GoggleSearch } from './GoggleSearch';
import { GoggleLogin } from './GoggleLogin';
import { SearchBox } from './SearchBox';
import { GoggleIcon } from './GoggleIcon';
import {
  GOGGLE_HOME,
  apps,
  goggleProvider,
  goggleRoute,
  searchResponseSchema,
  sessionSchema,
} from './goggle-model';
import type { GoggleSession } from './goggle-model';
import './goggle.css';

export function GoggleSite({
  address,
  page,
  navigate,
}: {
  address: string;
  page: BrowserPage;
  navigate: (address: string) => void;
}) {
  const route = goggleRoute(address);
  const [session, setSession] = useState<GoggleSession>({
    account: null,
    historyEnabled: false,
    history: [],
  });
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const revision = useGame((state) => state.revision);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    let active = true;
    void request('goggle_session', {}, sessionSchema)
      .then((next) => {
        if (active) {
          setSession(next);
        }
      })
      .catch((e: unknown) => {
        if (active) {
          setError(String(e));
        }
      });
    return () => {
      active = false;
    };
  }, [revision]);
  const mutate = async (command: string, args: Record<string, unknown> = {}) => {
    setBusy(true);
    setError('');
    try {
      await perform(command, args, emptySchema);
      const next = await request('goggle_session', {}, sessionSchema);
      if (mounted.current) {
        setSession(next);
      }
    } catch (e) {
      if (mounted.current) {
        setError(String(e));
      }
    } finally {
      if (mounted.current) {
        setBusy(false);
      }
    }
  };
  const search = (query: string, lucky = false) => {
    if (!lucky) {
      navigate(goggleProvider.searchUrl(query));
      return;
    }
    setBusy(true);
    setError('');
    void request(
      'search_query',
      { query, mode: 'all', source: null, offset: 0 },
      searchResponseSchema,
    )
      .then((result) => {
        if (mounted.current) {
          navigate(result.documents[0]?.url ?? goggleProvider.searchUrl(query));
        }
      })
      .catch((e: unknown) => {
        if (mounted.current) {
          setError(String(e));
        }
      })
      .finally(() => {
        if (mounted.current) {
          setBusy(false);
        }
      });
  };
  const imageSearch = (source: string) =>
    navigate(`${GOGGLE_HOME}/search?q=&image=${encodeURIComponent(source)}`);
  if (!route) {
    return null;
  }
  const isHome = route.path === '/';
  const isSearch = ['/search', '/images'].includes(route.path);
  const app = apps.find((app) => route.path === `/apps/${app.id}`);
  return (
    <div className="goggle-site" aria-busy={busy}>
      <GoggleHeader
        account={session.account}
        navigate={navigate}
        home={isHome}
        busy={busy}
        logout={() => {
          void mutate('goggle_logout');
        }}
      >
        {isSearch && (
          <SearchBox initialQuery={route.query} compact search={search} imageSearch={imageSearch} />
        )}
      </GoggleHeader>
      {error && (
        <p role="alert" className="goggle-service-error">
          {error}
        </p>
      )}
      {isHome ? (
        <GoggleHome search={search} imageSearch={imageSearch} />
      ) : isSearch ? (
        <GoggleSearch
          query={route.query}
          mode={route.path === '/images' ? 'images' : route.mode}
          source={route.image}
          offset={route.offset}
          navigate={navigate}
          imageSearch={imageSearch}
        />
      ) : route.path === '/login' ? (
        <GoggleLogin complete={() => navigate(GOGGLE_HOME)} />
      ) : route.path === '/account' ? (
        <main className="goggle-info">
          <h1>
            {session.account
              ? `Olá, ${session.account.displayName}`
              : 'Suas preferências de pesquisa'}
          </h1>
          {session.account && <p>{session.account.email}</p>}
          <h2>Histórico de pesquisa</h2>
          <p>Guarde suas pesquisas nesta campanha para consultá-las depois.</p>
          <label className="goggle-history-toggle">
            <input
              type="checkbox"
              checked={session.historyEnabled}
              disabled={busy}
              onChange={(e) => {
                void mutate('goggle_preferences', {
                  historyEnabled: e.target.checked,
                  clearHistory: false,
                });
              }}
            />
            Salvar histórico de pesquisa
          </label>
          <button
            className="goggle-outline"
            disabled={busy || !session.history.length}
            onClick={() => {
              void mutate('goggle_preferences', {
                historyEnabled: session.historyEnabled,
                clearHistory: true,
              });
            }}
          >
            Limpar histórico
          </button>
          <ul className="goggle-history">
            {session.history.map((query) => (
              <li key={query}>
                <button onClick={() => search(query)}>{query}</button>
              </li>
            ))}
          </ul>
          {!session.history.length && <p>Nenhuma pesquisa salva.</p>}
        </main>
      ) : app ? (
        <main className="goggle-info goggle-coming-soon">
          <GoggleIcon name={app.icon} />
          <h1>{app.name}</h1>
          <span className="goggle-status-badge">Em breve</span>
          <p>Estamos preparando este serviço. Ele ainda não está disponível.</p>
          <button className="goggle-blue" onClick={() => navigate(GOGGLE_HOME)}>
            Voltar à pesquisa
          </button>
        </main>
      ) : (
        <main className="goggle-info">
          <h1>{page.title}</h1>
          {page.body.split('\n\n').map((paragraph, i) => (
            <p key={i}>{paragraph}</p>
          ))}
          {route.path === '/privacy' && (
            <button className="goggle-outline" onClick={() => navigate(`${GOGGLE_HOME}/account`)}>
              Gerenciar histórico
            </button>
          )}
        </main>
      )}
      <GoggleFooter navigate={navigate} />
    </div>
  );
}
