import { useState } from 'react';
import { dateTime, money, type WebPage } from './web-model';

type Action = (action: string, text?: string, id?: string) => Promise<boolean>;
export function CommunityPanel({
  page,
  act,
  navigate,
  pending,
}: {
  page: WebPage;
  act: Action;
  navigate: (url: string) => void;
  pending: boolean;
}) {
  const [draft, setDraft] = useState('');
  const [message, setMessage] = useState('');
  const [offer, setOffer] = useState('');
  const community = page.community;
  if (!community || page.status !== 200) {
    return null;
  }
  const doc = page.document;
  const detail = doc?.detail;
  const social = page.brand.platform === 'SOCIAL';
  const profile = detail?.kind === 'socialProfile' ? detail : null;
  const listing = detail?.kind === 'listing' ? detail : null;
  const home = !doc || doc.path === '/';
  if (!home && !profile && !listing && !community.messages.length) {
    return null;
  }
  const submit = async (action: string, text: string, clear: () => void, id?: string) => {
    if (await act(action, text, id)) {
      clear();
    }
  };
  return (
    <section className="web-community-panel" aria-label={social ? 'Sua rede' : 'Negociação'}>
      {profile && (
        <section className="web-social-profile">
          <div className="web-person-badge" aria-hidden="true">
            {doc?.title
              .split(' ')
              .map((s) => s[0])
              .slice(0, 2)
              .join('')}
          </div>
          <div>
            <h2>Sobre</h2>
            <p>
              {profile.role} · {profile.location}
            </p>
            <p>{profile.bio}</p>
            <button className="web-link" onClick={() => navigate(profile.employer.url)}>
              {profile.employer.label}
            </button>
            <p>
              {community.followers} seguidores · {profile.following.length} conexões
            </p>
            {!!community.connections?.length && (
              <details>
                <summary>Conexões</summary>
                {community.connections.map((c) => (
                  <p key={c.id}>
                    <button className="web-link" onClick={() => navigate(c.url)}>
                      {c.title}
                    </button>{' '}
                    · {c.summary}
                  </p>
                ))}
              </details>
            )}
            <button
              className="web-primary"
              disabled={pending}
              aria-pressed={community.following.includes(doc?.id ?? '')}
              onClick={() => void act('follow')}
            >
              {community.following.includes(doc?.id ?? '') ? 'Deixar de seguir' : 'Seguir'}
            </button>
          </div>
        </section>
      )}
      {social && home && (
        <div className="web-social-dashboard">
          <section>
            <h2>Olá, {community.nickname}</h2>
            <p>Compartilhe uma descoberta com a sua rede.</p>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                void submit('post', draft, () => setDraft(''));
              }}
            >
              <label>
                Nova publicação
                <textarea
                  required
                  maxLength={1000}
                  value={draft}
                  onChange={(e) => setDraft(e.target.value)}
                />
              </label>
              <button className="web-primary" disabled={pending || !draft.trim()}>
                Publicar na rede
              </button>
            </form>
            {community.posts.map((post) => (
              <article className="web-personal-post" key={post.id}>
                <strong>{community.nickname}</strong>
                <time>{dateTime(post.timestamp)}</time>
                <p>{post.text}</p>
                <button disabled={pending} onClick={() => void act('deletePost', post.id)}>
                  Excluir publicação
                </button>
              </article>
            ))}
          </section>
          <aside>
            {!!community.suggested?.length && (
              <section className="web-people-suggestions">
                <h2>Pessoas para conhecer</h2>
                {community.suggested.map((c) => (
                  <div key={c.id}>
                    <button className="web-link" onClick={() => navigate(c.url)}>
                      {c.title}
                    </button>
                    <small>{c.summary}</small>
                  </div>
                ))}
              </section>
            )}
            <details>
              <summary>
                Notificações · {community.notifications.filter((n) => !n.read).length} novas
              </summary>
              {!community.notifications.length && (
                <p>Siga pessoas para acompanhar suas publicações.</p>
              )}
              {community.notifications.map((n) => (
                <div key={n.id}>
                  <button className="web-link" onClick={() => navigate(n.url)}>
                    {n.title}
                  </button>
                  {!n.read && (
                    <button
                      disabled={pending}
                      onClick={() => void act('readNotification', '', n.id)}
                    >
                      Marcar como lida
                    </button>
                  )}
                </div>
              ))}
            </details>
            <p>Seguindo {community.following.length} pessoas</p>
          </aside>
        </div>
      )}
      {social && (home || profile) && (
        <section className="web-following-feed">
          <h2>{profile ? 'Publicações desta pessoa' : 'Das suas conexões'}</h2>
          {!community.feed.length && (
            <p>As publicações das pessoas que você segue aparecem aqui.</p>
          )}
          {community.feed.map((card) => (
            <article key={card.id}>
              <strong>{card.author}</strong>
              <h3>
                <button className="web-link" onClick={() => navigate(card.url)}>
                  {card.title}
                </button>
              </h3>
              <p>{card.summary}</p>
            </article>
          ))}
        </section>
      )}
      {listing && community.listing && (
        <section className="web-negotiation">
          <div>
            <strong className="web-price">{money(listing.priceCents)}</strong>
            <p>
              {listing.condition} · {listing.location}
            </p>
            <button className="web-link" onClick={() => navigate(listing.seller.url)}>
              {listing.seller.label}
            </button>
            <p role="status">
              {
                { AVAILABLE: 'Disponível', RESERVED: 'Reservado para você', SOLD: 'Vendido' }[
                  community.listing.status
                ]
              }
            </p>
            {community.listing.offerCents !== null && (
              <p>Proposta aceita: {money(community.listing.offerCents)}</p>
            )}
          </div>
          {community.listing.status === 'AVAILABLE' && (
            <div>
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  void submit('offer', String(Math.round(Number(offer) * 100)), () => setOffer(''));
                }}
              >
                <label>
                  Sua proposta (R$)
                  <input
                    type="number"
                    min="0.01"
                    max={listing.priceCents / 100}
                    step="0.01"
                    required
                    value={offer}
                    onChange={(e) => setOffer(e.target.value)}
                  />
                </label>
                <button disabled={pending}>Enviar proposta</button>
              </form>
              <button
                className="web-primary"
                disabled={pending}
                onClick={() => void act('reserve')}
              >
                Reservar item
              </button>
            </div>
          )}
          {community.listing.status === 'RESERVED' && (
            <div className="web-deal-actions">
              <button disabled={pending} onClick={() => void act('release')}>
                Liberar reserva
              </button>
              <button className="web-primary" disabled={pending} onClick={() => void act('sold')}>
                Confirmar retirada
              </button>
              <p>
                A confirmação encerra o anúncio nesta campanha. A negociação não movimenta saldo
                bancário.
              </p>
            </div>
          )}
        </section>
      )}
      {(profile || listing || community.messages.length > 0) && (
        <details className="web-messages">
          <summary>Mensagens · {community.messages.length}</summary>
          <div aria-live="polite">
            {community.messages.map((m) => (
              <article className={m.incoming ? 'incoming' : 'outgoing'} key={m.id}>
                <strong>{m.incoming ? 'Resposta' : 'Você'}</strong>
                <p>{m.text}</p>
                <small>{dateTime(m.timestamp)}</small>
              </article>
            ))}
          </div>
          {(profile || listing) && (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                void submit('message', message, () => setMessage(''));
              }}
            >
              <label>
                Mensagem
                <textarea
                  required
                  maxLength={1000}
                  value={message}
                  onChange={(e) => setMessage(e.target.value)}
                />
              </label>
              <button className="web-primary" disabled={pending || !message.trim()}>
                Enviar mensagem
              </button>
            </form>
          )}
        </details>
      )}
    </section>
  );
}
