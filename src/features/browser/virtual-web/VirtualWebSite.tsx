import { useState, useEffect, type CSSProperties } from 'react';
import { pageSchema, request } from '../../../lib/api';
import { useGame, perform } from '../../../lib/game-store';
import {
  date,
  dateTime,
  stockLabel,
  webPageSchema,
  money,
  type WebCard,
  type WebDocument,
  type WebLink,
  type WebPage,
} from './web-model';
import { WebVisual } from './WebVisual';
import { CommunityPanel } from './CommunityPanel';
import { Sponsored } from './Sponsored';
import { WebDevTools } from './WebDevTools';
import './virtual-web.css';
import './community.css';
import './brand-identity.css';

type Navigate = (url: string) => void;
function Link({
  link,
  navigate,
  className,
}: {
  link: WebLink;
  navigate: Navigate;
  className?: string;
}) {
  return (
    <button className={className ?? 'web-link'} onClick={() => navigate(link.url)}>
      {link.label}
    </button>
  );
}
function Facts({ rows }: { rows: string[][] }) {
  return (
    <dl className="web-facts">
      {rows.map(([label, value], i) => (
        <div key={`${label}-${i}`}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}
function Card({
  card,
  navigate,
  compact = false,
}: {
  card: WebCard;
  navigate: Navigate;
  compact?: boolean;
}) {
  return (
    <article className={`web-card${compact ? ' web-card-compact' : ''}`}>
      <button
        className="web-card-image"
        aria-label={`Abrir ${card.title}`}
        onClick={() => navigate(card.url)}
      >
        <WebVisual kind={card.visual} />
      </button>
      <div className="web-card-copy">
        <span className="web-kicker">{card.category}</span>
        <h2>
          <button onClick={() => navigate(card.url)}>{card.title}</button>
        </h2>
        <p>{card.summary}</p>
        {card.priceCents !== null ? (
          <div>
            <strong className="web-price">{money(card.priceCents)}</strong>
            {card.stock && <small className="web-stock">{stockLabel(card.stock)}</small>}
            {card.listingStatus && (
              <small className="web-stock">
                {
                  { AVAILABLE: 'Disponível', RESERVED: 'Reservado', SOLD: 'Vendido' }[
                    card.listingStatus
                  ]
                }
              </small>
            )}
            {card.rating !== null && card.rating !== undefined && (
              <small className="web-stock">
                ★ {card.rating.toLocaleString('pt-BR', { maximumFractionDigits: 1 })} nas avaliações
              </small>
            )}
          </div>
        ) : (
          <small>
            {card.author} <span aria-hidden="true">↗</span>
          </small>
        )}
      </div>
    </article>
  );
}
function Blocks({ document, navigate }: { document: WebDocument; navigate: Navigate }) {
  return (
    <div className="web-prose">
      {document.blocks.map((block, i) => {
        switch (block.kind) {
          case 'heading':
            return (
              <h2 id={`section-${i}`} key={i}>
                {block.text}
              </h2>
            );
          case 'paragraph':
            return <p key={i}>{block.text}</p>;
          case 'quote':
            return (
              <blockquote key={i}>
                {block.text}
                <cite>{block.attribution}</cite>
              </blockquote>
            );
          case 'code':
            return (
              <pre key={i}>
                <small>{block.language}</small>
                <code>{block.text}</code>
              </pre>
            );
          case 'link':
            return (
              <p key={i}>
                <Link link={block} navigate={navigate} />
              </p>
            );
          case 'list':
            return block.ordered ? (
              <ol key={i}>
                {block.items.map((s, j) => (
                  <li key={j}>{s}</li>
                ))}
              </ol>
            ) : (
              <ul key={i}>
                {block.items.map((s, j) => (
                  <li key={j}>{s}</li>
                ))}
              </ul>
            );
          case 'table':
            return (
              <div className="web-table-wrap" key={i}>
                <table>
                  <thead>
                    <tr>
                      {block.headings.map((s, j) => (
                        <th key={j}>{s}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {block.rows.map((row, k) => (
                      <tr key={k}>
                        {row.map((s, j) => (
                          <td key={j}>{s}</td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            );
        }
      })}
    </div>
  );
}
function Detail({
  page,
  navigate,
  act,
  pending,
}: {
  page: WebPage;
  navigate: Navigate;
  act: (action: string, text?: string) => void;
  pending: boolean;
}) {
  const doc = page.document;
  if (!doc) {
    return null;
  }
  const detail = doc.detail;
  switch (detail.kind) {
    case 'socialPost':
      return (
        <div className="web-social-post-actions">
          <button disabled={pending} aria-pressed={page.liked} onClick={() => act('like')}>
            {page.liked ? '♥ Curtido' : '♡ Curtir'} · {detail.likes + Number(page.liked)}
          </button>
          <span>
            {doc.comments.length} {doc.comments.length === 1 ? 'comentário' : 'comentários'}
          </span>
        </div>
      );
    case 'product':
      return (
        <section className="web-product-buy">
          <span className="web-kicker">Casa, trabalho e pequenos prazeres</span>
          <strong className="web-price">{money(detail.priceCents)}</strong>
          <p>{stockLabel(detail.stock)}</p>
          <button
            className="web-primary"
            disabled={pending || !['IN_STOCK', 'LOW_STOCK'].includes(detail.stock)}
            onClick={() => act('cart')}
          >
            Adicionar ao carrinho
          </button>
          <p>
            Vendido por <Link link={detail.seller} navigate={navigate} />
          </p>
          <Facts rows={detail.specifications} />
          {page.offerHistory.length > 1 && (
            <details className="web-offer-history">
              <summary>Histórico de preço e estoque</summary>
              <ol>
                {[...page.offerHistory].reverse().map((offer, i) => (
                  <li key={`${offer.timestamp}-${i}`}>
                    <time dateTime={new Date(offer.timestamp * 1000).toISOString()}>
                      {dateTime(offer.timestamp)}
                    </time>
                    <strong>{money(offer.priceCents)}</strong>
                    <span>{stockLabel(offer.stock)}</span>
                  </li>
                ))}
              </ol>
            </details>
          )}
        </section>
      );
    case 'recipe':
      return (
        <section className="web-recipe">
          <div className="web-stats">
            <span>
              <b>{detail.minutes} min</b>preparo total
            </span>
            <span>
              <b>{detail.servings}</b>porções
            </span>
            <span>
              <b>{detail.difficulty}</b>dificuldade
            </span>
          </div>
          <div className="web-recipe-columns">
            <section>
              <h2>Ingredientes</h2>
              {detail.ingredients.map((item, i) => (
                <label key={i}>
                  <input type="checkbox" />
                  {item}
                </label>
              ))}
            </section>
            <section>
              <h2>Modo de preparo</h2>
              <ol>
                {detail.steps.map((item, i) => (
                  <li key={i}>{item}</li>
                ))}
              </ol>
            </section>
          </div>
        </section>
      );
    case 'course':
      return (
        <section className="web-curriculum">
          <div className="web-stats">
            <span>
              <b>{detail.minutes} min</b>de estudo
            </span>
            <span>
              <b>{detail.level}</b>nível
            </span>
            <span>
              <b>Gratuito</b>acesso livre
            </span>
          </div>
          <p>
            Com <Link link={detail.teacher} navigate={navigate} />
          </p>
          <h2>Seu percurso</h2>
          {detail.lessons.map((lesson, i) => (
            <Link key={i} link={lesson} navigate={navigate} className="web-lesson-link" />
          ))}
        </section>
      );
    case 'lesson':
      return (
        <section className="web-lesson-controls">
          <p>
            <Link link={detail.course} navigate={navigate} /> · Aula {detail.position}
          </p>
          <button
            className="web-primary"
            disabled={pending || page.completed}
            onClick={() => act('complete')}
          >
            {page.completed ? '✓ Aula concluída' : 'Concluir esta aula'}
          </button>
          {detail.next && <Link link={detail.next} navigate={navigate} />}
        </section>
      );
    case 'thread':
      return (
        <div className="web-thread-meta">
          <button disabled={pending} aria-pressed={page.liked} onClick={() => act('like')}>
            ↑ {detail.votes + Number(page.liked)}
          </button>
          <span>
            r/{detail.community} · {detail.locked ? 'Conversa encerrada' : 'Conversa aberta'}
          </span>
        </div>
      );
    case 'video':
      return (
        <section className="web-video-info">
          <p>
            <Link link={detail.channel} navigate={navigate} /> ·{' '}
            {detail.views.toLocaleString('pt-BR')} visualizações · {Math.floor(detail.seconds / 60)}
            :{String(detail.seconds % 60).padStart(2, '0')}
          </p>
          <h2>Neste vídeo</h2>
          <ul>
            {detail.chapters.map((c) => (
              <li key={c}>{c}</li>
            ))}
          </ul>
          <button aria-pressed={page.liked} disabled={pending} onClick={() => act('like')}>
            {page.liked ? '♥ Curtido' : '♡ Curtir'}
          </button>
        </section>
      );
    case 'profile':
      return (
        <aside className="web-profile">
          <h2>Sobre</h2>
          <p>{detail.bio}</p>
          <p>{detail.location}</p>
        </aside>
      );
    case 'reference':
      return <Facts rows={detail.facts} />;
    case 'article':
      return (
        <div className="web-article-meta">
          {detail.readingMinutes} min de leitura ·{' '}
          <button disabled={pending} aria-pressed={page.liked} onClick={() => act('like')}>
            {page.liked ? '♥ Curtido' : '♡ Curtir'}
          </button>
        </div>
      );
  }
}
function Comments({
  document,
  act,
  pending,
}: {
  document: WebDocument;
  act: (action: string, text?: string) => Promise<boolean>;
  pending: boolean;
}) {
  const [text, setText] = useState('');
  const [limit, setLimit] = useState(8);
  const enabled =
    !['reference', 'profile', 'socialProfile'].includes(document.detail.kind) &&
    !(document.detail.kind === 'thread' && document.detail.locked);
  if (!enabled && !document.comments.length) {
    return null;
  }
  return (
    <section className="web-comments">
      <h2>
        {document.detail.kind === 'product' ? 'Avaliações e perguntas' : 'Conversa'}{' '}
        <small>{document.comments.length}</small>
      </h2>
      {document.comments.slice(0, limit).map((c) => (
        <article key={c.id}>
          <div className="web-avatar">{c.author.slice(0, 1).toUpperCase()}</div>
          <div>
            <strong>{c.author}</strong>
            <small>
              {date(c.publishedAt)}
              {c.rating !== null && ` · ${'★'.repeat(c.rating)}`}
            </small>
            <p>{c.text}</p>
          </div>
        </article>
      ))}
      {limit < document.comments.length && (
        <button onClick={() => setLimit(limit + 8)}>Ver mais comentários</button>
      )}
      {enabled && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void act('comment', text).then((saved) => {
              if (saved) {
                setText('');
              }
            });
          }}
        >
          <label>
            Participe da conversa
            <textarea
              required
              maxLength={1000}
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder="Compartilhe uma experiência ou faça uma pergunta"
            />
          </label>
          <button className="web-primary" disabled={pending || !text.trim()}>
            Publicar comentário
          </button>
        </form>
      )}
    </section>
  );
}

export function VirtualWebSite({
  initialPage,
  navigate,
}: {
  initialPage: WebPage;
  navigate: Navigate;
}) {
  const [page, setPage] = useState(initialPage);
  const [query, setQuery] = useState(page.query);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [cartOpen, setCartOpen] = useState(false);
  const worldRevision = useGame((s) => s.revision);
  useEffect(() => {
    let active = true;
    void request('web_page', { address: initialPage.canonicalUrl }, webPageSchema.nullable())
      .then((next) => {
        if (active && next) {
          setPage(next);
        }
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, [worldRevision, initialPage.canonicalUrl]);
  const home = `https://www.${page.brand.domain}/`;
  const listing = (category: string, offset = 0) =>
    `${home}search?q=${encodeURIComponent(query)}&category=${encodeURIComponent(category)}&offset=${offset}${Object.entries(
      page.filters ?? {},
    )
      .map(([key, value]) => `&${key}=${encodeURIComponent(value)}`)
      .join('')}`;
  const act = async (
    action: string,
    text = '',
    id = page.document?.id ?? `web-${page.brand.id}-home`,
  ) => {
    if (!id || pending) {
      return false;
    }
    setPending(true);
    setError('');
    setNotice('');
    try {
      const result = await perform(
        'web_interact',
        { id, action, text, address: page.canonicalUrl },
        pageSchema,
      );
      if (result.virtualWeb) {
        setPage(result.virtualWeb);
      }
      setNotice(
        action === 'cart'
          ? 'Produto adicionado ao carrinho.'
          : action === 'comment'
            ? 'Comentário publicado.'
            : action === 'complete'
              ? 'Seu progresso foi salvo.'
              : '',
      );
      return true;
    } catch (e) {
      setError(String(e));
      return false;
    } finally {
      setPending(false);
    }
  };
  const doc = page.document?.path === '/' ? null : page.document;
  const displayPage = { ...page, document: doc };
  const style = { '--web-accent': page.brand.accent } as CSSProperties;
  return (
    <div
      className={`virtual-web web-${page.brand.layout}`}
      style={style}
      data-world-revision={worldRevision}
      data-typography={page.brand.identity?.typography}
      data-header={page.brand.identity?.header}
      data-cards={page.brand.identity?.cards}
      data-density={page.brand.identity?.density}
      data-surface={page.brand.identity?.surface}
    >
      <div className="web-topline">
        <span>{page.brand.platform === 'EDITORIAL' ? 'EDIÇÃO DIGITAL' : page.brand.domain}</span>
        <button onClick={() => navigate('https://www.goggle.com')}>Pesquisar no Goggle ↗</button>
      </div>
      <header className="web-header">
        <button className="web-brand" onClick={() => navigate(home)}>
          <span className="web-mark">{page.brand.mark}</span>
          <span>
            {page.brand.name}
            <small>{page.brand.tagline}</small>
          </span>
        </button>
        <form
          role="search"
          onSubmit={(e) => {
            e.preventDefault();
            navigate(listing(''));
          }}
        >
          <input
            aria-label={`Pesquisar em ${page.brand.name}`}
            placeholder={
              page.brand.platform === 'COMMERCE' ? 'O que você procura?' : 'Buscar no site'
            }
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            maxLength={200}
          />
          <button aria-label="Buscar no site">⌕</button>
        </form>
        {page.brand.platform === 'COMMERCE' && (
          <button className="web-cart-button" onClick={() => setCartOpen(!cartOpen)}>
            Carrinho <b>{page.cartCount}</b>
          </button>
        )}
      </header>
      <nav className="web-navigation" aria-label={`Navegação ${page.brand.name}`}>
        <button onClick={() => navigate(home)}>Início</button>
        {page.brand.navigation.map((l) => (
          <Link key={l.url} link={l} navigate={navigate} />
        ))}
        {page.brand.id === 'nexora' && (
          <Link link={{ label: 'Sobre a Nexora', url: `${home}sobre` }} navigate={navigate} />
        )}
      </nav>
      {error && (
        <p className="web-notice" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="web-notice" role="status">
          {notice}
        </p>
      )}
      {cartOpen && (
        <section className="web-cart-panel">
          <h2>Seu carrinho</h2>
          <p>
            {page.cartCount
              ? `${page.cartCount} ${page.cartCount === 1 ? 'item adicionado' : 'itens adicionados'}.`
              : 'Seu carrinho está vazio.'}
          </p>
          {page.cart.map((item) => (
            <div className="web-cart-row" key={item.card.id}>
              <div>
                {item.card.url ? (
                  <Link link={{ label: item.card.title, url: item.card.url }} navigate={navigate} />
                ) : (
                  <span>{item.card.title}</span>
                )}
                {item.available === false && (
                  <small className="web-stock">Indisponível · fora do total</small>
                )}
              </div>
              <span>
                {item.quantity} ×{' '}
                {item.card.priceCents === null ? '—' : money(item.card.priceCents)}
              </span>
              <button disabled={pending} onClick={() => void act('removeCart', '', item.card.id)}>
                Remover
              </button>
            </div>
          ))}
          <strong>
            Total disponível:{' '}
            {money(
              page.cart.reduce(
                (sum, item) =>
                  sum +
                  (item.available === false ? 0 : (item.card.priceCents ?? 0) * item.quantity),
                0,
              ),
            )}
          </strong>
          {!!page.cart.length && (
            <p>
              Os valores acompanham o preço atual da loja. Produtos indisponíveis não entram no
              total.
            </p>
          )}
          <button onClick={() => setCartOpen(false)}>Continuar explorando</button>
        </section>
      )}
      <main className="web-main">
        {!!page.ads?.length && <Sponsored ads={page.ads} navigate={navigate} />}
        {page.capture && (
          <aside className="web-archive-banner">
            <strong>Captura de {dateTime(page.capture.timestamp)}</strong>
            <p>
              Esta versão foi preservada antes da retirada. Ofertas e informações podem ter mudado.
            </p>
            <button className="web-link" onClick={() => navigate(page.capture!.originalUrl)}>
              Consultar endereço original
            </button>
          </aside>
        )}
        {!doc && <CommunityPanel page={page} act={act} navigate={navigate} pending={pending} />}
        {page.status === 404 ? (
          <section className="web-not-found">
            <span className="web-kicker">{page.brand.name}</span>
            <strong>404</strong>
            <h1>
              {page.brand.platform === 'FORUM'
                ? 'Esta conversa não foi encontrada.'
                : page.brand.platform === 'COMMERCE'
                  ? 'Não encontramos este produto ou página.'
                  : 'Esta página não está disponível.'}
            </h1>
            <p>O endereço pode ter mudado ou a publicação pode ter sido retirada.</p>
            {page.archiveUrl && (
              <button onClick={() => navigate(page.archiveUrl!)}>
                Consultar captura na Memória Web
              </button>
            )}
            <button className="web-primary" onClick={() => navigate(home)}>
              Voltar ao início
            </button>
          </section>
        ) : doc ? (
          <>
            <div className="web-breadcrumb">
              <button onClick={() => navigate(home)}>{page.brand.name}</button>
              <span>/</span>
              <button
                onClick={() => navigate(`${home}?category=${encodeURIComponent(doc.category)}`)}
              >
                {doc.category}
              </button>
            </div>
            <article className={`web-document web-document-${doc.detail.kind}`}>
              <header className="web-document-heading">
                <span className="web-kicker">{doc.category}</span>
                <h1>{doc.title}</h1>
                {!doc.blocks.some(
                  (block) => block.kind === 'paragraph' && block.text === doc.summary,
                ) && <p>{doc.summary}</p>}
                <div className="web-byline">
                  <span className="web-avatar">{doc.author.slice(0, 1)}</span>
                  <span>
                    <b>{doc.author}</b>
                    <small>{date(doc.publishedAt)}</small>
                    {doc.updatedAt && <small>Atualizado em {dateTime(doc.updatedAt)}</small>}
                  </span>
                </div>
              </header>
              {!['thread', 'lesson', 'reference', 'profile', 'socialProfile'].includes(
                doc.detail.kind,
              ) && (
                <div className="web-document-visual">
                  <WebVisual kind={doc.visual} large />
                  {doc.detail.kind === 'video' && (
                    <span className="web-video-label">
                      {Math.floor(doc.detail.seconds / 60)} min · Guia em capítulos
                    </span>
                  )}
                </div>
              )}
              <div className="web-document-detail">
                <CommunityPanel page={page} act={act} navigate={navigate} pending={pending} />
                <Detail
                  page={displayPage}
                  navigate={navigate}
                  act={(a, t) => void act(a, t)}
                  pending={pending || !!page.capture}
                />
              </div>
              <Blocks document={doc} navigate={navigate} />
              {!!doc.links.length && (
                <aside className="web-sources">
                  <h2>{doc.detail.kind === 'product' ? 'Conheça também' : 'Continue por aqui'}</h2>
                  {doc.links.map((l) => (
                    <Link key={l.url} link={l} navigate={navigate} />
                  ))}
                </aside>
              )}
            </article>
            {!page.capture && (
              <Comments key={doc.id} document={doc} act={(a, t) => act(a, t)} pending={pending} />
            )}
            {!!page.related.length && (
              <section className="web-related">
                <h2>Mais em {page.brand.name}</h2>
                <div className="web-grid">
                  {page.related.map((card) => (
                    <Card key={card.id} card={card} navigate={navigate} compact />
                  ))}
                </div>
              </section>
            )}
          </>
        ) : (
          <>
            <section className="web-intro">
              <span className="web-kicker">
                {page.category ||
                  (page.brand.depth === 'LONG_TAIL'
                    ? page.brand.name
                    : {
                        tech: 'IDEIAS. TESTES. DESCOBERTAS.',
                        community: 'A COMUNIDADE ESTÁ CONVERSANDO',
                        shop: 'ESCOLHAS PARA TODO DIA',
                        reference: 'BEM-VINDO À WIPÉDIA',
                        kitchen: 'DA NOSSA COZINHA PARA A SUA',
                        learning: 'SEU PRÓXIMO PASSO',
                        corporate: 'DESENHADO PARA A VIDA REAL',
                        video: 'DESCUBRA UMA NOVA IDEIA',
                        personal: 'ENTRE, O CAFÉ ESTÁ PASSADO',
                        newspaper: 'O QUE ESTÁ ACONTECENDO',
                      }[page.brand.layout])}
              </span>
              <h1>{page.query ? `Resultados para “${page.query}”` : page.brand.tagline}</h1>
              <p>
                {page.brand.depth === 'LONG_TAIL'
                  ? page.brand.voice
                  : page.query
                    ? `${page.total} publicações encontradas`
                    : {
                        tech: 'Análises, guias e perguntas que ajudam a escolher melhor.',
                        community:
                          'Experiências diferentes. Uma pergunta em comum. Encontre seu próximo assunto.',
                        shop: 'Dos pequenos detalhes aos equipamentos que acompanham você.',
                        reference: 'Explore conceitos, descubra relações e siga as referências.',
                        kitchen:
                          'Medidas claras, ingredientes de verdade e espaço para experimentar.',
                        learning: 'Aprenda no seu ritmo, uma aula de cada vez.',
                        corporate:
                          'Conheça nossos produtos, encontre documentação e continue explorando.',
                        video: 'Tecnologia, cultura e cotidiano, com tempo para entender.',
                        personal:
                          page.brand.id === 'blog'
                            ? 'Sou a Lia. Este é meu espaço para guardar descobertas, dúvidas e pequenas histórias.'
                            : page.brand.voice,
                        newspaper: 'Tecnologia, cidades e cultura. Informação com contexto.',
                      }[page.brand.layout]}
              </p>
            </section>
            <div className="web-listing-layout">
              <aside className="web-categories">
                {page.brand.platform === 'CLASSIFIEDS' && (
                  <form
                    className="web-classified-filters"
                    onSubmit={(e) => {
                      e.preventDefault();
                      const data = new FormData(e.currentTarget);
                      const params = new URLSearchParams({ q: query, category: page.category });
                      for (const [key, value] of data) {
                        if (typeof value === 'string') {
                          params.set(key, value);
                        }
                      }
                      navigate(`${home}search?${params}`);
                    }}
                  >
                    <label>
                      Localização
                      <select name="location" defaultValue={page.filters?.location ?? ''}>
                        <option value="">Todas</option>
                        {['Porto Claro', 'Santa Aurora', 'Vila Horizonte'].map((v) => (
                          <option key={v}>{v}</option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Condição
                      <select name="condition" defaultValue={page.filters?.condition ?? ''}>
                        <option value="">Todas</option>
                        <option value="novo">Novo</option>
                        <option value="usado">Usado</option>
                      </select>
                    </label>
                    <label>
                      Situação
                      <select name="status" defaultValue={page.filters?.status ?? ''}>
                        <option value="">Todas</option>
                        <option value="AVAILABLE">Disponível</option>
                        <option value="RESERVED">Reservado</option>
                        <option value="SOLD">Vendido</option>
                      </select>
                    </label>
                    <button className="web-primary">Aplicar filtros</button>
                  </form>
                )}
                <h2>{page.brand.platform === 'FORUM' ? 'Comunidades' : 'Explore'}</h2>
                <button
                  aria-current={!page.category ? 'page' : undefined}
                  onClick={() => navigate(listing(''))}
                >
                  Tudo
                </button>
                {page.categories
                  .filter((c) => c !== 'Início')
                  .map((c) => (
                    <button
                      key={c}
                      aria-current={page.category === c ? 'page' : undefined}
                      onClick={() => navigate(listing(c))}
                    >
                      {c}
                    </button>
                  ))}
              </aside>
              <section className="web-feed" aria-label="Publicações">
                <div className="web-grid">
                  {page.cards
                    .filter((c) => c.url !== home)
                    .map((card) => (
                      <Card key={card.id} card={card} navigate={navigate} />
                    ))}
                </div>
                {page.total === 0 && (
                  <p className="web-empty">
                    Nenhuma publicação corresponde a esta busca. Tente outras palavras.
                  </p>
                )}
                {page.total > 18 && (
                  <nav className="web-pagination" aria-label="Páginas do site">
                    <button
                      disabled={page.offset === 0}
                      onClick={() =>
                        navigate(listing(page.category, Math.max(0, page.offset - 18)))
                      }
                    >
                      ← Anterior
                    </button>
                    <span>
                      {Math.floor(page.offset / 18) + 1} / {Math.ceil(page.total / 18)}
                    </span>
                    <button
                      disabled={page.offset + 18 >= page.total}
                      onClick={() => navigate(listing(page.category, page.offset + 18))}
                    >
                      Próxima →
                    </button>
                  </nav>
                )}
              </section>
            </div>
          </>
        )}
      </main>
      <footer className="web-footer">
        <strong>{page.brand.name}</strong>
        <span>{page.brand.tagline}</span>
        <button onClick={() => navigate(home)}>Voltar ao início ↑</button>
      </footer>
      {import.meta.env.DEV && <WebDevTools address={page.canonicalUrl} navigate={navigate} />}
    </div>
  );
}
