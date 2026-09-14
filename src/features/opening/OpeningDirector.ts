import { z } from 'zod';
import { worldSchema, commandSchema, pageSchema } from '../../lib/api';
import { useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import type { WindowId } from '../../lib/window-store';
import { installDevelopmentTransport } from '../../lib/development-transport';
import { audioManager } from '../../lib/audio-manager';
import type { AudioCue } from '../../lib/audio-manager';
import { getTerminal } from '../terminal/terminal-runtime';
import data from './scenarios/gameplay-world.json';
import { OPENING_DURATION, sceneAt, TimelinePlayer } from './OpeningTimeline';
import type { OpeningEvent, OpeningScene } from './OpeningTimeline';

const WIFI: WindowId = 'tool:kali-aircrack-ng';
const TRAFFIC: WindowId = 'tool:org.wireshark.Wireshark';
const commands = z.record(z.string(), commandSchema).parse(data.commands);
const pages = z.record(z.string(), pageSchema).parse(data.pages);
export type OpeningFrame = { time: number; scene: OpeningScene; title: boolean; done: boolean };
export class OpeningDirector {
  private events: OpeningEvent[] = [];
  private player: TimelinePlayer;
  private restore: (() => void)[] = [];
  private frameId = 0;
  private started = 0;
  private time = 0;
  private lastScene: OpeningScene = 'opening/desktop';
  private cursor = { x: 870, y: 440 };
  private moves: { at: number; x: number; y: number }[] = [{ at: 0, x: 870, y: 440 }];
  readonly audio: { at: number; cue: AudioCue }[] = [];
  readonly executed: { at: number; id: string }[] = [];
  readonly failures: string[] = [];
  constructor(
    private readonly surface: HTMLElement,
    private readonly onFrame: (frame: OpeningFrame) => void,
  ) {
    if (!import.meta.env.DEV) {
      throw new Error('OpeningScenarioMode is not available in production');
    }
    const game = useGame.getState();
    const windows = useWindows.getState();
    this.restore.push(() => {
      useGame.setState(game, true);
      useWindows.setState(windows, true);
    });
    useGame.setState({
      world: worldSchema.parse(structuredClone(data.world)),
      missions: [],
      error: '',
      busy: false,
      revision: 0,
    });
    useWindows.getState().reset();
    this.restore.push(
      installDevelopmentTransport(async (command, args) => {
        await Promise.resolve();
        const world = useGame.getState().world!;
        switch (command) {
          case 'world_get':
            return world;
          case 'mission_get_state':
            return [];
          case 'vfs_list':
            return Object.values(world.vfs.nodes).filter((node) => node.parentId === args.path);
          case 'vfs_read': {
            const node = world.vfs.nodes[String(args.path)];
            if (!node) {
              throw new Error(`Missing scenario file: ${String(args.path)}`);
            }
            return node.content;
          }
          case 'messages_read':
            this.mutate((copy) => {
              copy.messages.forEach((message) => {
                if (message.contact === args.contact) {
                  message.read = true;
                }
              });
            });
            return null;
          case 'message_reply':
            this.notify(String(args.contact), `Você: ${String(args.text)}`);
            return null;
          case 'execute_terminal': {
            const result = commands[String(args.command)];
            if (!result) {
              throw new Error(`Missing capture command: ${String(args.command)}`);
            }
            if (args.command === 'lab scan 100') {
              this.mutate((copy) => {
                copy.memoryCandidates = data.labCandidates;
              });
            }
            return result;
          }
          case 'wireless_scan':
            return data.wireless;
          case 'wireless_inspect': {
            if (args.bssid !== '02:00:00:00:20:04') {
              throw new Error('Unknown scenario signal');
            }
            this.mutate((copy) => {
              const path = data.evidence.capturePath;
              copy.vfs.nodes[path] = {
                id: path,
                parentId: '/home/kali/Downloads',
                name: 'orion.capture.json',
                kind: 'file',
                content: JSON.stringify(data.packets, null, 2),
                owner: 'kali',
                group: 'kali',
                mode: 420,
                modifiedAt: 100,
                metadata: {},
              };
            });
            return data.evidence;
          }
          case 'traffic_read':
          case 'traffic_follow':
            return data.packets;
          case 'browser_navigate': {
            const page = pages[String(args.address)];
            if (!page) {
              throw new Error('Unknown scenario page');
            }
            return page;
          }
          default:
            throw new Error(`Scenario cannot invoke ${command}`);
        }
      }),
    );
    this.restore.push(
      audioManager.observe((cue, bus) => {
        if (bus === 'sfx' && this.started && this.time <= OPENING_DURATION) {
          this.audio.push({ at: Math.round(this.time), cue });
        }
      }),
    );
    this.build();
    this.events.sort((a, b) => a.at - b.at);
    this.moves.sort((a, b) => a.at - b.at);
    this.player = new TimelinePlayer(this.events);
  }
  private mutate(change: (world: z.infer<typeof worldSchema>) => void): void {
    const world = structuredClone(useGame.getState().world!);
    change(world);
    useGame.setState((state) => ({ world, revision: state.revision + 1 }));
  }
  private at(time: number, id: string, action: () => void): void {
    this.events.push({
      at: time,
      id,
      run: () => {
        try {
          action();
          this.executed.push({ at: time, id });
        } catch (error) {
          this.failures.push(`${id}: ${String(error)}`);
        }
      },
    });
  }
  private move(at: number, x: number, y: number): void {
    this.moves.push({ at, x, y });
  }
  private open(
    id: WindowId,
    x: number,
    y: number,
    width: number,
    height: number,
    path?: string,
  ): void {
    useWindows.getState().open(id, path);
    useWindows.getState().update(id, { x, y, width, height, maximized: false, fullscreen: false });
  }
  private hide(id: WindowId): void {
    useWindows.getState().update(id, { minimized: true });
  }
  private button(id: WindowId, text: string, double = false): void {
    const root = this.surface.querySelector(`[data-window-id="${id}"]`);
    const button = Array.from(root?.querySelectorAll<HTMLButtonElement>('button') ?? []).find(
      (item) => item.textContent?.trim() === text,
    );
    if (!button) {
      throw new Error(`Missing ${id} button: ${text}`);
    }
    if (double) {
      button.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
    } else {
      button.click();
    }
    audioManager.play('ui-click');
  }
  private notify(contact: string, text: string): void {
    this.mutate((world) => {
      if (!world.contacts.includes(contact)) {
        world.contacts.push(contact);
      }
      world.messages.push({ id: `opening-${world.messages.length}`, contact, text, read: false });
    });
  }
  private write(id: WindowId, text: string): void {
    const terminal = getTerminal(id);
    if (!terminal) {
      throw new Error(`Terminal ${id} was not mounted`);
    }
    terminal.write(text);
  }
  private type(time: number, id: WindowId, input: string, ms = 36, result = true): void {
    [...input].forEach((char, index) =>
      this.at(time + index * ms, `type:${id}:${index}:${time}`, () => getTerminal(id)?.type(char)),
    );
    this.at(time + input.length * ms + 120, `submit:${input}`, () => {
      this.write(id, '\r\n');
      audioManager.play('ui-terminal');
      if (result) {
        const output = commands[input];
        if (!output) {
          throw new Error(`Command fixture missing: ${input}`);
        }
        this.write(id, `${output.stdout}\r\n`);
        getTerminal(id)?.command('');
        getTerminal(id)?.prompt(output);
      }
    });
  }
  private background(start: number, end: number, interval: number): void {
    for (let time = start; time < end; time += interval) {
      const index = Math.floor((time - start) / interval);
      this.at(time, `activity:${time}`, () => {
        this.write(
          'terminal',
          `\x1b[36m[${String(index).padStart(4, '0')}]\x1b[0m host discovered · vex.local  service detected\r\n`,
        );
        this.write(
          'terminal:2',
          `/home/kali/${['projects', 'Downloads', 'tools'][index % 3]}  ${['indexed', 'checksum OK', 'session active'][index % 3]}\r\n`,
        );
        this.write(
          'terminal:3',
          `23:17:${String(index % 60).padStart(2, '0')}  10.20.4.15 → 10.20.4.2  session active\r\n`,
        );
      });
    }
  }
  private build(): void {
    this.move(1800, 280, 19);
    this.at(2600, 'open:first-terminal', () => this.open('terminal', 160, 100, 800, 460));
    this.at(3150, 'clear:first-terminal', () => getTerminal()?.clear());
    this.move(5600, 280, 19);
    this.at(6000, 'open:second-terminal', () => this.open('terminal:2', 490, 270, 770, 400));
    this.at(6500, 'clear:second-terminal', () => getTerminal('terminal:2')?.clear());
    this.move(7700, 850, 300);
    this.move(8500, 970, 180);
    this.at(8500, 'move:second-terminal', () =>
      useWindows.getState().update('terminal:2', { x: 635, y: 150 }),
    );
    this.move(9300, 280, 19);
    this.at(9600, 'open:third-terminal', () => this.open('terminal:3', 300, 440, 820, 345));
    this.at(10100, 'clear:third-terminal', () => getTerminal('terminal:3')?.clear());
    this.type(12100, 'terminal', 'nmap vex.local');
    this.type(12200, 'terminal:2', 'tree /home/kali');
    this.type(12400, 'terminal:3', 'ss');
    this.background(13900, 18000, 540);
    this.move(17700, 93, 19);
    this.at(18000, 'open:files', () =>
      this.open('files', 210, 85, 1050, 660, '/home/kali/Downloads'),
    );
    this.move(18800, 480, 210);
    this.at(19000, 'files:directory', () => this.button('files', 'recebidos', true));
    this.at(20100, 'files:select', () => this.button('files', 'registro.txt'));
    this.at(20900, 'files:back', () => this.button('files', '↑'));
    this.move(21400, 275, 330);
    this.at(21600, 'files:projects', () => this.button('files', 'projects'));
    this.at(22600, 'files:open-notes', () => this.button('files', 'notas.txt', true));
    this.at(23100, 'position:editor', () =>
      useWindows.getState().update('editor', { x: 550, y: 220, width: 740, height: 450 }),
    );
    this.at(24000, 'message:vex', () => {
      this.hide('files');
      this.hide('editor');
      this.notify('VEX', 'Tenho um trabalho pra você.');
    });
    this.move(24700, 1215, 800);
    this.at(25400, 'open:messenger', () => this.open('messages', 300, 130, 900, 590));
    this.at(26600, 'message:vex-fragment', () =>
      this.notify('VEX', 'Olha os sinais próximos. Tem algo fora do lugar naquela empresa.'),
    );
    this.at(27800, 'message:reply', () => this.notify('VEX', 'Você: Estou vendo.'));
    this.at(29000, 'open:forum', () => {
      this.hide('messages');
      this.open('forum', 200, 65, 1060, 715);
    });
    this.move(30000, 990, 530);
    for (let i = 0; i <= 30; i++) {
      this.at(30100 + i * 65, `forum:scroll:${i}`, () => {
        const forum = this.surface.querySelector<HTMLElement>('.forum');
        if (forum) {
          forum.scrollTop = i * 9;
        }
      });
    }
    this.at(34000, 'open:wireless', () => {
      this.hide('forum');
      this.open(WIFI, 180, 85, 1100, 655);
    });
    this.move(34300, 1130, 140);
    this.at(34800, 'wireless:scan', () => this.button(WIFI, 'Atualizar redes'));
    this.move(35400, 310, 285);
    this.at(35900, 'wireless:select', () => this.button(WIFI, 'ORION-CORP'));
    this.move(36500, 290, 340);
    this.at(36800, 'wireless:inspect', () => this.button(WIFI, 'Inspecionar sinal'));
    this.at(40000, 'open:traffic', () => {
      this.hide(WIFI);
      this.open(TRAFFIC, 125, 60, 1200, 735, data.evidence.capturePath ?? undefined);
    });
    this.move(40100, 1220, 110);
    this.at(40500, 'traffic:open', () => this.button(TRAFFIC, 'Abrir captura'));
    this.move(40800, 1230, 160);
    this.at(41100, 'traffic:follow', () => this.button(TRAFFIC, 'Seguir fluxo 4'));
    this.at(46000, 'operation:terminal', () => {
      this.hide(TRAFFIC);
      this.open('terminal:3', 260, 125, 970, 610);
      getTerminal('terminal:3')?.clear();
    });
    this.move(46300, 620, 350);
    this.type(46400, 'terminal:3', 'ssh vex@vex.local lab-only', 26);
    this.type(47600, 'terminal:3', 'sudo cat /var/log/web.log', 24);
    this.type(48800, 'terminal:3', 'sudo edit /etc/web.conf enabled=true', 17);
    this.type(50000, 'terminal:3', 'sudo service web restart', 20);
    this.type(51100, 'terminal:3', 'sudo id', 28);
    this.at(52000, 'escalation:windows', () => {
      this.open('terminal', 95, 70, 785, 420);
      this.open('terminal:2', 660, 220, 735, 410);
      this.open('terminal:3', 230, 435, 785, 370);
      this.open('files', 560, 350, 795, 420, '/home/kali/Downloads');
      this.open('browser', 635, 55, 775, 440);
    });
    this.at(52600, 'browser:archive', () => this.button('browser', 'archive'));
    this.at(53900, 'escalation:focus', () => useWindows.getState().focus('terminal'));
    this.background(52600, 58000, 250);
    this.at(55000, 'escalation:message', () => this.notify('VEX', 'Preserva o original.'));
    this.at(56800, 'escalation:files', () => useWindows.getState().focus('files'));
    this.at(58000, 'null:got-you', () => {
      this.notify('NULL', 'Te peguei.');
      this.open('messages', 280, 125, 950, 610);
    });
    this.at(58500, 'null:focus-contact', () => {
      const contact = Array.from(this.surface.querySelectorAll<HTMLButtonElement>('.contact')).find(
        (button) => button.textContent?.includes('NULL'),
      );
      if (!contact) {
        throw new Error('NULL contact unavailable');
      }
      contact.click();
    });
    this.move(57800, 900, 390);
    this.move(63000, 900, 390);
    this.at(63000, 'chaos:desktop', () => {
      this.hide('messages');
      useWindows.getState().focus('terminal:2');
    });
    this.background(63000, 69000, 90);
    this.at(64000, 'chaos:browser', () => {
      useWindows.getState().focus('browser');
      this.button('browser', 'b1');
    });
    this.at(64700, 'chaos:message', () => this.notify('NULL', 'Não fecha a janela.'));
    this.at(65400, 'chaos:file', () => {
      this.mutate((world) => {
        world.vfs.nodes['/home/kali/projects/notas.txt'].content +=
          '\nPROCESSAMENTO CONCLUÍDO\nIntegridade: verificada\n';
      });
      this.open('editor', 570, 250, 755, 430, '/home/kali/projects/notas.txt');
    });
    this.at(66700, 'chaos:move', () =>
      useWindows.getState().update('terminal:3', { x: 130, y: 325, width: 905, height: 465 }),
    );
    this.at(67900, 'chaos:focus', () => useWindows.getState().focus('terminal:3'));
    this.move(68700, 320, 130);
    this.at(69000, 'final:focus', () => {
      useWindows.getState().focus('terminal');
      getTerminal()?.terminal.focus();
    });
    this.move(69800, 830, 122);
    this.at(70100, 'final:maximize', () =>
      useWindows.getState().update('terminal', { maximized: true }),
    );
    this.at(71800, 'final:clear', () => getTerminal()?.clear());
    this.at(72500, 'final:fullscreen', () =>
      useWindows.getState().update('terminal', { fullscreen: true }),
    );
    this.move(72500, 1380, 840);
    this.type(74400, 'terminal', 'lab trace', 105, false);
    const output = commands['lab trace'].stdout.trim().split('\n');
    output.forEach((line, index) =>
      this.at(76000 + index * 450, `trace:${index}`, () => this.write('terminal', `${line}\r\n`)),
    );
    for (let time = 80000; time < 85000; time += 80) {
      const index = Math.floor((time - 80000) / 80);
      this.at(time, `flood:${index}`, () => {
        const count = 2 + Math.floor(index * 0.45);
        const text = Array.from(
          { length: count },
          (_, row) =>
            `${(0x00af21 + index * 17 + row).toString(16).toUpperCase().padStart(8, '0')}  01001001  node discovered  route established  session opened  packet received  access granted`,
        ).join('\r\n');
        this.write('terminal', `\x1b[92m${text}\r\n`);
      });
    }
    this.at(84920, 'title:cursor-handoff', () => this.write('terminal', '\x1b[?25l'));
    this.at(85000, 'title:takeover', () =>
      this.onFrame({ time: 85000, scene: 'opening/title', title: true, done: false }),
    );
  }
  start(): void {
    this.started = performance.now();
    const tick = () => {
      this.time = Math.min(performance.now() - this.started, OPENING_DURATION);
      this.player.advance(this.time);
      const scene = sceneAt(this.time);
      const currentMove = this.moves.findIndex((move) => move.at > this.time);
      const end = this.moves[currentMove < 0 ? this.moves.length - 1 : currentMove];
      const begin =
        this.moves[Math.max(0, (currentMove < 0 ? this.moves.length : currentMove) - 1)];
      const duration = Math.min(700, end.at - begin.at);
      const fraction = duration
        ? Math.max(0, Math.min(1, (this.time - end.at + duration) / duration))
        : 1;
      const eased = fraction * fraction * (3 - 2 * fraction);
      this.cursor = {
        x: begin.x + (end.x - begin.x) * eased,
        y: begin.y + (end.y - begin.y) * eased,
      };
      const cursor = this.surface.querySelector<HTMLElement>('.scenario-cursor');
      if (cursor) {
        cursor.style.transform = `translate(${this.cursor.x}px, ${this.cursor.y}px)`;
        cursor.style.opacity = this.time >= 73500 ? '0' : '1';
      }
      this.surface.style.setProperty(
        '--opening-fade',
        String(Math.max(0, 1 - (this.time - 400) / 1400)),
      );
      this.surface.dataset.openingTime = String(Math.round(this.time));
      this.surface.dataset.openingState = scene;
      if (scene !== this.lastScene || this.time >= 85000) {
        this.onFrame({
          time: this.time,
          scene,
          title: this.time >= 85000,
          done: this.time >= OPENING_DURATION,
        });
        this.lastScene = scene;
      }
      if (this.time < OPENING_DURATION) {
        this.frameId = requestAnimationFrame(tick);
      }
    };
    this.frameId = requestAnimationFrame(tick);
  }
  dispose(): void {
    cancelAnimationFrame(this.frameId);
    this.restore.reverse().forEach((dispose) => dispose());
    audioManager.stopAll();
  }
}
