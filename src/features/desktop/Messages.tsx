import { useEffect, useState } from 'react';

import { act, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';

export function Messages() {
  const world = useGame((s) => s.world);
  const [contact, setContact] = useState(
    () => world?.messages.filter((message) => !message.read).at(-1)?.contact ?? 'Mãe',
  );
  const [draft, setDraft] = useState('');
  const hasUnread = world?.messages.some((m) => m.contact === contact && !m.read);
  useEffect(() => {
    if (hasUnread) {
      act('messages_read', { contact });
    }
  }, [contact, hasUnread]);
  if (!world) {
    return null;
  }
  return (
    <div className="messenger">
      <aside>
        <div className="section-heading">
          <h3>Mensagens</h3>
          <small className="muted">{world.nickname} · disponível</small>
        </div>
        {world.contacts.map((c) => (
          <button
            className={contact === c ? 'contact selected' : 'contact'}
            key={c}
            onClick={() => setContact(c)}
          >
            <span className="avatar">{c.slice(0, 1)}</span>
            <span>
              {c}
              <small>
                {world.messages.filter((m) => m.contact === c && !m.read).length || 'offline'}
              </small>
            </span>
          </button>
        ))}
      </aside>
      <section className="conversation">
        <header>
          <b>{contact}</b>
          <small>Conversa privada</small>
        </header>
        <div className="message-log" role="log">
          {world.messages
            .filter((m) => m.contact === contact)
            .map((m) => (
              <p key={m.id} className={m.text.startsWith('Você:') ? 'bubble outgoing' : 'bubble'}>
                {m.text}
              </p>
            ))}
        </div>
        <form
          className="toolbar"
          onSubmit={(e) => {
            e.preventDefault();
            if (draft.trim()) {
              act('message_reply', { contact, text: draft });
              setDraft('');
            }
          }}
        >
          <input
            aria-label="Mensagem"
            placeholder="Escreva uma mensagem…"
            value={draft}
            maxLength={1000}
            onChange={(e) => setDraft(e.target.value)}
          />
          <button>Enviar</button>
        </form>
        <button className="text-button" onClick={() => useWindows.getState().open('missions')}>
          Ver trabalhos e respostas da história →
        </button>
      </section>
    </div>
  );
}
