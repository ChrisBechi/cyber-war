import { useGame } from '../../lib/game-store';

export function Journey() {
  const world = useGame((s) => s.world);
  return (
    <div className="app-scroll">
      <div className="section-heading">
        <div className="eyebrow">TECHNICAL JOURNEY</div>
        <h2>O que você aprendeu fazendo.</h2>
        <p className="muted">
          As ferramentas estão disponíveis desde o início. Este registro cresce com resultados
          válidos.
        </p>
      </div>
      <div className="techniques">
        {world?.techniques.map((t) => (
          <span key={t}>✓ {t}</span>
        ))}
      </div>
      {world?.techniques.length === 0 && (
        <p className="muted">Abra o terminal e explore seu computador.</p>
      )}
      <h3>Inventário</h3>
      <div className="techniques">
        {world?.inventory.map((t) => (
          <span key={t}>{t}</span>
        ))}
      </div>
    </div>
  );
}
