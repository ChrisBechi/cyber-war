import { act, useGame } from '../../lib/game-store';

export function Missions() {
  const { missions, busy } = useGame();
  const activeMission = missions.find((mission) => mission.status === 'active');
  return (
    <div className="app-scroll mission-list">
      <div className="section-heading">
        <div className="eyebrow">CAIXA DE ENTRADA</div>
        <h2>O que vem a seguir</h2>
        <p className="muted">Resolva o problema. O caminho é seu.</p>
        {activeMission && (
          <p className="muted">
            Conclua ou abandone “{activeMission.title}” para iniciar outra missão.
          </p>
        )}
      </div>
      {missions.map((m) => (
        <article className={`mission-card ${m.status}`} key={m.id}>
          <div className="eyebrow">
            {m.contact}{' '}
            <span>
              {m.status === 'completed'
                ? 'CONCLUÍDO'
                : m.status === 'active'
                  ? 'EM ANDAMENTO'
                  : 'DISPONÍVEL'}
            </span>
          </div>
          <h3>{m.title}</h3>
          <p>{m.description}</p>
          {m.objective && <p className="objective">↳ {m.objective}</p>}
          {m.hint && (
            <details>
              <summary>Preciso de uma pista</summary>
              <p className="hint">{m.hint}</p>
            </details>
          )}
          {m.status === 'available' && (
            <button
              className="primary"
              disabled={busy || Boolean(activeMission)}
              onClick={() => act('mission_start', { missionId: m.id })}
            >
              {activeMission ? 'Aguardando missão atual' : 'Aceitar'}
            </button>
          )}
          {m.choices.map((c) => (
            <button
              key={c.id}
              disabled={busy}
              onClick={() => act('mission_choose', { missionId: m.id, choiceId: c.id })}
            >
              {c.label}
            </button>
          ))}
          {m.status === 'active' && (
            <button
              className="text-button"
              disabled={busy}
              onClick={() => act('mission_abort_attempt', { missionId: m.id })}
            >
              Sair da tentativa e voltar ao computador
            </button>
          )}
        </article>
      ))}
    </div>
  );
}
