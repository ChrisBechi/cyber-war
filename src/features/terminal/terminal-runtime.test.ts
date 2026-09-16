import { describe, expect, it } from 'vitest';
import { commandSchema } from '../../lib/api';
import { terminalOutput } from './terminal-runtime';

describe('terminal output streams', () => {
  it('keeps backend write order through IPC validation and xterm formatting', () => {
    const result = commandSchema.parse({
      stdout: 'first\nlast\n',
      stderr: 'error\n',
      ordered: [
        [1, 'first\n'],
        [2, 'error\n'],
        [1, 'last\n'],
      ],
      cwd: '/home/kali',
      user: 'kali',
      host: 'lifeos',
      exitCode: 1,
    });
    expect(terminalOutput(result)).toBe('first\n\x1b[31merror\n\x1b[0mlast\n');
  });
  it('renders aggregate archive results and older transports', () => {
    expect(terminalOutput({ stdout: 'ok\n', stderr: 'error\n', ordered: [] })).toBe(
      'ok\n\x1b[31merror\n\x1b[0m',
    );
    expect(terminalOutput({ stdout: 'ok\n', stderr: '' })).toBe('ok\n');
  });
  it('renders only the unconsumed suffix after streaming without dropping stderr', () => {
    const result = {
      stdout: 'á\nlast\n',
      stderr: 'warning\n',
      ordered: [
        [1, 'á\n'],
        [2, 'warning\n'],
        [1, 'last\n'],
      ] as [1 | 2, string][],
    };
    expect(terminalOutput(result, { 1: 2, 2: 0 })).toBe('\x1b[31mwarning\n\x1b[0mlast\n');
    expect(terminalOutput(result, { 1: 7, 2: 8 })).toBe('');
  });
});
