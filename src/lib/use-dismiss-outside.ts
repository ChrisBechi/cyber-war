import { useEffect } from 'react';

/** Capture also sees presses consumed by a window or terminal below the popup. */
export function useDismissOutside(
  enabled: boolean,
  isInside: (target: Node) => boolean,
  dismiss: () => void,
) {
  useEffect(() => {
    if (!enabled) {
      return;
    }
    const outside = (event: Event) => {
      if (event.target instanceof Node && !isInside(event.target)) {
        dismiss();
      }
    };
    document.addEventListener('pointerdown', outside, true);
    // Keyboard/assistive activation can emit click without a preceding pointer.
    document.addEventListener('click', outside, true);
    return () => {
      document.removeEventListener('pointerdown', outside, true);
      document.removeEventListener('click', outside, true);
    };
  }, [enabled, isInside, dismiss]);
}
