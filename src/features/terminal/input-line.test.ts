import { describe, expect, it } from 'vitest';
import { InputLine } from './input-line';

describe('terminal input editing', () => {
  it('inserts, backspaces and deletes at the cursor without discarding the suffix', () => {
    const line = new InputLine();
    line.insert('cat notes.tx');
    line.edit('\x1b[D');
    line.insert('t');
    expect(line.text).toBe('cat notes.ttx');
    line.edit('\x7f');
    expect(line.text).toBe('cat notes.tx');
    line.edit('\x1b[C');
    line.insert('t');
    expect(line.text).toBe('cat notes.txt');
    line.edit('\x1b[H');
    line.edit('\x1b[3~');
    line.insert('c');
    expect(line.submit()).toBe('cat notes.txt');
  });
  it('keeps cursor bounds, accented graphemes and the draft when browsing history', () => {
    const line = new InputLine();
    line.insert('echo café');
    line.submit();
    line.insert('cat a\u0301rvore.txt');
    const draft = line.text;
    line.edit('\x1b[A');
    expect(line.text).toBe('echo café');
    line.edit('\x1b[B');
    expect(line.text).toBe(draft);
    line.set('a\u0301🌳');
    line.edit('\x1b[D');
    line.edit('\x7f');
    expect(line.text).toBe('🌳');
    expect(line.cursor).toBe(0);
    line.edit('\x1b[D');
    expect(line.cursor).toBe(0);
    line.edit('\x1b[F');
    line.edit('\x1b[C');
    expect(line.cursor).toBe(1);
  });
  it('scrolls a long input horizontally and keeps editing correct after a resize', () => {
    const line = new InputLine();
    line.insert('echo ' + 'x'.repeat(200));
    expect(line.viewport(40).text).toHaveLength(40);
    expect(line.viewport(40).cursorColumn).toBe(40);
    line.edit('\x1b[H');
    expect(line.viewport(15).text.startsWith('echo ')).toBe(true);
    expect(line.viewport(15).cursorColumn).toBe(0);
    line.edit('\x1b[C');
    line.edit('\x1b[3~');
    line.insert('X');
    expect(line.text.startsWith('eXho ')).toBe(true);
    line.set('cat Doc notes.txt', 7);
    line.set('cat Documents/ notes.txt', 14);
    line.insert('report');
    expect(line.text).toBe('cat Documents/report notes.txt');
  });
});
