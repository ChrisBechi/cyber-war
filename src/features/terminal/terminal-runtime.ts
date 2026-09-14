import type { Terminal } from '@xterm/xterm';

export const TERMINAL_FONT = "'Cascadia Mono', Consolas, monospace";
export const TERMINAL_THEME = {
  background: '#151a20',
  foreground: '#d8dee9',
  cursor: '#69cfff',
  selectionBackground: '#2e516a',
  green: '#4e9a06',
  brightGreen: '#8ae234',
  cyan: '#06989a',
  brightCyan: '#34e2e2',
};
export const terminalPrompt = (user: string, host: string, cwd: string) =>
  `\x1b[1;36m┌──(${user}㉿${host})-[${cwd}]\x1b[0m\r\n└─$ `;
export type TerminalHandle = {
  terminal: Terminal;
  clear: () => void;
  write: (text: string) => void;
  command: (text: string) => void;
  prompt: (context?: { user: string; host: string; cwd: string }) => void;
  type: (text: string) => void;
};
const terminals = new Map<string, TerminalHandle>();
export function registerTerminal(id: string, handle: TerminalHandle): () => void {
  terminals.set(id, handle);
  return () => {
    if (terminals.get(id) === handle) {
      terminals.delete(id);
    }
  };
}
export const getTerminal = (id = 'terminal'): TerminalHandle | undefined => terminals.get(id);
