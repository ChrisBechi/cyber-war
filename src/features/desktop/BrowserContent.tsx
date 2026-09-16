import type { BrowserPage } from '../../lib/api';
import { AppIcon } from './AppIcon';
import { DomainPortal } from './DomainPortal';
import { addressKey, BLACKWIRE_ONION_ADDRESS } from './browser-model';
import { VigiliaDownload } from './VigiliaDownload';
import './vigilia-download.css';

export function BrowserContent({
  address,
  page,
  error,
  value,
  busy,
  navigate,
  setValue,
  action,
}: {
  address: string;
  page: BrowserPage | null;
  error: string;
  value: string;
  busy: boolean;
  navigate: (address: string) => void;
  setValue: (value: string) => void;
  action: () => void;
}) {
  const isBlackwire = address.startsWith(BLACKWIRE_ONION_ADDRESS);
  const isVigilia = addressKey(address) === 'vigilia.org';
  return (
    <div
      className={`app-scroll browser-page ${!page ? 'browser-start-page' : isVigilia ? 'browser-vigilia' : isBlackwire ? 'browser-deep' : address === 'b1.tech' ? 'browser-news' : ''}`}
    >
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {page ? (
        <>
          {isVigilia ? (
            <VigiliaDownload busy={busy} onDownload={action} />
          ) : address === 'meudominio.com.br' ? (
            <DomainPortal />
          ) : isBlackwire || address === 'b1.tech' ? (
            <div className="browser-publication">
              <header>
                <strong>{isBlackwire ? 'BLACKWIRE' : 'B1'}</strong>
                <span>
                  {isBlackwire
                    ? 'ESPELHOS / ARQUIVOS / COMUNIDADE'
                    : 'TECNOLOGIA / PRIVACIDADE / MUNDO'}
                </span>
              </header>
              <div className="publication-columns">
                <main>
                  <div className="eyebrow">{address}</div>
                  <h1>{page.title}</h1>
                  {page.body.split('\n\n').map((article, index) => {
                    const [heading, ...body] = article.split('\n');
                    return (
                      <article key={index}>
                        <h2>{heading}</h2>
                        <p>{body.join('\n')}</p>
                      </article>
                    );
                  })}
                </main>
                <aside>
                  <small>{isBlackwire ? 'DIRETÓRIO' : 'EM ACOMPANHAMENTO'}</small>
                  <h3>{isBlackwire ? 'A trilha continua.' : 'Os rastros de uma conexão'}</h3>
                  <p>
                    {address.startsWith('blackwire')
                      ? 'Mensagens, espelhos e registros preservados pela comunidade.'
                      : 'Rede corporativa, arquivos e horários. Acompanhe os registros desta investigação.'}
                  </p>
                  {isBlackwire && (
                    <button
                      onClick={() =>
                        navigate(
                          address.endsWith('/archive')
                            ? BLACKWIRE_ONION_ADDRESS
                            : `${BLACKWIRE_ONION_ADDRESS}/archive`,
                        )
                      }
                    >
                      {address.endsWith('/archive') ? 'Voltar ao índice' : 'Abrir arquivo →'}
                    </button>
                  )}
                  <div className="publication-stamp">
                    23:17
                    <br />
                    <small>{isBlackwire ? 'ÚLTIMA ATIVIDADE' : 'ÚLTIMA ATUALIZAÇÃO'}</small>
                  </div>
                </aside>
              </div>
            </div>
          ) : (
            <>
              <div className="eyebrow">{address}</div>
              <h1>{page.title}</h1>
              {page.title === 'Virtual Software Repository' ? (
                <div className="page-body">
                  {page.body
                    .split('\n\n')
                    .filter(Boolean)
                    .map((item) => {
                      const [name, url] = item.split('\n');
                      return (
                        <p key={url}>
                          <button disabled={busy} onClick={() => navigate(url)}>
                            {name}
                          </button>
                        </p>
                      );
                    })}
                </div>
              ) : (
                <p className="page-body">{page.body}</p>
              )}
            </>
          )}
          {!isVigilia && page.action === 'recover' && (
            <label>
              Código correlacionado
              <input
                aria-label="Código de recuperação"
                value={value}
                onChange={(e) => setValue(e.target.value)}
              />
            </label>
          )}
          {!isVigilia && page.action && (
            <button className="primary" disabled={busy} onClick={action}>
              {page.action.startsWith('package-download:')
                ? 'Baixar pacote .deb'
                : page.action === 'firmware-download'
                  ? 'Baixar firmware_AXR550_v1.4.tar.gz'
                  : page.action === 'source-download'
                    ? 'Baixar tool-2.1.tar.gz'
                    : page.action === 'download'
                      ? 'Baixar Cyber Siege'
                      : page.action === 'recover'
                        ? 'Recuperar acesso'
                        : 'Comprar melhoria'}
            </button>
          )}
        </>
      ) : (
        <div className="browser-home">
          <AppIcon name="browser" size={64} />
          <h1>Uma janela para o mundo.</h1>
          <p>Favoritos, conversas e coisas que você ainda não sabe.</p>
          <button onClick={() => navigate('wipedia.org')}>Abrir a biblioteca →</button>
        </div>
      )}
    </div>
  );
}
