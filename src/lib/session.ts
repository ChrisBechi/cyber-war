import { emptySchema, worldSchema } from './api';
import { perform, useGame } from './game-store';
import { useWindows } from './window-store';
import { useVfsClipboard } from './vfs-clipboard';
import type { BuiltinAppId } from './window-store';

const autostartApps: BuiltinAppId[] = ['terminal', 'files', 'editor', 'browser', 'messages'];

function parseAutostart(value: string | undefined): BuiltinAppId[] {
  try {
    const parsed: unknown = JSON.parse(value ?? '[]');
    return Array.isArray(parsed)
      ? [...new Set(parsed)].filter(
          (item): item is BuiltinAppId =>
            typeof item === 'string' && autostartApps.includes(item as BuiltinAppId),
        )
      : [];
  } catch {
    return [];
  }
}

let pending: Promise<void> | undefined;

function transitionSession(command: 'end_session' | 'session_start', navigate: () => void) {
  // Coalesce repeated clicks and block competing navigation until IPC and refresh finish.
  if (pending) {
    return pending;
  }
  useGame.setState({ sessionPending: true });
  pending = (async () => {
    try {
      const world =
        command === 'session_start'
          ? await perform(command, {}, worldSchema)
          : await perform(command, {}, emptySchema);
      useWindows.getState().reset();
      useVfsClipboard.getState().clear();
      if (world) {
        parseAutostart(world.settings.autostartApps).forEach((id) =>
          useWindows.getState().open(id),
        );
      }
      navigate();
    } finally {
      pending = undefined;
      useGame.setState({ sessionPending: false });
    }
  })();
  return pending;
}

export function endSession(navigate: () => void): Promise<void> {
  return transitionSession('end_session', navigate);
}

export function startSession(navigate: () => void): Promise<void> {
  return transitionSession('session_start', navigate);
}
