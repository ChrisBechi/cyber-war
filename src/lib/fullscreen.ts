import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useGame } from './game-store';

let transition = Promise.resolve();

export function toggleFullscreen(): void {
  if (!isTauri()) {
    return;
  }
  transition = transition
    .then(async () => {
      const window = getCurrentWindow();
      await window.setFullscreen(!(await window.isFullscreen()));
    })
    .catch((error: unknown) => {
      useGame.setState({ error: `Tela cheia: ${String(error)}` });
    });
}

export function handleFullscreenKey(event: KeyboardEvent): void {
  if (event.key !== 'F11') {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  if (!event.repeat) {
    toggleFullscreen();
  }
}
