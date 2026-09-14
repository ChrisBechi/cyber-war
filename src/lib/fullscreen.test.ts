import { beforeEach, describe, expect, it, vi } from 'vitest';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { handleFullscreenKey } from './fullscreen';

vi.mock('@tauri-apps/api/core', () => ({ isTauri: vi.fn(() => true) }));
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: vi.fn() }));
vi.mock('./game-store', () => ({ useGame: { setState: vi.fn() } }));

describe('native fullscreen shortcut', () => {
  const state = { fullscreen: false };
  const setFullscreen = vi.fn((value: boolean) => {
    state.fullscreen = value;
    return Promise.resolve();
  });
  beforeEach(() => {
    state.fullscreen = false;
    setFullscreen.mockClear();
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(getCurrentWindow).mockReturnValue({
      isFullscreen: () => Promise.resolve(state.fullscreen),
      setFullscreen,
    } as unknown as ReturnType<typeof getCurrentWindow>);
  });
  it('uses native fullscreen on F11 and returns to the window on the next press', async () => {
    const event = new KeyboardEvent('keydown', { key: 'F11', cancelable: true });
    handleFullscreenKey(event);
    await vi.waitFor(() => expect(setFullscreen).toHaveBeenLastCalledWith(true));
    expect(event.defaultPrevented).toBe(true);
    handleFullscreenKey(new KeyboardEvent('keydown', { key: 'F11' }));
    await vi.waitFor(() => expect(setFullscreen).toHaveBeenLastCalledWith(false));
  });
  it('ignores key-repeat and unrelated keys', async () => {
    handleFullscreenKey(new KeyboardEvent('keydown', { key: 'F11', repeat: true }));
    handleFullscreenKey(new KeyboardEvent('keydown', { key: 'F1' }));
    await Promise.resolve();
    expect(setFullscreen).not.toHaveBeenCalled();
  });
});
