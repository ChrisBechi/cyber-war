import type { Terminal } from '@xterm/xterm';
import { waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { nanoOptionsSchema } from '../../lib/api';
import type { NanoLaunch } from '../../lib/api';
import { NanoEditor } from './nano';

function editor(content: string, overrides: Partial<NanoLaunch['options']> = {}) {
  const defaults = Object.fromEntries(
    Object.entries(nanoOptionsSchema.shape).map(([key, schema]) => [
      key,
      key === 'tabSize' ? 8 : schema.safeParse(false).success ? false : null,
    ]),
  );
  const options = nanoOptionsSchema.parse({ ...defaults, ...overrides });
  const actions = {
    write: vi.fn(() => Promise.resolve()),
    read: vi.fn(() => Promise.resolve('inserted')),
    close: vi.fn(() => Promise.resolve()),
    exited: vi.fn(),
  };
  const write = vi.fn();
  const terminal = { cols: 80, rows: 24, write } as unknown as Terminal;
  const instance = new NanoEditor(
    terminal,
    {
      path: '/home/kali/note.txt',
      displayName: 'note.txt',
      content,
      expectedContent: content,
      options,
      startingLine: 1,
      startingColumn: 1,
    },
    actions,
  );
  return { instance, actions, terminal, write };
}
async function save(instance: NanoEditor) {
  instance.handle('\x0f');
  instance.handle('\r');
  await Promise.resolve();
}
describe('interactive nano compatibility', () => {
  it.each([
    ['hello', false, 'hello\n'],
    ['hello', true, 'hello'],
    ['hello\n', true, 'hello\n'],
    ['hello\n\n', true, 'hello\n\n'],
    ['', false, ''],
  ] as const)(
    'preserves or adds final newlines correctly: %j, nonewlines=%s',
    async (content, noNewlines, expected) => {
      const { instance, actions } = editor(content, { noNewlines });
      await save(instance);
      await waitFor(() =>
        expect(actions.write).toHaveBeenCalledWith('/home/kali/note.txt', expected, content),
      );
      instance.dispose();
    },
  );
  it('read-only mode never writes or inserts into a buffer', async () => {
    const { instance, actions } = editor('original', { view: true });
    instance.handle('changed');
    await save(instance);
    expect(actions.write).not.toHaveBeenCalled();
    instance.dispose();
  });
  it('undo/redo and tabs-to-spaces operate inside their own buffer', async () => {
    const a = editor('', { tabsToSpaces: true, tabSize: 4 });
    const b = editor('other', { noNewlines: true });
    a.instance.handle('\t');
    a.instance.handle('x');
    a.instance.handle('\x1a');
    a.instance.handle('\x19');
    await save(a.instance);
    await save(b.instance);
    expect(a.actions.write).toHaveBeenCalledWith('/home/kali/note.txt', '    x\n', '');
    expect(b.actions.write).toHaveBeenCalledWith('/home/kali/note.txt', 'other', 'other');
    a.instance.dispose();
    b.instance.dispose();
  });
  it('a failed save keeps the buffer available for retry', async () => {
    const { instance, actions } = editor('');
    actions.write.mockRejectedValueOnce(new Error('conflict'));
    instance.handle('keep');
    await save(instance);
    await waitFor(() => expect(actions.write).toHaveBeenCalledTimes(1));
    await save(instance);
    await waitFor(() => expect(actions.write).toHaveBeenCalledTimes(2));
    expect(actions.write).toHaveBeenLastCalledWith('/home/kali/note.txt', 'keep\n', '');
    instance.dispose();
  });

  it('search wraps the current line, uses codepoint positions and keeps not-found status', async () => {
    const { instance, actions, write } = editor('é😀 target END', { noNewlines: true });
    instance.handle('\x05');
    instance.handle('\x17');
    instance.handle('TARGET');
    instance.handle('\r');
    instance.handle('X');
    await save(instance);
    expect(actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'é😀 Xtarget END',
      'é😀 target END',
    );
    instance.handle('\x17');
    for (let i = 0; i < 6; i += 1) {
      instance.handle('\x7f');
    }
    instance.handle('missing');
    instance.handle('\r');
    expect(write).toHaveBeenLastCalledWith(expect.stringContaining('not found'));
  });

  it('replaces occurrences interactively across lines with skip, all and one-step undo', async () => {
    const { instance, actions } = editor('foo foo\nFOO tail', { noNewlines: true });
    instance.handle('\x1c');
    instance.handle('foo');
    instance.handle('\r');
    instance.handle('bar');
    instance.handle('\r');
    instance.handle('n');
    instance.handle('a');
    await save(instance);
    expect(actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'foo bar\nbar tail',
      'foo foo\nFOO tail',
    );
    instance.handle('\x1bu');
    await save(instance);
    expect(actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'foo foo\nFOO tail',
      'foo bar\nbar tail',
    );
    instance.handle('\x1be');
    await save(instance);
    expect(actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'foo bar\nbar tail',
      'foo foo\nFOO tail',
    );
  });

  it('cuts marked multiline text and pastes at the cursor without losing line boundaries', async () => {
    const { instance, actions } = editor('abc\ndef\nghi', { noNewlines: true });
    instance.handle('\x1b[C');
    instance.handle('\x1e');
    instance.handle('\x1b[B');
    instance.handle('\x1b[C');
    instance.handle('\x0b');
    await save(instance);
    expect(actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'af\nghi',
      'abc\ndef\nghi',
    );
    instance.handle('\x15');
    await save(instance);
    expect(actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'abc\ndef\nghi',
      'af\nghi',
    );
  });

  it('copies without mutation, cuts consecutive lines and pastes CRLF once', async () => {
    const copied = editor('line\ntail', { noNewlines: true });
    copied.instance.handle('\x1b6');
    copied.instance.handle('\x1b[B');
    copied.instance.handle('\x15');
    await save(copied.instance);
    expect(copied.actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'line\nline\ntail',
      'line\ntail',
    );
    const cut = editor('one\ntwo\nthree', { noNewlines: true });
    cut.instance.handle('\x0b');
    cut.instance.handle('\x0b');
    cut.instance.handle('\x15');
    await save(cut.instance);
    expect(cut.actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'one\ntwo\nthree',
      'one\ntwo\nthree',
    );
    const pasted = editor('', { noNewlines: true, autoIndent: true });
    pasted.instance.handle('  a\r\nb');
    await save(pasted.instance);
    expect(pasted.actions.write).toHaveBeenLastCalledWith('/home/kali/note.txt', '  a\nb', '');
  });

  it('saves and closes after exit confirmation, but keeps failed saves open', async () => {
    const { instance, actions } = editor('a', { noNewlines: true });
    instance.handle('b');
    instance.handle('\x18');
    instance.handle('y');
    instance.handle('\r');
    await waitFor(() => expect(actions.exited).toHaveBeenCalledOnce());
    expect(actions.write).toHaveBeenCalledWith('/home/kali/note.txt', 'ba', 'a');
    const failed = editor('');
    failed.actions.write.mockRejectedValueOnce(new Error('conflict'));
    failed.instance.handle('keep');
    failed.instance.handle('\x18');
    failed.instance.handle('y');
    failed.instance.handle('\r');
    await waitFor(() => expect(failed.actions.write).toHaveBeenCalled());
    expect(failed.actions.close).not.toHaveBeenCalled();
    await save(failed.instance);
    expect(failed.actions.write).toHaveBeenLastCalledWith('/home/kali/note.txt', 'keep\n', '');
  });

  it('inserts newline without adding an extra final newline and restricts replacements to the mark', async () => {
    const split = editor('ab', { noNewlines: true });
    split.instance.handle('\x1b[C');
    split.instance.handle('\r');
    await save(split.instance);
    expect(split.actions.write).toHaveBeenLastCalledWith('/home/kali/note.txt', 'a\nb', 'ab');
    const marked = editor('foo foo foo', { noNewlines: true });
    for (let i = 0; i < 4; i += 1) {
      marked.instance.handle('\x1b[C');
    }
    marked.instance.handle('\x1e');
    for (let i = 0; i < 3; i += 1) {
      marked.instance.handle('\x1b[C');
    }
    marked.instance.handle('\x1c');
    marked.instance.handle('foo');
    marked.instance.handle('\r');
    marked.instance.handle('x');
    marked.instance.handle('\r');
    marked.instance.handle('a');
    await save(marked.instance);
    expect(marked.actions.write).toHaveBeenLastCalledWith(
      '/home/kali/note.txt',
      'foo x foo',
      'foo foo foo',
    );
  });
});
