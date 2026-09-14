import { useMemo, useRef, useState } from 'react';
import { ForumAvatar, ForumBody, ForumIcon } from './ForumElements';
import { ForumReader } from './ForumReader';
import {
  boardMembers,
  boardThreads,
  forumCategories,
  forumDate,
  forumGroups,
  memberStats,
  searchThreads,
} from './forum-model';
import type { ForumDispatch, ForumState, ForumThread } from './forum-model';
import './forum.css';

type Route =
  | { kind: 'home' | 'members' | 'rules' }
  | { kind: 'category'; id: string }
  | { kind: 'thread'; id: string }
  | { kind: 'profile'; name: string }
  | { kind: 'search'; query: string; author?: string; followed?: boolean; unread?: boolean }
  | { kind: 'compose'; category: string };

export function ForumBoard({
  nickname,
  reputation,
  flags,
  connected,
  state,
  onAction,
  onMission,
}: {
  nickname: string;
  reputation: number;
  flags: string[];
  connected: boolean;
  state: ForumState;
  onAction: ForumDispatch;
  onMission: () => void;
}) {
  const [history, setHistory] = useState<Route[]>([{ kind: 'home' }]);
  const [group, setGroup] = useState('all');
  const [query, setQuery] = useState('');
  const [sort, setSort] = useState('activity');
  const [notice, setNotice] = useState('');
  const [pending, setPending] = useState(false);
  const scroll = useRef<HTMLDivElement>(null);
  const route = history[history.length - 1];
  const threads = useMemo(() => boardThreads(state, flags), [state, flags]);
  const members = useMemo(() => boardMembers(nickname, reputation), [nickname, reputation]);
  const player = members[members.length - 1];
  const activeThread =
    route.kind === 'thread' ? threads.find((thread) => thread.id === route.id) : undefined;
  const activeCategory =
    route.kind === 'category' || route.kind === 'compose'
      ? forumCategories.find(
          (category) => category.id === (route.kind === 'category' ? route.id : route.category),
        )
      : activeThread
        ? forumCategories.find((category) => category.id === activeThread.category)
        : undefined;
  const unread = (thread: ForumThread) => (state.readCounts[thread.id] ?? 0) < thread.posts.length;
  const go = (next: Route) => {
    setHistory((items) => [...items.slice(-29), next]);
    setNotice('');
    if (scroll.current) {
      scroll.current.scrollTop = 0;
    }
    if (next.kind === 'thread') {
      const thread = threads.find((item) => item.id === next.id);
      if (thread && unread(thread)) {
        void onAction({ kind: 'markRead', threadId: next.id }).catch((e: unknown) =>
          setNotice(String(e)),
        );
      }
    }
  };
  const profile = (name: string) => go({ kind: 'profile', name });
  const authorSearch = (author: string) => go({ kind: 'search', query: '', author });
  const totalPosts = threads.reduce((count, thread) => count + thread.posts.length, 0);
  const memberFor = (name: string) => members.find((member) => member.name === name) ?? player;
  const title =
    route.kind === 'home'
      ? 'Início'
      : route.kind === 'members'
        ? 'Membros'
        : route.kind === 'rules'
          ? 'Regras da comunidade'
          : route.kind === 'profile'
            ? route.name
            : route.kind === 'compose'
              ? 'Novo tópico'
              : route.kind === 'search'
                ? 'Pesquisa'
                : (activeThread?.title ?? activeCategory?.name ?? 'Tópico indisponível');
  const threadRows = (items: ForumThread[]) => {
    const sorted = [...items].sort(
      (a, b) =>
        Number(Boolean(b.pinned)) - Number(Boolean(a.pinned)) ||
        (sort === 'replies'
          ? b.posts.length - a.posts.length
          : sort === 'title'
            ? a.title.localeCompare(b.title, 'pt-BR')
            : b.updatedAt.localeCompare(a.updatedAt)),
    );
    return (
      <div className="fb-thread-list">
        <div className="fb-list-labels">
          <span>Tópico / autor</span>
          <span>Respostas</span>
          <span>Última mensagem</span>
        </div>
        {sorted.map((thread) => (
          <div className="fb-thread-row" key={thread.id}>
            <span
              className={`fb-topic-symbol ${unread(thread) ? 'fb-is-unread' : ''}`}
              title={unread(thread) ? 'Mensagens não lidas' : 'Lido'}
            >
              <ForumIcon name={thread.locked ? 'lock' : thread.pinned ? 'pin' : 'chat'} size={22} />
            </span>
            <div className="fb-thread-summary">
              <button
                className="fb-thread-link"
                onClick={() => go({ kind: 'thread', id: thread.id })}
              >
                {thread.tag && (
                  <span className={`fb-tag fb-tag-${thread.tag.toLowerCase()}`}>{thread.tag}</span>
                )}
                {thread.title}
              </button>
              <small>
                <button
                  className={`fb-color-${memberFor(thread.author).color}`}
                  onClick={() => profile(thread.author)}
                >
                  {thread.author}
                </button>{' '}
                · {forumDate(thread.createdAt)}
                {state.followed.includes(thread.id) && <ForumIcon name="star" size={12} />}
              </small>
            </div>
            <div className="fb-count">
              {thread.posts.length - 1}
              <small>respostas</small>
            </div>
            <div className="fb-last-post">
              <button onClick={() => go({ kind: 'thread', id: thread.id })}>
                {forumDate(thread.updatedAt)}
              </button>
              <small>
                por{' '}
                <button
                  className={`fb-color-${memberFor(thread.posts.at(-1)?.author ?? thread.author).color}`}
                  onClick={() => profile(thread.posts.at(-1)?.author ?? thread.author)}
                >
                  {thread.posts.at(-1)?.author}
                </button>
              </small>
            </div>
          </div>
        ))}
        {sorted.length === 0 && (
          <div className="fb-empty">
            <ForumIcon name="chat" size={28} />
            <h3>Nenhum tópico por aqui ainda.</h3>
            <p>Experimente outra pesquisa ou comece uma conversa nesta seção.</p>
          </div>
        )}
      </div>
    );
  };
  return (
    <div className="forum-board">
      <nav className="fb-topbar" aria-label="Navegação do fórum">
        <div>
          <button
            className={route.kind === 'home' ? 'fb-nav-current' : ''}
            aria-label="Página inicial do fórum"
            onClick={() => go({ kind: 'home' })}
          >
            <ForumIcon name="home" />
          </button>
          <button onClick={() => go({ kind: 'home' })}>Fóruns</button>
          <button onClick={() => go({ kind: 'members' })}>
            <ForumIcon name="members" />
            Membros
          </button>
          <button
            onClick={() => {
              setQuery('');
              go({ kind: 'search', query: '' });
            }}
          >
            <ForumIcon name="search" />
            Buscar
          </button>
          <button onClick={() => go({ kind: 'rules' })}>
            <ForumIcon name="document" />
            Regras
          </button>
        </div>
        <button className="fb-account" onClick={() => profile(nickname)}>
          <span className="fb-online-dot" />
          {nickname}
          <span aria-hidden="true">▾</span>
        </button>
      </nav>
      <div className="fb-scroll" ref={scroll}>
        <div className="fb-content">
          <header className="fb-masthead">
            <button aria-label="Terminal Board — início" onClick={() => go({ kind: 'home' })}>
              <span className="fb-wordmark-symbol">&gt;_</span>
              <span>
                terminal<span className="fb-wordmark-light">board</span>
                <small>CONHECIMENTO COMPARTILHADO.</small>
              </span>
            </button>
            <span className="fb-community-label">
              <span className="fb-online-dot" />
              Conexões que vão além do código.
            </span>
          </header>
          <div className="fb-breadcrumb">
            <div>
              <button
                aria-label="Voltar no fórum"
                disabled={history.length === 1}
                onClick={() => {
                  setHistory((items) => items.slice(0, -1));
                  setNotice('');
                }}
              >
                <ForumIcon name="back" size={13} />
              </button>
              <button onClick={() => go({ kind: 'home' })}>Terminal Board</button>
              {activeCategory && route.kind !== 'category' && (
                <>
                  <ForumIcon name="arrow" size={11} />
                  <button onClick={() => go({ kind: 'category', id: activeCategory.id })}>
                    {activeCategory.name}
                  </button>
                </>
              )}
              {route.kind !== 'home' && (
                <>
                  <ForumIcon name="arrow" size={11} />
                  <span>{title}</span>
                </>
              )}
            </div>
            <button
              onClick={() => {
                setQuery('');
                go({ kind: 'search', query: '', unread: true });
              }}
            >
              Não lidos <b>{threads.filter(unread).length}</b>
            </button>
          </div>
          {!connected ? (
            <div className="fb-empty fb-offline">
              <ForumIcon name="network" size={38} />
              <h1>Sem conexão com o fórum</h1>
              <p>Conecte-se a uma rede pelo sistema para voltar às conversas.</p>
              <small>terminalboard.net · conexão indisponível</small>
            </div>
          ) : (
            <>
              {notice && (
                <div className="fb-notice" role="status">
                  {notice}
                </div>
              )}
              {route.kind === 'home' && (
                <>
                  <div className="fb-tabs" role="tablist" aria-label="Seções do fórum">
                    {[{ id: 'all', name: 'Todos' }, ...forumGroups].map((item) => (
                      <button
                        role="tab"
                        aria-selected={group === item.id}
                        key={item.id}
                        onClick={() => setGroup(item.id)}
                      >
                        {item.id === 'all' && <ForumIcon name="home" size={12} />}
                        {item.name}
                      </button>
                    ))}
                  </div>
                  <div className="fb-home-columns">
                    <main className="fb-forums">
                      {forumGroups
                        .filter((item) => group === 'all' || group === item.id)
                        .map((item) => (
                          <section className="fb-panel" key={item.id}>
                            <h2 className="fb-section-title">
                              <ForumIcon name={item.id === 'general' ? 'home' : 'chat'} size={14} />
                              {item.name}
                              <span>−</span>
                            </h2>
                            {forumCategories
                              .filter((category) => category.group === item.id)
                              .map((category) => {
                                const items = threads.filter(
                                  (thread) => thread.category === category.id,
                                );
                                const last = items[0];
                                return (
                                  <div className="fb-category-row" key={category.id}>
                                    <button
                                      className={`fb-category-icon ${items.some(unread) ? 'fb-is-unread' : ''}`}
                                      aria-label={`Abrir ${category.name}`}
                                      onClick={() => go({ kind: 'category', id: category.id })}
                                    >
                                      <ForumIcon name={category.icon} size={23} />
                                    </button>
                                    <div className="fb-category-description">
                                      <button
                                        onClick={() => go({ kind: 'category', id: category.id })}
                                      >
                                        {category.name}
                                      </button>
                                      <p>{category.description}</p>
                                      <small>
                                        {category.readOnly
                                          ? '› Regras e avisos da equipe'
                                          : `› ${items.length} tópicos nesta seção`}
                                      </small>
                                    </div>
                                    <div className="fb-category-counts">
                                      <span>
                                        {items.length}
                                        <small>Tópicos</small>
                                      </span>
                                      <span>
                                        {items.reduce(
                                          (sum, thread) => sum + thread.posts.length,
                                          0,
                                        )}
                                        <small>Mensagens</small>
                                      </span>
                                    </div>
                                    <div className="fb-category-latest">
                                      {last ? (
                                        <>
                                          <button
                                            onClick={() => go({ kind: 'thread', id: last.id })}
                                          >
                                            {last.title}
                                          </button>
                                          <small>
                                            por{' '}
                                            <button
                                              className={`fb-color-${memberFor(last.posts.at(-1)?.author ?? last.author).color}`}
                                              onClick={() =>
                                                profile(last.posts.at(-1)?.author ?? last.author)
                                              }
                                            >
                                              {last.posts.at(-1)?.author}
                                            </button>
                                          </small>
                                          <time>{forumDate(last.updatedAt)}</time>
                                        </>
                                      ) : (
                                        <small>Nenhuma mensagem</small>
                                      )}
                                    </div>
                                  </div>
                                );
                              })}
                          </section>
                        ))}
                      <div className="fb-page-tools fb-board-legend">
                        <span>
                          <ForumIcon name="chat" />
                          Novas mensagens <span className="fb-read-dot" />
                          Sem novidades
                        </span>
                        <button
                          disabled={pending}
                          onClick={() => {
                            setPending(true);
                            void onAction({ kind: 'markAllRead' })
                              .then(() =>
                                setNotice(
                                  'Todos os tópicos disponíveis foram marcados como lidos.',
                                ),
                              )
                              .catch((e: unknown) => setNotice(String(e)))
                              .finally(() => setPending(false));
                          }}
                        >
                          <ForumIcon name="check" />
                          Marcar tudo como lido
                        </button>
                      </div>
                    </main>
                    <aside className="fb-sidebar">
                      <section className="fb-panel">
                        <h2 className="fb-section-title">
                          <ForumIcon name="chat" size={14} />
                          Últimas mensagens
                        </h2>
                        <div className="fb-latest-list">
                          {threads.slice(0, 6).map((thread) => (
                            <article key={thread.id}>
                              <button
                                className="fb-latest-title"
                                onClick={() => go({ kind: 'thread', id: thread.id })}
                              >
                                {thread.tag && (
                                  <span className={`fb-tag fb-tag-${thread.tag.toLowerCase()}`}>
                                    {thread.tag}
                                  </span>
                                )}
                                {thread.title}
                              </button>
                              <small>
                                <button
                                  className={`fb-color-${memberFor(thread.posts.at(-1)?.author ?? thread.author).color}`}
                                  onClick={() =>
                                    profile(thread.posts.at(-1)?.author ?? thread.author)
                                  }
                                >
                                  {thread.posts.at(-1)?.author}
                                </button>{' '}
                                · {forumDate(thread.updatedAt)}
                              </small>
                            </article>
                          ))}
                        </div>
                      </section>
                      <section className="fb-panel">
                        <h2 className="fb-section-title">
                          <ForumIcon name="network" size={14} />
                          Estatísticas
                        </h2>
                        <dl className="fb-sidebar-stats">
                          <div>
                            <dt>Tópicos</dt>
                            <dd>{threads.length}</dd>
                          </div>
                          <div>
                            <dt>Mensagens</dt>
                            <dd>{totalPosts}</dd>
                          </div>
                          <div>
                            <dt>Membros</dt>
                            <dd>{members.length}</dd>
                          </div>
                          <div>
                            <dt>Acompanhando</dt>
                            <dd>
                              <button
                                onClick={() => go({ kind: 'search', query: '', followed: true })}
                              >
                                {state.followed.length}
                              </button>
                            </dd>
                          </div>
                        </dl>
                      </section>
                      <section className="fb-panel">
                        <h2 className="fb-section-title">
                          <ForumIcon name="members" size={14} />
                          Sua sessão
                        </h2>
                        <div className="fb-session">
                          <ForumAvatar member={player} small />
                          <div>
                            <button className="fb-color-teal" onClick={() => profile(nickname)}>
                              {nickname}
                            </button>
                            <small>
                              <span className="fb-online-dot" /> Online
                            </small>
                          </div>
                        </div>
                        <button
                          className="fb-sidebar-action"
                          onClick={() => go({ kind: 'compose', category: 'lounge' })}
                        >
                          <ForumIcon name="plus" />
                          Começar uma conversa
                        </button>
                      </section>
                    </aside>
                  </div>
                </>
              )}
              {route.kind === 'category' && activeCategory && (
                <>
                  <div className="fb-page-heading">
                    <div>
                      <h1>{activeCategory.name}</h1>
                      <p>{activeCategory.description}</p>
                    </div>
                    {!activeCategory.readOnly && (
                      <button
                        className="fb-primary"
                        onClick={() => go({ kind: 'compose', category: activeCategory.id })}
                      >
                        <ForumIcon name="plus" />
                        Novo tópico
                      </button>
                    )}
                  </div>
                  <div className="fb-panel">
                    <h2 className="fb-section-title">
                      <ForumIcon name={activeCategory.icon} />
                      Tópicos em {activeCategory.name}
                    </h2>
                    {threadRows(threads.filter((thread) => thread.category === activeCategory.id))}
                  </div>
                  <div className="fb-page-tools">
                    <span>
                      {threads.filter((thread) => thread.category === activeCategory.id).length}{' '}
                      tópicos
                    </span>
                    <label className="fb-inline-label">
                      Ordenar por
                      <select value={sort} onChange={(event) => setSort(event.target.value)}>
                        <option value="activity">Última atividade</option>
                        <option value="replies">Mais respostas</option>
                        <option value="title">Título</option>
                      </select>
                    </label>
                  </div>
                </>
              )}
              {route.kind === 'thread' &&
                (activeThread ? (
                  <ForumReader
                    key={activeThread.id}
                    thread={activeThread}
                    threads={threads}
                    members={members}
                    state={state}
                    nickname={nickname}
                    onAction={onAction}
                    onProfile={profile}
                    onAuthorSearch={authorSearch}
                    onMission={onMission}
                  />
                ) : (
                  <div className="fb-empty">
                    <h1>Tópico indisponível</h1>
                    <p>Esta conversa ainda não está disponível na sua campanha.</p>
                    <button onClick={() => go({ kind: 'home' })}>Voltar ao início</button>
                  </div>
                ))}
              {route.kind === 'search' && (
                <>
                  <div className="fb-page-heading">
                    <div>
                      <h1>
                        {route.followed
                          ? 'Tópicos acompanhados'
                          : route.unread
                            ? 'Mensagens não lidas'
                            : route.author
                              ? `Mensagens de ${route.author}`
                              : 'Pesquisar na comunidade'}
                      </h1>
                      <p>Encontre conversas pelo título, autor ou conteúdo das mensagens.</p>
                    </div>
                  </div>
                  <form
                    className="fb-search-form"
                    onSubmit={(event) => {
                      event.preventDefault();
                      go({ kind: 'search', query });
                    }}
                  >
                    <label>
                      <span className="fb-sr-only">Pesquisar no fórum</span>
                      <input
                        autoFocus
                        aria-label="Pesquisar no fórum"
                        value={query}
                        onChange={(event) => setQuery(event.target.value)}
                        placeholder="O que você está procurando?"
                      />
                    </label>
                    <button className="fb-primary">
                      <ForumIcon name="search" />
                      Pesquisar
                    </button>
                  </form>
                  <div className="fb-panel">
                    <h2 className="fb-section-title">
                      <ForumIcon name="search" />
                      {route.query ? `Resultados para “${route.query}”` : 'Tópicos'}
                    </h2>
                    {threadRows(
                      searchThreads(threads, route.query, route.author).filter(
                        (thread) =>
                          (!route.followed || state.followed.includes(thread.id)) &&
                          (!route.unread || unread(thread)),
                      ),
                    )}
                  </div>
                </>
              )}
              {route.kind === 'members' && (
                <>
                  <div className="fb-page-heading">
                    <div>
                      <h1>Membros da comunidade</h1>
                      <p>Quem escreve, compartilha e ajuda a manter o fórum vivo.</p>
                    </div>
                    <span>{members.length} membros</span>
                  </div>
                  <div className="fb-panel">
                    <h2 className="fb-section-title">
                      <ForumIcon name="members" />
                      Diretório de membros
                    </h2>
                    {members.map((member) => {
                      const stats = memberStats(member.name, threads);
                      return (
                        <div className="fb-member-row" key={member.name}>
                          <button
                            className="fb-avatar-button"
                            aria-label={`Ver perfil de ${member.name}`}
                            onClick={() => profile(member.name)}
                          >
                            <ForumAvatar member={member} small />
                          </button>
                          <div>
                            <button
                              className={`fb-member-name fb-color-${member.color}`}
                              onClick={() => profile(member.name)}
                            >
                              {member.name}
                            </button>
                            <p>{member.bio}</p>
                            <small>{member.rank}</small>
                          </div>
                          <div className="fb-count">
                            {stats.posts}
                            <small>mensagens</small>
                          </div>
                          <div className="fb-count fb-reputation">
                            {member.reputation}
                            <small>reputação</small>
                          </div>
                          <button
                            aria-label={`Abrir perfil de ${member.name}`}
                            onClick={() => profile(member.name)}
                          >
                            <ForumIcon name="arrow" />
                          </button>
                        </div>
                      );
                    })}
                  </div>
                </>
              )}
              {route.kind === 'profile' &&
                (() => {
                  const member = memberFor(route.name);
                  const stats = memberStats(member.name, threads);
                  return (
                    <>
                      <div className="fb-panel">
                        <h1 className="fb-section-title">
                          <ForumIcon name="members" />
                          Perfil de {member.name}
                        </h1>
                        <div className="fb-profile">
                          <ForumAvatar member={member} />
                          <div>
                            <h2 className={`fb-color-${member.color}`}>
                              {member.name}
                              {member.name === nickname && (
                                <span className="fb-you-label">Você</span>
                              )}
                            </h2>
                            <div className={`fb-rank fb-color-${member.color}`}>{member.rank}</div>
                            <p>{member.bio}</p>
                            <blockquote>{member.signature}</blockquote>
                            <button onClick={() => authorSearch(member.name)}>
                              <ForumIcon name="search" />
                              Encontrar mensagens
                            </button>
                          </div>
                          <dl className="fb-user-stats">
                            <div>
                              <dt>Registro</dt>
                              <dd>
                                {new Date(`${member.joined}T12:00:00Z`).toLocaleDateString(
                                  'pt-BR',
                                  { timeZone: 'UTC' },
                                )}
                              </dd>
                            </div>
                            <div>
                              <dt>Tópicos</dt>
                              <dd>{stats.threads}</dd>
                            </div>
                            <div>
                              <dt>Mensagens</dt>
                              <dd>{stats.posts}</dd>
                            </div>
                            <div>
                              <dt>Reputação</dt>
                              <dd className="fb-reputation">{member.reputation}</dd>
                            </div>
                          </dl>
                        </div>
                      </div>
                      <div className="fb-panel">
                        <h2 className="fb-section-title">
                          <ForumIcon name="chat" />
                          Conversas de {member.name}
                        </h2>
                        {threadRows(
                          threads.filter((thread) =>
                            thread.posts.some((post) => post.author === member.name),
                          ),
                        )}
                      </div>
                    </>
                  );
                })()}
              {route.kind === 'compose' && (
                <ForumComposer
                  key={route.category}
                  category={route.category}
                  nickname={nickname}
                  onAction={onAction}
                  onCancel={() => go({ kind: 'category', id: route.category })}
                  onCreated={(id) => go({ kind: 'thread', id })}
                />
              )}
              {route.kind === 'rules' && (
                <section className="fb-panel">
                  <h1 className="fb-section-title">
                    <ForumIcon name="document" />
                    Regras da comunidade
                  </h1>
                  <div className="fb-rules">
                    <h2>Antes de participar</h2>
                    <p>
                      O Terminal Board reúne pessoas com experiências diferentes. Um bom tópico
                      ajuda quem está lendo agora e quem encontrar a conversa depois.
                    </p>
                    <ol>
                      <li>
                        <strong>Respeite os outros membros.</strong> Sem ataques pessoais, assédio
                        ou exposição de dados privados.
                      </li>
                      <li>
                        <strong>Escolha a seção certa.</strong> Pesquise antes de publicar e dê um
                        título claro à sua conversa.
                      </li>
                      <li>
                        <strong>Compartilhe contexto.</strong> Explique o problema, as tentativas e
                        o resultado. Use blocos de código para logs e comandos.
                      </li>
                      <li>
                        <strong>Preserve os créditos e os originais.</strong> Identifique a autoria
                        e trabalhe com cópias quando estiver investigando um arquivo.
                      </li>
                      <li>
                        <strong>Respeite o escopo dos laboratórios.</strong> Não publique
                        credenciais nem informações pessoais. Trabalhos devem ter autorização de
                        acesso.
                      </li>
                      <li>
                        <strong>Sinalize problemas.</strong> Use o botão Sinalizar na mensagem e
                        descreva o motivo.
                      </li>
                    </ol>
                    <button onClick={() => go({ kind: 'thread', id: 'welcome' })}>
                      Ler o anúncio de boas-vindas
                      <ForumIcon name="arrow" />
                    </button>
                  </div>
                </section>
              )}
            </>
          )}
          <footer className="fb-footer">
            <div>
              <strong>Terminal Board</strong>
              <span>Conhecimento se constrói em comunidade.</span>
            </div>
            <div>
              <button onClick={() => go({ kind: 'rules' })}>Regras</button>
              <button onClick={() => go({ kind: 'members' })}>Equipe e membros</button>
              <button
                onClick={() => {
                  if (scroll.current) {
                    scroll.current.scrollTop = 0;
                  }
                }}
              >
                Voltar ao topo ↑
              </button>
            </div>
          </footer>
        </div>
      </div>
    </div>
  );
}

function ForumComposer({
  category,
  nickname,
  onAction,
  onCreated,
  onCancel,
}: {
  category: string;
  nickname: string;
  onAction: ForumDispatch;
  onCreated: (id: string) => void;
  onCancel: () => void;
}) {
  const [selected, setSelected] = useState(category);
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [preview, setPreview] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  return (
    <section className="fb-panel">
      <h1 className="fb-section-title">
        <ForumIcon name="plus" />
        Criar novo tópico
      </h1>
      <form
        className="fb-editor-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (pending) {
            return;
          }
          setPending(true);
          setError('');
          void onAction({ kind: 'createThread', category: selected, title, body })
            .then((id) => {
              if (id) {
                onCreated(id);
              }
            })
            .catch((e: unknown) => setError(String(e)))
            .finally(() => setPending(false));
        }}
      >
        <div className="fb-editor-caption">
          Postando como <strong>{nickname}</strong>
          <span>Um título claro faz toda a diferença.</span>
        </div>
        <label>
          Seção
          <select value={selected} onChange={(event) => setSelected(event.target.value)}>
            {forumCategories
              .filter((item) => !item.readOnly)
              .map((item) => (
                <option value={item.id} key={item.id}>
                  {item.name}
                </option>
              ))}
          </select>
        </label>
        <label>
          Título do tópico
          <input
            autoFocus
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            required
            minLength={5}
            maxLength={120}
            placeholder="Sobre o que você quer conversar?"
          />
        </label>
        <label className={preview ? 'fb-sr-only' : ''}>
          Mensagem
          <textarea
            value={body}
            onChange={(event) => setBody(event.target.value)}
            required
            minLength={10}
            maxLength={12000}
            rows={12}
            placeholder="Compartilhe uma ideia, explique sua dúvida ou mostre o que descobriu…"
          />
        </label>
        {preview && (
          <div className="fb-editor-preview">
            <h2>{title || 'Título do tópico'}</h2>
            <ForumBody text={body || 'Sua mensagem aparecerá aqui.'} />
          </div>
        )}
        <small>
          Formatação: **negrito** e blocos de código entre três crases. Até 12.000 caracteres.
        </small>
        {error && (
          <div className="fb-notice fb-error" role="alert">
            {error}
          </div>
        )}
        <div className="fb-editor-actions">
          <div>
            <button type="button" onClick={onCancel}>
              Cancelar
            </button>
            <button type="button" aria-pressed={preview} onClick={() => setPreview(!preview)}>
              {preview ? 'Voltar à edição' : 'Visualizar tópico'}
            </button>
          </div>
          <button
            className="fb-primary"
            disabled={pending || title.trim().length < 5 || body.trim().length < 10}
          >
            {pending ? 'Publicando…' : 'Publicar tópico'}
          </button>
        </div>
      </form>
    </section>
  );
}
