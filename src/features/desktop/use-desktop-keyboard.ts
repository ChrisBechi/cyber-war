import { useEffect } from 'react';
import { useWindows } from '../../lib/window-store';
import type { WindowId } from '../../lib/window-store';
import { desktopKeyboardAction } from './desktop-keyboard';

function focusWindow(id?: WindowId) {
  requestAnimationFrame(() => {
    const element = id
      ? Array.from(document.querySelectorAll<HTMLElement>('[data-window-id]')).find(
          (item) => item.dataset.windowId === id,
        )
      : document.querySelector<HTMLElement>('[data-testid="desktop"]');
    element?.focus({ preventScroll: true });
  });
}

export function useDesktopKeyboard({
  disabled,
  presentation,
  onMenu,
  onDismiss,
  onHelp,
  onLock,
}: {
  disabled: boolean;
  presentation: boolean;
  onMenu: () => void;
  onDismiss: () => void;
  onHelp: () => void;
  onLock: () => void;
}) {
  useEffect(() => {
    if (presentation) {
      return;
    }
    const keydown = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.isComposing) {
        return;
      }
      const action = desktopKeyboardAction(event);
      if (!action) {
        if (!disabled && event.key === 'Escape') {
          onDismiss();
        }
        return;
      }
      const modalOpen = Array.from(document.querySelectorAll('[aria-modal="true"]')).some(
        (element) => !element.closest('[aria-hidden="true"], [hidden], [inert]'),
      );
      if (disabled || modalOpen) {
        event.preventDefault();
        if (action !== 'help') {
          event.stopPropagation();
        }
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat) {
        return;
      }
      const state = useWindows.getState();
      const active = () =>
        state.windows
          .filter((item) => item.workspace === state.workspace && !item.minimized)
          .sort((a, b) => b.z - a.z)[0];
      if (action === 'menu') {
        onMenu();
        return;
      }
      onDismiss();
      switch (action) {
        case 'terminal':
          focusWindow(state.newTerminal());
          break;
        case 'files':
        case 'browser':
        case 'editor':
        case 'settings':
        case 'processes':
          state.open(action);
          focusWindow(action);
          break;
        case 'help':
          onHelp();
          break;
        case 'lock':
          onLock();
          break;
        case 'desktop': {
          state.toggleDesktop();
          const updated = useWindows.getState();
          focusWindow(
            updated.windows
              .filter((item) => item.workspace === updated.workspace && !item.minimized)
              .sort((a, b) => b.z - a.z)[0]?.id,
          );
          break;
        }
        case 'next':
        case 'previous':
          focusWindow(state.cycle(action === 'next' ? 1 : -1));
          break;
        case 'maximize':
        case 'minimize': {
          const current = active();
          if (current) {
            state.update(
              current.id,
              action === 'maximize'
                ? { maximized: !current.maximized, fullscreen: false }
                : { minimized: true },
            );
            if (action === 'minimize') {
              focusWindow(
                useWindows
                  .getState()
                  .windows.filter((item) => item.workspace === state.workspace && !item.minimized)
                  .sort((a, b) => b.z - a.z)[0]?.id,
              );
            }
          }
          break;
        }
        default: {
          const workspace = Number(action.slice(-1));
          useWindows.setState({ workspace });
          focusWindow(
            state.windows
              .filter((item) => item.workspace === workspace && !item.minimized)
              .sort((a, b) => b.z - a.z)[0]?.id,
          );
        }
      }
    };
    window.addEventListener('keydown', keydown, true);
    return () => window.removeEventListener('keydown', keydown, true);
  }, [disabled, presentation, onMenu, onDismiss, onHelp, onLock]);
}
