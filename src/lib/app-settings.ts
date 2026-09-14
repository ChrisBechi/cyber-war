import { create } from 'zustand';
import { z } from 'zod';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { desktopRuntime, request } from './api';
import { audioManager } from './audio-manager';

export const appSettingsSchema = z.object({
  skipStudioIntro: z.boolean(),
  skipTrailer: z.boolean(),
  language: z.literal('pt-BR'),
  master: z.number().int().min(0).max(100),
  music: z.number().int().min(0).max(100),
  sfx: z.number().int().min(0).max(100),
  voice: z.number().int().min(0).max(100),
  fullscreen: z.boolean(),
  resolution: z.enum(['1024x640', '1280x720', '1440x900', '1920x1080']),
  uiScale: z.union([z.literal(90), z.literal(100), z.literal(110), z.literal(125)]),
  reducedMotion: z.boolean(),
  highContrast: z.boolean(),
});
export type AppSettings = z.infer<typeof appSettingsSchema>;
export const defaultAppSettings: AppSettings = {
  skipStudioIntro: false,
  skipTrailer: false,
  language: 'pt-BR',
  master: 80,
  music: 70,
  sfx: 80,
  voice: 80,
  fullscreen: true,
  resolution: '1440x900',
  uiScale: 100,
  reducedMotion: false,
  highContrast: false,
};
type Store = { settings: AppSettings; ready: boolean; error: string; saving: boolean };
export const useAppSettings = create<Store>(() => ({
  settings: defaultAppSettings,
  ready: false,
  error: '',
  saving: false,
}));
let loading: Promise<void> | undefined;

async function applyDisplay(settings: AppSettings): Promise<void> {
  if (!desktopRuntime) {
    return;
  }
  const window = getCurrentWindow();
  await window.setFullscreen(settings.fullscreen);
  if (!settings.fullscreen) {
    const [width, height] = settings.resolution.split('x').map(Number);
    await window.setSize(new LogicalSize(width, height));
    await window.center();
  }
}
export function loadAppSettings(): Promise<void> {
  loading ??= (async () => {
    try {
      const settings = desktopRuntime
        ? await request('settings_global_get', {}, appSettingsSchema)
        : defaultAppSettings;
      audioManager.setVolumes(settings);
      await applyDisplay(settings);
      useAppSettings.setState({ settings, ready: true, error: '' });
    } catch (error) {
      useAppSettings.setState({
        ready: true,
        error: `Não foi possível carregar as preferências: ${String(error)}`,
      });
    }
  })();
  return loading;
}
export async function saveAppSettings(next: AppSettings): Promise<void> {
  const settings = appSettingsSchema.parse(next);
  const previous = useAppSettings.getState().settings;
  useAppSettings.setState({ saving: true, error: '' });
  const displayChanged =
    previous.fullscreen !== settings.fullscreen || previous.resolution !== settings.resolution;
  try {
    if (displayChanged) {
      await applyDisplay(settings);
    }
    const saved = await request('settings_global_save', { settings }, appSettingsSchema);
    audioManager.setVolumes(saved);
    useAppSettings.setState({ settings: saved });
  } catch (error) {
    if (displayChanged) {
      await applyDisplay(previous).catch(() => undefined);
    }
    audioManager.setVolumes(previous);
    useAppSettings.setState({ error: String(error) });
    throw error;
  } finally {
    useAppSettings.setState({ saving: false });
  }
}
