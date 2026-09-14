import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';
import { useEffect, useRef } from 'react';
import { z } from 'zod';
import { commandSchema, emptySchema, request } from '../../lib/api';
import type { NanoLaunch } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { InputLine } from './input-line';
import { NanoEditor } from './nano';
import {
  registerTerminal,
  TERMINAL_FONT,
  TERMINAL_THEME,
  terminalPrompt,
} from './terminal-runtime';
import { audioManager } from '../../lib/audio-manager';
import '@xterm/xterm/css/xterm.css';

const completionSchema = z.object({
  line: z.string(),
  cursor: z.number().int().nonnegative(),
  candidates: z.array(z.string()),
});
const sessionSchema = z.object({
  cwd: z.string(),
  user: z.string(),
  host: z.string().nullable(),
});

export function TerminalWindow({
  sessionId,
  windowId,
  initialCommand,
  initialCwd,
  asRoot = false,
}: {
  sessionId?: string;
  windowId?: string;
  initialCommand?: string;
  initialCwd?: string;
  asRoot?: boolean;
}) {
  const container = useRef<HTMLDivElement>(null);
  const identity = useRef<string | null>(null);
  if (identity.current === null) {
    identity.current = windowId ?? sessionId ?? crypto.randomUUID();
  }
  const terminalId = identity.current;
  const fontSize = useGame((s) => Number(s.world?.settings.fontSize ?? 14));
  const initial = useRef({ fontSize, command: initialCommand, cwd: initialCwd, asRoot });
  const terminal = useRef<Terminal | null>(null);
  const resizeTerminal = useRef<(() => void) | null>(null);
  useEffect(() => {
    if (!container.current) {
      return;
    }
    const term = new Terminal({
      cursorBlink: true,
      fontSize: initial.current.fontSize,
      fontFamily: TERMINAL_FONT,
      fontWeight: 'bold',
      theme: TERMINAL_THEME,
      convertEol: true,
      scrollback: 2000,
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(container.current);
    terminal.current = term;
    const input = new InputLine();
    let busy = true;
    let disposed = false;
    let nanoEditor: NanoEditor | null = null;
    const localHost = useGame.getState().world?.hostname ?? 'game-hacker';
    let context = { user: 'kali', host: localHost, cwd: '/home/kali' };
    const draw = () => {
      const visible = input.viewport(term.cols - 5);
      term.write(`\r\x1b[2K└─$ ${visible.text}\x1b[${5 + visible.cursorColumn}G`);
    };
    const prompt = (nextContext?: { user: string; host: string; cwd: string }) => {
      if (nextContext) {
        context = nextContext;
      }
      term.write(`\r\n${terminalPrompt(context.user, context.host, context.cwd)}`);
      draw();
    };
    const startNano = (launch: NanoLaunch) => {
      term.write('\x1b[?1049h\x1b[2J\x1b[H');
      nanoEditor = new NanoEditor(term, launch, {
        write: async (path, content, expectedContent) => {
          if (disposed) {
            throw new Error('Terminal fechado');
          }
          await perform(
            'nano_write',
            { sessionId: terminalId, path, content, expectedContent },
            emptySchema,
          );
        },
        read: (path) => {
          if (disposed) {
            return Promise.reject(new Error('Terminal fechado'));
          }
          return request('nano_read', { sessionId: terminalId, path }, z.string());
        },
        close: async () => {
          if (!disposed) {
            await perform('nano_close', { sessionId: terminalId }, emptySchema);
          }
        },
        exited: () => {
          nanoEditor = null;
          if (!disposed) {
            term.write('\x1b[?1049l');
            busy = false;
            prompt();
          }
        },
      });
    };
    const submit = () => {
      const command = input.submit();
      audioManager.play('ui-terminal');
      term.write(`\r\x1b[2K└─$ ${command}\r\n`);
      if (!command.trim()) {
        prompt();
        return;
      }
      busy = true;
      void perform('execute_terminal', { sessionId: terminalId, command }, commandSchema)
        .then((result) => {
          if (!disposed) {
            context = { user: result.user, host: result.host, cwd: result.cwd };
            if (result.launchApp === 'vigilia') {
              useWindows.getState().open('vigilia');
            }
            if (result.interactive) {
              startNano(result.interactive);
              return;
            }
            // Keep a readable breathing line between the submitted command and its result.
            term.write('\r\n');
            term.write(result.stdout);
            if (result.stderr) {
              term.write(`\x1b[31m${result.stderr}\x1b[0m`);
            }
          }
        })
        .catch((error: unknown) => {
          if (!disposed) {
            term.writeln('');
            term.writeln(`\x1b[31m${String(error)}\x1b[0m`);
          }
        })
        .finally(() => {
          busy = false;
          if (!disposed && !nanoEditor) {
            prompt();
          }
        });
    };
    const complete = () => {
      const revision = input.revision;
      void request(
        'terminal_complete',
        { sessionId: terminalId, line: input.text, cursor: input.cursor },
        completionSchema,
      )
        .then((result) => {
          if (disposed || busy || revision !== input.revision) {
            return;
          }
          const changed = result.line !== input.text || result.cursor !== input.cursor;
          input.set(result.line, result.cursor);
          if (!changed && result.candidates.length > 1) {
            term.write(`\r\x1b[2K\r\n${result.candidates.join('  ')}\r\n`);
            prompt();
          } else if (result.candidates.length === 0) {
            term.write('\x07');
          }
          draw();
        })
        .catch(() => {
          if (!disposed && !busy && revision === input.revision) {
            term.write('\x07');
          }
        });
    };
    term.attachCustomKeyEventHandler((event) => {
      if (event.key === 'Tab' && !event.shiftKey) {
        event.preventDefault();
      }
      return true;
    });
    term.writeln('LifeOS / terminal');
    term.writeln('help · ↑ ↓ histórico · ← → editar · Tab comandos/arquivos · Ctrl+C limpar');
    const subscription = term.onData((data) => {
      if (nanoEditor) {
        nanoEditor.handle(data);
        return;
      }
      if (busy) {
        return;
      }
      if (input.edit(data)) {
        draw();
        return;
      }
      if (data === '\t') {
        complete();
        return;
      }
      if (data === '\x03') {
        term.write(`\r\x1b[2K└─$ ${input.text}^C\r\n`);
        input.set('');
        prompt();
        return;
      }
      if (data === '\r') {
        submit();
        return;
      }
      if (data.startsWith('\x1b')) {
        return;
      }
      // Paste never executes a command implicitly.
      const safe = [...data.replace(/[\r\n]+/g, ' ')]
        .filter((c) => c.charCodeAt(0) >= 32 && c.charCodeAt(0) !== 127)
        .join('');
      input.insert(safe);
      if (safe) {
        audioManager.play('ui-key');
      }
      draw();
    });
    // The development director feeds the real xterm instance, including its ANSI parser.
    const unregister = registerTerminal(terminalId, {
      terminal: term,
      clear: () => {
        input.set('');
        term.reset();
        prompt();
        term.focus();
      },
      write: (text) => term.write(text),
      command: (text) => {
        input.set(text);
      },
      prompt,
      type: (text) => {
        input.insert(text);
        draw();
        audioManager.play('ui-key');
      },
    });
    const resize = () => {
      if (container.current && container.current.clientWidth > 0) {
        fit.fit();
        if (nanoEditor) {
          nanoEditor.resize();
        } else if (!busy) {
          draw();
        }
      }
    };
    const observer = new ResizeObserver(resize);
    resizeTerminal.current = resize;
    observer.observe(container.current);
    fit.fit();
    term.focus();
    // Queue lifecycle operations with mutations so StrictMode remounts and a
    // window closing during an outstanding command cannot reopen a stale session.
    void perform(
      'terminal_open',
      { sessionId: terminalId, cwd: initial.current.cwd, asRoot: initial.current.asRoot },
      sessionSchema,
    )
      .then((session) => {
        if (disposed) {
          return;
        }
        context = { ...session, host: session.host ?? localHost };
        busy = false;
        prompt();
        if (initial.current.command) {
          input.set(initial.current.command);
          draw();
          submit();
        }
      })
      .catch((error: unknown) => {
        if (!disposed) {
          term.writeln(`\r\n\x1b[31m${String(error)}\x1b[0m`);
        }
      });
    return () => {
      disposed = true;
      nanoEditor?.dispose();
      observer.disconnect();
      subscription.dispose();
      unregister();
      term.dispose();
      terminal.current = null;
      resizeTerminal.current = null;
      void perform('terminal_close', { sessionId: terminalId }, emptySchema).catch(() => undefined);
    };
  }, [terminalId]);
  useEffect(() => {
    if (terminal.current) {
      terminal.current.options.fontSize = fontSize;
      resizeTerminal.current?.();
    }
  }, [fontSize]);
  return (
    <div
      ref={container}
      className="terminal-body"
      data-terminal-id={terminalId}
      aria-label="Terminal de comandos"
    />
  );
}
