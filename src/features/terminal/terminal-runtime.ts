import type { Terminal } from '@xterm/xterm';

export const TERMINAL_FONT = "'Cascadia Mono', Consolas, monospace";
export const TERMINAL_THEME = {
  background: '#00000000',
  foreground: '#eeeeec',
  cursor: '#eeeeec',
  selectionBackground: '#ffffff40',
  green: '#4e9a06',
  brightGreen: '#8ae234',
  cyan: '#06989a',
  brightCyan: '#34e2e2',
};
export const terminalPrompt = (user: string, host: string, cwd: string) =>
  `\x1b[1;36m┌──(${user}㉿${host})-[${cwd}]\x1b[0m\r\n└─$ `;

export function terminalOutput(
  result: {
    stdout: string;
    stderr: string;
    ordered?: [1 | 2, string][];
  },
  consumed: { 1: number; 2: number } = { 1: 0, 2: 0 },
): string {
  const chunks = result.ordered?.length
    ? result.ordered
    : ([
        [1, result.stdout],
        [2, result.stderr],
      ] as [1 | 2, string][]);
  const remaining = { ...consumed };
  return chunks
    .map(([fd, original]) => {
      const skip = Math.min(remaining[fd], original.length);
      remaining[fd] -= skip;
      const text = original.slice(skip);
      return fd === 2 && text ? `\x1b[31m${text}\x1b[0m` : text;
    })
    .join('');
}
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
