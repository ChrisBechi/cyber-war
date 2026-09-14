import { useEffect, useRef, useState } from 'react';
import './vigilia-game.css';

type VigiliaSnapshot = {
  ready: boolean;
  screen: string;
  state: string;
  profile: {
    credits: number;
    owned: string[];
    secrets: string[];
    best: number;
  };
  world: {
    levelIndex: number;
    levelName: string;
    status: string;
    hp: number;
    maxHp: number;
    ammo: number;
    mag: number;
    weapon: string;
    score: number;
    time: number;
    x: number;
  } | null;
};

const gameUrl = '/games/vigilia/index.html';

export function VigiliaGame() {
  const frame = useRef<HTMLIFrameElement>(null);
  const [snapshot, setSnapshot] = useState<VigiliaSnapshot | null>(null);
  const [credits, setCredits] = useState('');
  const [health, setHealth] = useState('');
  const [ammo, setAmmo] = useState('');
  const [score, setScore] = useState('');
  const [godMode, setGodMode] = useState(false);
  const [message, setMessage] = useState('Carregando SECTOR IX…');

  useEffect(() => {
    const receive = (event: MessageEvent) => {
      if (event.source !== frame.current?.contentWindow) {
        return;
      }
      const message =
        typeof event.data === 'object' && event.data !== null
          ? (event.data as {
              type?: string;
              state?: VigiliaSnapshot;
              result?: { reason?: string };
            })
          : null;
      if (message?.type === 'sector-ix-trainer-state' && message.state) {
        const next = message.state;
        setSnapshot(next);
        setCredits(String(next.profile.credits));
        if (next.world) {
          setHealth(String(next.world.hp));
          setAmmo(String(next.world.ammo));
          setScore(String(next.world.score));
        }
        setMessage(next.ready ? 'Trainer conectado ao jogo interno.' : 'Inicializando jogo…');
      }
      if (message?.type === 'sector-ix-trainer-response' && message.result?.reason) {
        setMessage(message.result.reason);
      }
    };
    window.addEventListener('message', receive);
    return () => window.removeEventListener('message', receive);
  }, []);

  const command = (action: string, value?: number | boolean) => {
    frame.current?.contentWindow?.postMessage(
      { type: 'sector-ix-trainer-command', requestId: crypto.randomUUID(), action, value },
      '*',
    );
  };

  const apply = (action: string, raw: string) => {
    const value = Number(raw);
    if (!Number.isFinite(value)) {
      setMessage('Informe um valor numérico válido.');
      return;
    }
    command(action, value);
  };

  const sync = () => command('sync');

  return (
    <div className="vigilia-shell">
      <div className="vigilia-stage">
        <iframe
          ref={frame}
          title="SECTOR IX — Protocolo Zero"
          src={gameUrl}
          allow="fullscreen"
          onLoad={sync}
        />
      </div>
      <aside className="vigilia-trainer" aria-label="Trainer de SECTOR IX">
        <header>
          <span className="vigilia-kicker">CYBER WAR / JOGO INTERNO</span>
          <h1>SECTOR IX</h1>
          <p>PROTOCOLO ZERO</p>
        </header>
        <div className={`vigilia-status ${snapshot?.ready ? 'is-ready' : ''}`} role="status">
          <i /> {message}
        </div>
        <section>
          <h2>Estado da operação</h2>
          <dl className="vigilia-readout">
            <div>
              <dt>Fase</dt>
              <dd>{snapshot?.world?.levelName ?? 'Menu'}</dd>
            </div>
            <div>
              <dt>Arma</dt>
              <dd>{snapshot?.world?.weapon ?? '—'}</dd>
            </div>
            <div>
              <dt>Segredos</dt>
              <dd>{snapshot ? `${snapshot.profile.secrets.length} / 6` : '—'}</dd>
            </div>
          </dl>
        </section>
        <section>
          <h2>Valores persistentes</h2>
          <label>
            Créditos
            <span>
              <input
                inputMode="numeric"
                value={credits}
                onChange={(event) => setCredits(event.target.value)}
              />
              <button onClick={() => apply('setCredits', credits)}>Aplicar</button>
            </span>
          </label>
          <button className="vigilia-action" onClick={() => command('addCredits', 1000)}>
            +1.000 créditos
          </button>
          <button className="vigilia-action" onClick={() => command('unlockAll')}>
            Liberar arsenal completo
          </button>
        </section>
        <section>
          <h2>Durante a missão</h2>
          <label>
            Vida
            <span>
              <input
                inputMode="numeric"
                value={health}
                onChange={(event) => setHealth(event.target.value)}
              />
              <button onClick={() => apply('setHealth', health)}>Aplicar</button>
            </span>
          </label>
          <label>
            Munição
            <span>
              <input
                inputMode="numeric"
                value={ammo}
                onChange={(event) => setAmmo(event.target.value)}
              />
              <button onClick={() => apply('setAmmo', ammo)}>Aplicar</button>
            </span>
          </label>
          <label>
            Pontuação
            <span>
              <input
                inputMode="numeric"
                value={score}
                onChange={(event) => setScore(event.target.value)}
              />
              <button onClick={() => apply('setScore', score)}>Aplicar</button>
            </span>
          </label>
          <label className="vigilia-check">
            <input
              type="checkbox"
              checked={godMode}
              onChange={(event) => {
                const enabled = event.target.checked;
                setGodMode(enabled);
                command('setGodMode', enabled);
              }}
            />
            Integridade protegida
          </label>
          <button className="vigilia-action" onClick={() => command('finishLevel')}>
            Concluir fase atual
          </button>
        </section>
        <section>
          <h2>Acesso rápido</h2>
          <div className="vigilia-levels">
            {[0, 1, 2].map((level) => (
              <button key={level} onClick={() => command('setLevel', level)}>
                Operação {level + 1}
              </button>
            ))}
          </div>
          <button className="vigilia-sync" onClick={sync}>
            Atualizar estado
          </button>
        </section>
        <small className="vigilia-note">
          Trainer local do jogo interno. As alterações de créditos e arsenal ficam salvas no perfil
          do SECTOR IX dentro desta instalação.
        </small>
      </aside>
    </div>
  );
}
