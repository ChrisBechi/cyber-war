import type { AppSettings } from '../../lib/app-settings';

export type BootStage = 'studio_intro' | 'opening_cinematic' | 'press_start' | 'main_menu';
export function initialBootStage(
  settings: Pick<AppSettings, 'skipStudioIntro' | 'skipTrailer'>,
): BootStage {
  return settings.skipStudioIntro
    ? settings.skipTrailer
      ? 'press_start'
      : 'opening_cinematic'
    : 'studio_intro';
}
export function nextBootStage(stage: BootStage, skipTrailer: boolean): BootStage {
  return stage === 'studio_intro'
    ? skipTrailer
      ? 'press_start'
      : 'opening_cinematic'
    : stage === 'opening_cinematic'
      ? 'press_start'
      : 'main_menu';
}
