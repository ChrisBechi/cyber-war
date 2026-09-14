import { useEffect, useRef, useState } from 'react';
import { flushSync } from 'react-dom';
import { z } from 'zod';
import { Desktop } from '../desktop/Desktop';
import { useWindows } from '../../lib/window-store';
import type { WindowId } from '../../lib/window-store';
import { useGame } from '../../lib/game-store';
import { commandSchema } from '../../lib/api';
import { audioManager } from '../../lib/audio-manager';
import type { AudioCue } from '../../lib/audio-manager';
import { getTerminal } from '../terminal/terminal-runtime';
import { OpeningDirector } from './OpeningDirector';
import { TimelinePlayer } from './OpeningTimeline';
import type { OpeningEvent } from './OpeningTimeline';
import scenes from './scenarios/montage-scenes.json';
import data from './scenarios/gameplay-world.json';
import './opening.css';

const duration = scenes.reduce((total, scene) => total + scene.duration * 1000, 0);
const commands = z.record(z.string(), commandSchema).parse(data.commands);
const wifi: WindowId = 'tool:kali-aircrack-ng';
const traffic: WindowId = 'tool:org.wireshark.Wireshark';
export function OpeningMontage() {
  const surface = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);
  const [running, setRunning] = useState(false);
  const [done, setDone] = useState(false);
  const time = useRef(0);
  const audio = useRef<{ at: number; cue: AudioCue }[]>([]);
  const executed = useRef<{ at: number; id: string }[]>([]);
  const failures = useRef<string[]>([]);
  useEffect(() => {
    if (!surface.current) {
      return;
    }
    const director = new OpeningDirector(surface.current, () => undefined);
    setReady(true);
    return () => director.dispose();
  }, []);
  useEffect(() => {
    if (!running || !surface.current) {
      return;
    }
    const root = surface.current;
    const events: OpeningEvent[] = [];
    const observe = audioManager.observe((cue, bus) => {
      if (bus === 'sfx') {
        audio.current.push({ at: time.current, cue });
      }
    });
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
    const mutate = (
      action: (world: NonNullable<ReturnType<typeof useGame.getState>['world']>) => void,
    ) => {
      const world = structuredClone(useGame.getState().world!);
      action(world);
      useGame.setState((state) => ({ world, revision: state.revision + 1 }));
    };
    const notify = (contact: string, text: string) =>
      mutate((world) => {
        if (!world.contacts.includes(contact)) {
          world.contacts.push(contact);
        }
        world.messages.push({ id: `montage-${world.messages.length}`, contact, text, read: false });
      });
    const open = (id: WindowId, path?: string) => {
      useWindows.getState().open(id, path);
      useWindows.getState().update(id, {
        x: 0,
        y: 0,
        width: window.innerWidth,
        height: window.innerHeight - 73,
        maximized: true,
      });
    };
    const place = (id: WindowId, rect: readonly number[], path?: string) => {
      const [x, y, w, h] = rect;
      useWindows.getState().open(id, path);
      useWindows.getState().update(id, {
        x: Math.round(x * window.innerWidth),
        y: Math.round(y * (window.innerHeight - 73)),
        width: Math.round(w * window.innerWidth) - 4,
        height: Math.round(h * (window.innerHeight - 73)) - 4,
        maximized: false,
      });
    };
    const tile = (layout = 'launch-terminals') => {
      const width = window.innerWidth;
      const height = window.innerHeight - 73;
      const left = Math.round(width * 0.52);
      if (layout === 'network-terminals') {
        place('terminal', [0, 0, 0.42, 1]);
        place('terminal:2', [0.42, 0, 0.58, 1]);
        return;
      }
      if (layout === 'swarm') {
        place('terminal', [0, 0, 0.6, 0.5]);
        place('terminal:2', [0.6, 0, 0.4, 0.5]);
        place('terminal:3', [0, 0.5, 1, 0.5]);
        return;
      }
      (['terminal', 'terminal:2', 'terminal:3'] as const).forEach((id, i) => {
        useWindows.getState().open(id);
        useWindows.getState().update(id, {
          x: i ? left + 4 : 0,
          y: i === 2 ? Math.round(height / 2) + 4 : 0,
          width: i ? width - left - 4 : left - 4,
          height: i ? Math.floor(height / 2) - 4 : height,
          maximized: false,
        });
      });
    };
    const button = (id: WindowId, label: string) => {
      const element = [
        ...root.querySelectorAll<HTMLButtonElement>(`[data-window-id="${id}"] button`),
      ].find((item) => item.textContent?.trim() === label);
      if (!element) {
        throw new Error(`Missing ${id}: ${label}`);
      }
      element.click();
      audioManager.play('ui-click');
    };
    const input = (selector: string, value: string) => {
      const element = root.querySelector<HTMLInputElement>(selector);
      if (!element) {
        throw new Error(`Missing input ${selector}`);
      }
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!.call(
        element,
        value,
      );
      flushSync(() => element.dispatchEvent(new Event('input', { bubbles: true })));
    };
    const browser = (address: string) => {
      input('input[aria-label="Endereço"]', address);
      root.querySelector<HTMLFormElement>('.address-bar')!.requestSubmit();
    };
    const write = (id: string, text: string) => {
      const terminal = getTerminal(id);
      if (!terminal) {
        throw new Error(`Terminal not ready: ${id}`);
      }
      terminal.write(text);
    };
    const type = (when: number, id: string, text: string, speed = 28) => {
      [...text].forEach((char, i) =>
        at(when + i * speed, `${id}:key:${when}:${i}`, () => getTerminal(id)?.type(char)),
      );
      at(when + text.length * speed + 70, `${id}:execute:${text}:${when}`, () => {
        const result = commands[text];
        if (!result) {
          throw new Error(`Missing real command result: ${text}`);
        }
        write(id, `\r\n${result.stdout}\r\n`);
        getTerminal(id)?.command('');
        getTerminal(id)?.prompt(result);
        audioManager.play('ui-terminal');
      });
    };
    const stream = (start: number, end: number, variant = 'discovery') => {
      for (let tick = start, index = 0; tick < end; tick += 145, index++) {
        const n = index;
        at(tick, `streams:${tick}`, () => {
          if (variant === 'network') {
            write(
              'terminal',
              `\r\x1b[2K\x1b[36m${String(n + 12).padStart(4, '0')}\x1b[0m ESTABLISHED 10.20.4.2:${49152 + n} → 10.20.4.15:22\r\n`,
            );
            write(
              'terminal:2',
              `\r\x1b[2Kwlan0  rx=${19420 + n * 137}  tx=${9050 + n * 41}  latency=${3 + (n % 5)}ms  loss=0%\r\n`,
            );
            return;
          }
          if (variant === 'services') {
            write(
              'terminal',
              `\r\x1b[2K\x1b[32m[${200 + n}]\x1b[0m GET /${['manifest', 'blocks/07', 'checksums', 'releases'][n % 4]}  ${512 + n * 23} bytes\r\n`,
            );
            write(
              'terminal:2',
              `\r\x1b[2K${['22/tcp ssh', '80/tcp http', '443/tcp https'][n % 3]}  response=${2 + (n % 7)}ms  ttl=64\r\n`,
            );
            write(
              'terminal:3',
              `\r\x1b[2K[VERIFY ${String(n).padStart(3, '0')}] ${(0xbf8494 + n * 8413).toString(16)}  /home/kali/${['projects/session.json', 'Documents/notes.txt', 'Downloads/recebidos/registro.txt', 'tools/node.trace'][n % 4]}  OK\r\n`,
            );
            return;
          }
          write(
            'terminal',
            `\r\x1b[2K\x1b[36m[${String(n).padStart(4, '0')}]\x1b[0m  10.20.4.15  host discovered · ssh active\r\n`,
          );
          write(
            'terminal:2',
            `\r\x1b[2K${(0xae4180 + n * 713).toString(16)}  /home/kali/${['projects', 'Downloads', 'tools'][n % 3]}  integrity OK\r\n`,
          );
          write(
            'terminal:3',
            `\r\x1b[2K23:17:${String(n % 60).padStart(2, '0')}  10.20.4.20 → 10.20.4.2  packet received\r\n`,
          );
        });
      }
    };
    let offset = 0;
    for (const scene of scenes) {
      const start = offset;
      const end = start + scene.duration * 1000;
      at(start, `scene:${scene.id}`, () => {
        root.dataset.openingState = scene.id;
        useWindows.getState().reset();
        if (['launch-terminals', 'root-operation'].includes(scene.id)) {
          open('terminal');
        } else if (['network-terminals', 'swarm'].includes(scene.id)) {
          tile(scene.id);
        } else if (scene.id === 'forum') {
          open('forum');
        } else if (['deep-web', 'news'].includes(scene.id)) {
          open('browser');
        } else if (['conversation', 'null'].includes(scene.id)) {
          const contact = scene.id === 'null' ? 'NULL' : 'VEX';
          mutate((world) => {
            world.messages = [];
          });
          notify(
            contact,
            contact === 'NULL' ? 'Te peguei.' : 'O mesmo horário. Três registros diferentes.',
          );
          open('messages');
        } else if (scene.id === 'wireless') {
          open(wifi);
        } else if (scene.id === 'traffic') {
          open(traffic, data.evidence.capturePath ?? undefined);
        } else if (scene.id === 'files') {
          open('files', '/home/kali/Downloads/recebidos');
        } else if (scene.id === 'archive') {
          place('browser', [0, 0, 0.64, 1]);
          place('files', [0.64, 0, 0.36, 1], '/home/kali/Downloads');
        } else if (scene.id === 'codelab') {
          place('codelab', [0, 0, 0.64, 1]);
          place('terminal', [0.64, 0, 0.36, 1]);
        } else if (scene.id === 'processes') {
          place('processes', [0, 0, 0.62, 0.5]);
          place('terminal', [0, 0.5, 0.62, 0.5]);
          place('editor', [0.62, 0, 0.38, 1], '/home/kali/tools/node.trace');
        } else if (scene.id === 'hash-compare') {
          place('editor', [0, 0, 0.38, 1], '/home/kali/Downloads/recebidos/registro.txt');
          place('terminal', [0.38, 0, 0.62, 1]);
        } else if (scene.id === 'routes') {
          place('terminal', [0, 0, 1, 0.43]);
          place('terminal:2', [0, 0.43, 0.38, 0.57]);
          place('browser', [0.38, 0.43, 0.62, 0.57]);
        } else if (scene.id === 'encoded-session') {
          place('files', [0, 0, 0.43, 0.46], '/home/kali/projects');
          place('terminal', [0, 0.46, 0.63, 0.54]);
          place('editor', [0.55, 0.06, 0.45, 0.86], '/home/kali/projects/session.json');
        } else if (scene.id === 'permissions') {
          place('files', [0, 0, 0.42, 1], '/home/kali/Downloads/recebidos');
          place('terminal', [0.42, 0, 0.58, 1]);
        } else {
          open('editor', '/home/kali/projects/notas.txt');
        }
      });
      if (scene.id === 'launch-terminals') {
        at(start + 250, 'launch:clear', () => getTerminal()?.clear());
        type(start + 400, 'terminal', 'lab trace', 45);
        at(start + 1150, 'launch:windows', () => tile());
        at(start + 1400, 'launch:clear-satellites', () => {
          getTerminal('terminal:2')?.clear();
          getTerminal('terminal:3')?.clear();
        });
        type(start + 1600, 'terminal:2', 'tree /home/kali');
        type(start + 1700, 'terminal:3', 'nmap vex.local');
        stream(start + 2600, end - 100);
      } else if (['network-terminals', 'swarm'].includes(scene.id)) {
        at(start + 200, `clear:${scene.id}`, () =>
          ['terminal', 'terminal:2', 'terminal:3'].forEach((id) => getTerminal(id)?.clear()),
        );
        type(start + 300, 'terminal', scene.id === 'swarm' ? 'curl https://archive.org' : 'ss');
        type(start + 400, 'terminal:2', scene.id === 'swarm' ? 'nmap archive.org' : 'ip addr');
        if (scene.id === 'swarm') {
          type(start + 500, 'terminal:3', 'find /home/kali');
          at(start + 2400, 'swarm:reframe', () => {
            place('terminal', [0, 0, 0.48, 0.56]);
            place('terminal:2', [0.48, 0, 0.52, 0.56]);
            place('terminal:3', [0, 0.56, 1, 0.44]);
          });
        }
        stream(start + 1300, end - 100, scene.id === 'swarm' ? 'services' : 'network');
      } else if (scene.id === 'root-operation') {
        at(start + 200, 'operation:clear', () => getTerminal()?.clear());
        type(start + 350, 'terminal', 'ssh vex@vex.local lab-only', 20);
        type(start + 1500, 'terminal', 'sudo cat /var/log/web.log', 17);
        type(start + 2550, 'terminal', 'sudo edit /etc/web.conf enabled=true', 16);
        type(start + 3600, 'terminal', 'sudo service web restart', 19);
        type(start + 4650, 'terminal', 'sudo id', 22);
      } else if (scene.id === 'forum') {
        for (let i = 0; i < 38; i++) {
          at(start + 650 + i * 75, `forum:scroll:${i}`, () => {
            root.querySelector('.forum')!.scrollTop = i * 6;
          });
        }
      } else if (['deep-web', 'news', 'archive'].includes(scene.id)) {
        at(start + 200, `navigate:${scene.id}`, () =>
          browser(
            scene.id === 'news'
              ? 'b1.tech'
              : scene.id === 'archive'
                ? 'archive.org'
                : 'pg6mmjiyjmcrsslvykfwnntlaru7p5svn6y2ymmju6nubxndf4pscryd.onion',
          ),
        );
        for (let i = 0; i < 25; i++) {
          at(start + 1000 + i * 80, `page:scroll:${scene.id}:${i}`, () => {
            root.querySelector('.browser-page')!.scrollTop = i * 5;
          });
        }
      } else if (scene.id === 'conversation') {
        at(start + 750, 'vex:message', () =>
          notify('VEX', 'Entra no espelho. Preserva o que encontrar.'),
        );
        const reply = 'Estou no arquivo. Vou cruzar os horários.';
        [...reply].forEach((_, i) =>
          at(start + 1550 + i * 24, `reply:${i}`, () => {
            input('input[aria-label="Mensagem"]', reply.slice(0, i + 1));
            audioManager.play('ui-key');
          }),
        );
        at(start + 2700, 'vex:send', () =>
          root.querySelector<HTMLFormElement>('.conversation form')!.requestSubmit(),
        );
        at(start + 3550, 'vex:response', () =>
          notify('VEX', 'Quando os terminais coincidirem, você vai entender.'),
        );
      } else if (scene.id === 'null') {
        at(start + 1000, 'null:typing', () => notify('NULL', 'Não fecha a janela.'));
        at(start + 2300, 'null:message', () =>
          notify('NULL', 'Eu também estou vendo os registros.'),
        );
      } else if (scene.id === 'wireless') {
        at(start + 350, 'wifi:scan', () => button(wifi, 'Atualizar redes'));
        at(start + 850, 'wifi:select', () => button(wifi, 'ORION-CORP'));
        at(start + 1350, 'wifi:inspect', () => button(wifi, 'Inspecionar sinal'));
      } else if (scene.id === 'traffic') {
        at(start + 350, 'traffic:open', () => button(traffic, 'Abrir captura'));
        at(start + 950, 'traffic:follow', () => button(traffic, 'Seguir fluxo 4'));
      } else if (scene.id === 'files') {
        at(start + 1000, 'files:select', () => button('files', 'registro.txt'));
        at(start + 2000, 'files:open', () => {
          const file = [
            ...root.querySelectorAll<HTMLButtonElement>('[data-window-id="files"] button'),
          ].find((element) => element.textContent?.trim() === 'registro.txt');
          file?.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
        });
        at(start + 2200, 'files:editor', () =>
          useWindows.getState().update('editor', { maximized: true }),
        );
      } else if (scene.id === 'codelab') {
        at(start + 200, 'lab:clear', () => getTerminal()?.clear());
        at(start + 600, 'lab:scan', () => button('codelab', 'Escanear'));
        type(start + 650, 'terminal', 'lab inspect');
        at(start + 1650, 'lab:inspect', () => button('codelab', 'Inspecionar runtime'));
      } else if (scene.id === 'processes') {
        at(start + 200, 'processes:clear', () => getTerminal()?.clear());
        type(start + 450, 'terminal', 'ps');
        at(start + 1700, 'processes:trace', () =>
          write(
            'terminal',
            '\r\n\x1b[36mprocess watcher attached\x1b[0m\r\nScheduler active · virtual processes synchronized\r\n',
          ),
        );
      } else if (scene.id === 'hash-compare') {
        at(start + 200, 'hash:clear', () => getTerminal()?.clear());
        type(start + 350, 'terminal', 'sha256sum /home/kali/Downloads/recebidos/registro.txt', 20);
        type(start + 1950, 'terminal', 'sha256sum /home/kali/projects/session.json', 20);
        at(start + 3450, 'hash:verified', () =>
          write('terminal', '\r\n\x1b[32mORIGINAL PRESERVED · SHA-256 recorded\x1b[0m\r\n'),
        );
      } else if (scene.id === 'routes') {
        at(start + 200, 'routes:clear', () => {
          getTerminal()?.clear();
          getTerminal('terminal:2')?.clear();
        });
        type(start + 350, 'terminal', 'traceroute archive.org', 28);
        type(start + 600, 'terminal:2', 'dig vex.local', 35);
        at(start + 400, 'routes:reference', () => browser('wipedia.org'));
        at(start + 1950, 'routes:hops', () =>
          write(
            'terminal',
            '\r\n\x1b[36mroute[0]\x1b[0m 10.20.4.1 → gateway\r\n\x1b[36mroute[1]\x1b[0m 10.20.4.20 → archive.org\r\nRoute complete · 2 hops · no packet loss\r\n',
          ),
        );
      } else if (scene.id === 'encoded-session') {
        at(start + 200, 'encoded:clear', () => getTerminal()?.clear());
        type(start + 400, 'terminal', 'base64 /home/kali/projects/session.json', 25);
        at(start + 2100, 'encoded:move', () =>
          place('editor', [0.58, 0, 0.42, 1], '/home/kali/projects/session.json'),
        );
      } else if (scene.id === 'permissions') {
        at(start + 200, 'permissions:clear', () => getTerminal()?.clear());
        type(start + 350, 'terminal', 'chmod 600 /home/kali/Downloads/recebidos/registro.txt', 21);
        type(start + 2050, 'terminal', 'stat /home/kali/Downloads/recebidos/registro.txt', 24);
      } else {
        at(start + 900, 'receipts:result', () =>
          mutate((world) => {
            world.vfs.nodes['/home/kali/projects/notas.txt'].content +=
              '\n\n23:17:04 / sessão aberta\n23:17:09 / transferência registrada\n23:17:12 / trilha interrompida\n\nINTEGRIDADE VERIFICADA\nOriginais preservados.\n';
          }),
        );
        at(start + 1050, 'receipts:reload', () => button('editor', 'Abrir'));
      }
      offset = end;
    }
    const player = new TimelinePlayer(
      events.sort((a, b) => a.at - b.at),
      duration,
    );
    const started = performance.now();
    let frame = 0;
    const tick = () => {
      time.current = Math.min(duration, performance.now() - started);
      player.advance(time.current);
      root.dataset.openingTime = String(Math.round(time.current));
      if (time.current < duration) {
        frame = requestAnimationFrame(tick);
      } else {
        if (player.completed !== events.length) {
          failures.current.push(`Incomplete capture: ${player.completed}/${events.length} events`);
        }
        setDone(true);
      }
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      observe();
    };
  }, [running]);
  return (
    <div
      ref={surface}
      className="opening-scenario montage-take"
      data-opening-ready={ready}
      data-opening-done={done}
    >
      {ready && <Desktop presentation onMenu={() => undefined} />}
      <div className="bookend-input-shield" />
      {!running && (
        <button className="scenario-start" onClick={() => setRunning(true)}>
          Gravar OpeningScenarioMode
        </button>
      )}
      {done && (
        <script type="application/json" id="opening-capture-manifest">
          {JSON.stringify({
            duration,
            events: executed.current,
            audio: audio.current,
            failures: failures.current,
          })}
        </script>
      )}
    </div>
  );
}
