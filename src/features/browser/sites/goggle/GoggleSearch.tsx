import { useEffect, useState } from 'react';
import { request } from '../../../../lib/api';
import { useGame } from '../../../../lib/game-store';
import { GoggleImages } from './GoggleImages';
import { SearchResult } from './SearchResult';
import { GOGGLE_HOME, searchResponseSchema } from './goggle-model';
import type { SearchResponse } from './goggle-model';
import { Sponsored } from '../../virtual-web/Sponsored';
import '../../virtual-web/community.css';

export function GoggleSearch({
  query,
  mode,
  source,
  offset,
  navigate,
  imageSearch,
}: {
  query: string;
  mode: 'all' | 'images' | 'videos' | 'news' | 'shopping';
  source: string | null;
  offset: number;
  navigate: (address: string) => void;
  imageSearch: (source: string) => void;
}) {
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [error, setError] = useState('');
  const revision = useGame((state) => state.revision);
  useEffect(() => {
    let active = true;
    setResults(null);
    setError('');
    const timer = window.setTimeout(() => {
      if (active) {
        active = false;
        setError('A pesquisa demorou mais que o esperado. Tente novamente.');
      }
    }, 5000);
    void request('search_query', { query, mode, source, offset }, searchResponseSchema)
      .then((result) => {
        if (active) {
          setResults(result);
          window.clearTimeout(timer);
        }
      })
      .catch((e: unknown) => {
        if (active) {
          setError(String(e));
          window.clearTimeout(timer);
        }
      });
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [query, mode, source, offset, revision]);
  const address = (tab: string, nextOffset = 0) =>
    `${GOGGLE_HOME}/${tab === 'images' ? 'images' : 'search'}?q=${encodeURIComponent(query)}${!['all', 'images'].includes(tab) ? `&type=${tab}` : ''}${source ? `&image=${encodeURIComponent(source)}` : ''}${nextOffset ? `&offset=${nextOffset}` : ''}`;
  return (
    <main className="goggle-results">
      <nav className="goggle-result-tabs" aria-label="Tipos de pesquisa">
        <button
          aria-current={mode === 'all' ? 'page' : undefined}
          onClick={() => navigate(address('all'))}
        >
          Todos
        </button>
        <button
          aria-current={mode === 'images' ? 'page' : undefined}
          onClick={() => navigate(address('images'))}
        >
          Imagens
        </button>
        {(
          [
            ['videos', 'Vídeos'],
            ['news', 'Notícias'],
            ['shopping', 'Compras'],
          ] as const
        ).map(([type, label]) => (
          <button
            key={type}
            aria-current={mode === type ? 'page' : undefined}
            onClick={() => navigate(address(type))}
          >
            {label}
          </button>
        ))}
      </nav>
      {error ? (
        <p role="alert" className="goggle-results-status">
          {error}
        </p>
      ) : !results ? (
        <p role="status" className="goggle-results-status">
          Pesquisando na rede…
        </p>
      ) : (
        <>
          {results.correction && (
            <p className="goggle-results-status">
              Você quis dizer:{' '}
              <button
                onClick={() =>
                  navigate(
                    `${GOGGLE_HOME}/search?q=${encodeURIComponent(results.correction ?? '')}`,
                  )
                }
              >
                {results.correction}
              </button>
            </p>
          )}
          <div className="goggle-results-status" role="status">
            {results.total} {results.total === 1 ? 'resultado' : 'resultados'}
            {source && <span> · Pesquisa por imagem</span>}
          </div>
          {results.total === 0 ? (
            <div className="goggle-no-results">
              <h1>Nenhum resultado encontrado</h1>
              <p>
                {source ? (
                  'Esta imagem ainda não está associada a publicações acessíveis.'
                ) : (
                  <>
                    Sua pesquisa por <strong>{query || 'uma consulta vazia'}</strong> não encontrou
                    páginas.
                  </>
                )}
              </p>
              <p>Tente outras palavras ou volte quando novas informações forem publicadas.</p>
            </div>
          ) : mode === 'images' ? (
            <GoggleImages results={results} navigate={navigate} imageSearch={imageSearch} />
          ) : (
            <div className="goggle-result-list">
              {!!results.ads?.length && <Sponsored ads={results.ads} navigate={navigate} />}
              {results.documents.map((document) => (
                <SearchResult key={document.id} document={document} navigate={navigate} />
              ))}
            </div>
          )}
          {results.total > 20 && (
            <nav className="goggle-pagination" aria-label="Páginas de resultados">
              <button
                disabled={offset === 0}
                onClick={() => navigate(address(mode, Math.max(0, offset - 20)))}
              >
                Anterior
              </button>
              <span>Página {Math.floor(offset / 20) + 1}</span>
              <button
                disabled={offset + 20 >= results.total}
                onClick={() => navigate(address(mode, offset + 20))}
              >
                Próxima
              </button>
            </nav>
          )}
        </>
      )}
    </main>
  );
}
