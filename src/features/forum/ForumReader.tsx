import { useRef, useState } from 'react';
import { ForumAvatar, ForumBody, ForumIcon } from './ForumElements';
import { forumDate, memberStats } from './forum-model';
import type { ForumDispatch, ForumMember, ForumPost, ForumState, ForumThread } from './forum-model';

const pageSize = 6;

export function ForumReader({
  thread,
  threads,
  members,
  state,
  nickname,
  onAction,
  onProfile,
  onAuthorSearch,
  onMission,
}: {
  thread: ForumThread;
  threads: ForumThread[];
  members: ForumMember[];
  state: ForumState;
  nickname: string;
  onAction: ForumDispatch;
  onProfile: (name: string) => void;
  onAuthorSearch: (name: string) => void;
  onMission: () => void;
}) {
  const [page, setPage] = useState(1);
  const [draft, setDraft] = useState('');
  const [quote, setQuote] = useState<ForumPost | null>(null);
  const [preview, setPreview] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [reportPost, setReportPost] = useState<string | null>(null);
  const [reason, setReason] = useState('');
  const replyInput = useRef<HTMLTextAreaElement>(null);
  const pageCount = Math.max(1, Math.ceil(thread.posts.length / pageSize));
  const followed = state.followed.includes(thread.id);
  const act = async (action: Parameters<ForumDispatch>[0], done?: () => void) => {
    if (pending) {
      return;
    }
    setPending(true);
    setError('');
    setNotice('');
    try {
      await onAction(action);
      done?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setPending(false);
    }
  };
  const reply = (post?: ForumPost) => {
    setQuote(post ?? null);
    setPreview(false);
    replyInput.current?.focus();
    replyInput.current?.scrollIntoView?.({ block: 'center', behavior: 'smooth' });
  };
  const pagination = (
    <div className="fb-pagination" aria-label="Páginas de respostas">
      <span>
        Página {page} de {pageCount}
      </span>
      {Array.from({ length: pageCount }, (_, index) => (
        <button
          key={index}
          aria-label={`Página ${index + 1}`}
          aria-current={page === index + 1 ? 'page' : undefined}
          onClick={() => setPage(index + 1)}
        >
          {index + 1}
        </button>
      ))}
    </div>
  );
  return (
    <>
      <div className="fb-page-tools">
        {pagination}
        <div>
          <button
            disabled={pending}
            aria-pressed={followed}
            onClick={() => void act({ kind: 'toggleFollow', threadId: thread.id })}
          >
            <ForumIcon name="star" />
            {followed ? 'Acompanhando' : 'Acompanhar'}
          </button>
          {!thread.locked && (
            <button className="fb-primary" onClick={() => reply()}>
              <ForumIcon name="reply" />
              Responder
            </button>
          )}
        </div>
      </div>
      <header className="fb-thread-heading">
        <h1>{thread.title}</h1>
        <p>
          por <button onClick={() => onProfile(thread.author)}>{thread.author}</button> ·{' '}
          {forumDate(thread.createdAt, true)}
        </p>
      </header>
      {thread.posts.slice((page - 1) * pageSize, page * pageSize).map((post, index) => {
        const member =
          members.find((item) => item.name === post.author) ?? members[members.length - 1];
        const stats = memberStats(post.author, threads);
        const reported = state.reports.some((item) => item.postId === post.id);
        return (
          <article className="fb-post" key={post.id} aria-label={`Mensagem de ${post.author}`}>
            <aside className="fb-post-author">
              <button
                className={`fb-author-name fb-color-${member.color}`}
                onClick={() => onProfile(post.author)}
              >
                {member.rank === 'Administrador' && <span aria-hidden="true">♛ </span>}
                {post.author}
              </button>
              <button
                className="fb-avatar-button"
                aria-label={`Ver perfil de ${post.author}`}
                onClick={() => onProfile(post.author)}
              >
                <ForumAvatar member={member} />
              </button>
              <div className="fb-user-caption">
                {post.author === nickname ? 'Você' : member.signature}
                <span className={post.author === nickname ? 'fb-online-dot' : 'fb-offline-dot'} />
              </div>
              <div className={`fb-rank fb-color-${member.color}`}>{member.rank}</div>
              <dl className="fb-user-stats">
                <div>
                  <dt>Mensagens:</dt>
                  <dd>{stats.posts}</dd>
                </div>
                <div>
                  <dt>Tópicos:</dt>
                  <dd>{stats.threads}</dd>
                </div>
                <div>
                  <dt>Registro:</dt>
                  <dd>
                    {new Date(`${member.joined}T12:00:00Z`).toLocaleDateString('pt-BR', {
                      month: 'short',
                      year: 'numeric',
                      timeZone: 'UTC',
                    })}
                  </dd>
                </div>
                <div>
                  <dt>Reputação:</dt>
                  <dd className="fb-reputation">{member.reputation}</dd>
                </div>
              </dl>
            </aside>
            <div className="fb-post-content">
              <div className="fb-post-meta">
                <time>{forumDate(post.createdAt, true)}</time>
                <span>#{(page - 1) * pageSize + index + 1}</span>
              </div>
              {post.quote && (
                <blockquote className="fb-quote">
                  <strong>{post.quote.author} escreveu:</strong>
                  <ForumBody text={post.quote.text} />
                </blockquote>
              )}
              <ForumBody text={post.text} />
              {thread.id === 'server' && post.id.endsWith(':op') && (
                <button className="fb-mission-link" onClick={onMission}>
                  <ForumIcon name="briefcase" />
                  Ver pedido de VEX em Trabalhos
                  <ForumIcon name="arrow" />
                </button>
              )}
              <div className="fb-signature">{member.signature}</div>
              <footer className="fb-post-actions">
                <div>
                  <button onClick={() => onProfile(post.author)}>
                    <ForumIcon name="members" />
                    Perfil
                  </button>
                  <button onClick={() => onAuthorSearch(post.author)}>
                    <ForumIcon name="search" />
                    Posts
                  </button>
                </div>
                <div>
                  {!thread.locked && (
                    <>
                      <button onClick={() => reply()}>
                        <ForumIcon name="reply" />
                        Responder
                      </button>
                      <button onClick={() => reply(post)}>
                        <ForumIcon name="quote" />
                        Citar
                      </button>
                    </>
                  )}
                  {post.author !== nickname && (
                    <button
                      disabled={reported}
                      onClick={() => {
                        setReportPost(reportPost === post.id ? null : post.id);
                        setReason('');
                      }}
                    >
                      <ForumIcon name="flag" />
                      {reported ? 'Sinalizado' : 'Sinalizar'}
                    </button>
                  )}
                </div>
              </footer>
              {reportPost === post.id && (
                <form
                  className="fb-report-form"
                  onSubmit={(event) => {
                    event.preventDefault();
                    void act(
                      { kind: 'report', threadId: thread.id, postId: post.id, reason },
                      () => {
                        setReportPost(null);
                        setReason('');
                        setNotice('Sinalização registrada neste tópico.');
                      },
                    );
                  }}
                >
                  <label>
                    Motivo da sinalização
                    <input
                      autoFocus
                      value={reason}
                      onChange={(event) => setReason(event.target.value)}
                      required
                      minLength={4}
                      maxLength={500}
                    />
                  </label>
                  <button type="button" onClick={() => setReportPost(null)}>
                    Cancelar
                  </button>
                  <button disabled={pending || reason.trim().length < 4}>Registrar</button>
                </form>
              )}
            </div>
          </article>
        );
      })}
      <div className="fb-page-tools">
        {pagination}
        <span>{thread.posts.length} mensagens neste tópico</span>
      </div>
      {error && (
        <p className="fb-notice fb-error" role="alert">
          {error}
        </p>
      )}
      {notice && (
        <p className="fb-notice" role="status">
          {notice}
        </p>
      )}
      {thread.locked ? (
        <div className="fb-notice">
          <ForumIcon name="lock" />
          Este tópico está fechado para novas respostas.
        </div>
      ) : (
        <section className="fb-panel fb-reply-panel">
          <h2 className="fb-section-title">
            <ForumIcon name="reply" />
            Resposta rápida
          </h2>
          <form
            className="fb-editor-form"
            onSubmit={(event) => {
              event.preventDefault();
              void act(
                { kind: 'reply', threadId: thread.id, body: draft, quoteId: quote?.id ?? null },
                () => {
                  setDraft('');
                  setQuote(null);
                  setPreview(false);
                  setPage(Math.ceil((thread.posts.length + 1) / pageSize));
                  setNotice('Resposta publicada.');
                },
              );
            }}
          >
            <div className="fb-editor-caption">
              Postando como <strong>{nickname}</strong>
              <span>**negrito** · blocos com ```código```</span>
            </div>
            {quote && (
              <blockquote className="fb-quote">
                <button type="button" aria-label="Remover citação" onClick={() => setQuote(null)}>
                  ×
                </button>
                <strong>Citando {quote.author}</strong>
                <p>
                  {quote.text.slice(0, 240)}
                  {quote.text.length > 240 ? '…' : ''}
                </p>
              </blockquote>
            )}
            <label className={preview ? 'fb-sr-only' : ''}>
              Sua resposta
              <textarea
                ref={replyInput}
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                required
                minLength={2}
                maxLength={12000}
                rows={6}
                placeholder="Participe da conversa…"
              />
            </label>
            {preview && (
              <div className="fb-editor-preview">
                <ForumBody text={draft || 'A prévia aparecerá aqui.'} />
              </div>
            )}
            <div className="fb-editor-actions">
              <button type="button" aria-pressed={preview} onClick={() => setPreview(!preview)}>
                {preview ? 'Voltar à edição' : 'Visualizar resposta'}
              </button>
              <button className="fb-primary" disabled={pending || draft.trim().length < 2}>
                {pending ? 'Publicando…' : 'Publicar resposta'}
              </button>
            </div>
          </form>
        </section>
      )}
    </>
  );
}
