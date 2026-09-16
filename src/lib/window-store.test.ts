import { beforeEach, describe, expect, it } from 'vitest';
import { useWindows } from './window-store';

describe('window manager', () => {
  beforeEach(() => useWindows.getState().reset());
  it('creates independent file/editor instances and preserves root as a per-window context', () => {
    const first = useWindows.getState().newInstance('files', '/home/kali/Desktop');
    const second = useWindows.getState().newInstance('files', '/root', true);
    const editor = useWindows.getState().newInstance('editor', '/root/notes.txt', true);
    expect(first).not.toBe(second);
    expect(useWindows.getState().windows.map((window) => window.asRoot)).toEqual([
      false,
      true,
      true,
    ]);
    useWindows.getState().close(second);
    expect(useWindows.getState().windows.map((window) => window.id)).toEqual([first, editor]);
  });
  it('keeps separate tool workspaces and restores each without duplicates', () => {
    useWindows.getState().open('tool:kali-nmap');
    useWindows.getState().open('tool:kali-burpsuite');
    useWindows.getState().update('tool:kali-nmap', { minimized: true, width: 900 });
    useWindows.getState().open('tool:kali-nmap');
    expect(useWindows.getState().windows).toHaveLength(2);
    expect(useWindows.getState().windows[0]).toMatchObject({ minimized: false, width: 900 });
  });
  it('reopens one app, preserves other windows and focuses across workspaces', () => {
    useWindows.getState().open('terminal');
    useWindows.getState().open('files');
    useWindows.getState().update('terminal', { minimized: true });
    useWindows.setState({ workspace: 2 });
    useWindows.getState().open('terminal');
    const [terminal, files] = useWindows.getState().windows;
    expect(useWindows.getState().windows).toHaveLength(2);
    expect(terminal.minimized).toBe(false);
    expect(terminal.workspace).toBe(2);
    expect(terminal.z).toBeGreaterThan(files.z);
    useWindows.getState().close('terminal');
    expect(useWindows.getState().windows.map((w) => w.id)).toEqual(['files']);
  });
  it('cycles through every window in both directions and restores minimized windows only in the current workspace', () => {
    const state = useWindows.getState();
    state.open('terminal');
    state.open('files');
    state.open('browser');
    state.open('settings');
    state.update('settings', { workspace: 2 });
    state.update('files', { minimized: true });
    expect(state.cycle(1)).toBe('terminal');
    expect(state.cycle(1)).toBe('files');
    expect(state.cycle(1)).toBe('browser');
    expect(state.cycle(-1)).toBe('files');
    expect(useWindows.getState().windows.find((item) => item.id === 'files')?.minimized).toBe(
      false,
    );
    expect(useWindows.getState().workspace).toBe(1);
  });
  it('restores only windows hidden by show desktop and preserves other workspaces and previously minimized windows', () => {
    const state = useWindows.getState();
    state.open('terminal');
    state.open('browser');
    state.open('files');
    state.update('browser', { minimized: true });
    state.update('files', { workspace: 2 });
    state.toggleDesktop();
    expect(useWindows.getState().windows.map((item) => item.minimized)).toEqual([
      true,
      true,
      false,
    ]);
    state.toggleDesktop();
    expect(useWindows.getState().windows.map((item) => item.minimized)).toEqual([
      false,
      true,
      false,
    ]);
    state.toggleDesktop();
    state.close('terminal');
    state.toggleDesktop();
    expect(useWindows.getState().windows.map((item) => item.id)).toEqual(['browser', 'files']);
    expect(useWindows.getState().windows[0].minimized).toBe(true);
  });
});
