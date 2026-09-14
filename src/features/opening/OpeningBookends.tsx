import { useEffect, useRef, useState } from 'react';
import { Desktop } from '../desktop/Desktop';
import { useWindows } from '../../lib/window-store';
import { useGame } from '../../lib/game-store';
import { audioManager } from '../../lib/audio-manager';
import type { AudioCue } from '../../lib/audio-manager';
import { getTerminal } from '../terminal/terminal-runtime';
import { OpeningDirector } from './OpeningDirector';
import { TimelinePlayer } from './OpeningTimeline';
import type { OpeningEvent } from './OpeningTimeline';
import { CodeRainTitle } from './title/CodeRainTitle';
import './opening.css';

const DURATION = 26500;
const USERNAME = 'root';
export function OpeningBookends() {
  const surface = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);
  const [running, setRunning] = useState(false);
  const [time, setTime] = useState(0);
  const currentTime = useRef(0);
  const audio = useRef<{ at: number; cue: AudioCue }[]>([]);
  const executed = useRef<{ at: number; id: string }[]>([]);
  const failures = useRef<string[]>([]);
  useEffect(() => {
    if (!surface.current) {
      return;
    }
    // Reuse the validated gameplay fixture and isolated transport from the source director.
    const world = new OpeningDirector(surface.current, () => undefined);
    useGame.setState((state) =>
      state.world
        ? {
            world: {
              ...state.world,
              nickname: 'root',
              terminal: { ...state.world.terminal, user: 'root', host: 'lifeos' },
            },
          }
        : {},
    );
    setReady(true);
    return () => world.dispose();
  }, []);
  useEffect(() => {
    if (!running) {
      return;
    }
    const observe = audioManager.observe((cue, bus) => {
      if (bus === 'sfx') {
        audio.current.push({ at: currentTime.current, cue });
      }
    });
    const events: OpeningEvent[] = [];
    const at = (when: number, id: string, action: () => void) =>
      events.push({
        at: when,
        id,
        run: () => {
          try {
            action();
            executed.current.push({ at: when, id });
          } catch (error) {
            failures.current.push(`${id}: ${String(error)}`);
          }
        },
      });
    [...USERNAME].forEach((_, i) =>
      at(650 + i * 125, `username:${i}`, () => audioManager.play('ui-key')),
    );
    for (let i = 0; i < 11; i++) {
      at(1930 + i * 65, `password:${i}`, () => audioManager.play('ui-key'));
    }
    at(3000, 'login:enter', () => audioManager.play('ui-terminal'));
    at(3500, 'prepare:terminal', () => {
      useWindows.getState().open('terminal');
      useWindows.getState().update('terminal', { maximized: true, fullscreen: true });
    });
    at(4300, 'terminal:clear', () => getTerminal()?.clear());
    const code = 'lab trace --all --stream';
    [...code].forEach((char, i) => at(4770 + i * 43, `code:${i}`, () => getTerminal()?.type(char)));
    at(5960, 'code:execute', () => {
      getTerminal()?.write('\r\n');
      audioManager.play('ui-terminal');
    });
    [
      'node discovered',
      'route established',
      'session opened',
      'packet received',
      'access granted',
    ].forEach((line, i) =>
      at(6010 + i * 90, `output:${i}`, () => getTerminal()?.write(`${line}\r\n`)),
    );
    at(6900, 'matrix:takeover', () => getTerminal()?.write('\x1b[?25l'));
    const player = new TimelinePlayer(events.sort((a, b) => a.at - b.at));
    const start = performance.now();
    let frame = 0;
    const tick = () => {
      const elapsed = Math.min(DURATION, performance.now() - start);
      currentTime.current = elapsed;
      player.advance(elapsed);
      setTime(elapsed);
      if (elapsed < DURATION) {
        frame = requestAnimationFrame(tick);
      }
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      observe();
    };
  }, [running]);
  const username = USERNAME.slice(0, Math.max(0, Math.floor((time - 650) / 125)));
  const password = '•'.repeat(Math.min(11, Math.max(0, Math.floor((time - 1930) / 65))));
  return (
    <div
      ref={surface}
      className="opening-scenario bookend-take"
      data-opening-ready={ready}
      data-opening-time={Math.round(time)}
      data-opening-done={time >= DURATION}
      data-opening-state={
        time < 4500
          ? 'opening/login'
          : time < 7000
            ? 'opening/code'
            : time < 18500
              ? 'opening/matrix'
              : 'opening/code-title'
      }
    >
      {ready && <Desktop presentation onMenu={() => undefined} />}
      {time < 4500 && (
        <section
          className={`opening-login ${time >= 3000 ? 'login-accepted' : ''}`}
          aria-label="Início da sessão LifeOS"
        >
          <div className="login-panel">
            <img src="/assets/kali/kali-panel-menu.svg" alt="" />
            <h1>root</h1>
            <p>LifeOS</p>
            <div className={`login-field ${time < 1930 ? 'is-typing' : ''}`}>
              <span>Usuário</span>
              <div>
                {username}
                <i />
              </div>
            </div>
            <div className={`login-field ${time >= 1930 && time < 3000 ? 'is-typing' : ''}`}>
              <span>Senha</span>
              <div>
                {password}
                <i />
              </div>
            </div>
            <div className="login-submit">{time >= 3000 ? 'Iniciando sessão…' : 'Entrar →'}</div>
          </div>
          <footer>
            Português (Brasil)<span>23:17</span>
            <span>⏻</span>
          </footer>
        </section>
      )}
      {time >= 7000 && <CodeRainTitle elapsed={time - 7000} />}
      <div className="bookend-input-shield" />
      {!running && (
        <button className="scenario-start" onClick={() => setRunning(true)}>
          Gravar OpeningScenarioMode
        </button>
      )}
      {time >= DURATION && (
        <script type="application/json" id="opening-capture-manifest">
          {JSON.stringify({
            duration: DURATION,
            events: executed.current,
            audio: audio.current,
            failures: failures.current,
          })}
        </script>
      )}
    </div>
  );
}
