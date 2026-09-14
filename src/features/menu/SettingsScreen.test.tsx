import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import { request } from '../../lib/api';
import type * as Api from '../../lib/api';
import { defaultAppSettings, loadAppSettings, useAppSettings } from '../../lib/app-settings';
import { SettingsScreen } from './SettingsScreen';

const setVolumes = vi.hoisted(() => vi.fn());

vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  desktopRuntime: true,
  request: vi.fn(),
}));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ setFullscreen: vi.fn(() => Promise.resolve()) }),
}));
vi.mock('../../lib/audio-manager', () => ({
  audioManager: { setVolumes, play: vi.fn() },
}));
beforeEach(() => {
  vi.clearAllMocks();
  useAppSettings.setState({ settings: defaultAppSettings, ready: false, saving: false, error: '' });
});
it('loads saved skips and volumes before boot', async () => {
  const saved = {
    ...defaultAppSettings,
    skipStudioIntro: true,
    skipTrailer: true,
    master: 20,
    music: 0,
  };
  vi.mocked(request).mockResolvedValue(saved);
  await loadAppSettings();
  expect(request).toHaveBeenCalledWith('settings_global_get', {}, expect.anything());
  expect(useAppSettings.getState()).toMatchObject({ ready: true, settings: saved });
  expect(setVolumes).toHaveBeenCalledWith(saved);
});
it('persists both skips and adjusted audio through the native preferences command', async () => {
  const saved = { ...defaultAppSettings, skipStudioIntro: true, skipTrailer: true, master: 35 };
  vi.mocked(request).mockResolvedValue(saved);
  render(<SettingsScreen />);
  fireEvent.click(screen.getByRole('checkbox', { name: /Pular intro do estúdio/ }));
  fireEvent.click(screen.getByRole('checkbox', { name: /Pular cinemática/ }));
  fireEvent.click(screen.getByRole('tab', { name: 'Áudio' }));
  fireEvent.change(screen.getByRole('slider', { name: /Volume geral/ }), {
    target: { value: '35' },
  });
  expect(setVolumes).toHaveBeenLastCalledWith(saved);
  fireEvent.click(screen.getByRole('button', { name: 'APLICAR' }));
  await act(async () => {
    await Promise.resolve();
  });
  expect(request).toHaveBeenCalledWith(
    'settings_global_save',
    { settings: saved },
    expect.anything(),
  );
  expect(useAppSettings.getState().settings).toEqual(saved);
  expect(screen.getByText('Preferências salvas.')).toHaveAttribute('role', 'status');
});
it('restores persisted audio when unsaved changes are discarded', () => {
  const { unmount } = render(<SettingsScreen />);
  fireEvent.click(screen.getByRole('tab', { name: 'Áudio' }));
  fireEvent.change(screen.getByRole('slider', { name: /Música/ }), { target: { value: '0' } });
  unmount();
  expect(setVolumes).toHaveBeenLastCalledWith(defaultAppSettings);
  expect(request).not.toHaveBeenCalled();
});
