import { useEffect, useLayoutEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { WindowState } from '../../lib/window-store';

type Box = { x: number; y: number; width: number; height: number };
const bounds = (element: HTMLElement): Box => ({
  x: element.offsetLeft,
  y: element.offsetTop,
  width: element.offsetWidth,
  height: element.offsetHeight,
});

/** Resize layout once, then let the compositor move/scale the existing window. */
export function useWindowGeometry(
  container: RefObject<HTMLElement | null>,
  model: WindowState,
  shown: boolean,
  interacting: boolean,
) {
  const previous = useRef<{
    box: Box;
    maximized: boolean;
    fullscreen: boolean;
    shown: boolean;
  } | null>(null);
  const running = useRef<{ animation: Animation; from: Box; to: Box } | null>(null);

  useLayoutEffect(() => {
    const element = container.current;
    if (!element) {
      return;
    }
    const last = previous.current;
    const next = bounds(element);
    previous.current = {
      box: next,
      maximized: model.maximized,
      fullscreen: !!model.fullscreen,
      shown,
    };
    let from = last?.box;
    const active = running.current;
    if (active) {
      // Reversing midway starts at the current visual bounds, avoiding a snap.
      const progress =
        active.animation.effect?.getComputedTiming().progress ??
        (active.animation.playState === 'finished' ? 1 : 0);
      const blend = (key: keyof Box) =>
        active.from[key] + (active.to[key] - active.from[key]) * progress;
      from = { x: blend('x'), y: blend('y'), width: blend('width'), height: blend('height') };
      active.animation.cancel();
      running.current = null;
      delete element.dataset.windowSizing;
    }
    if (
      !from ||
      !last ||
      last.maximized === model.maximized ||
      !last.shown ||
      !shown ||
      last.fullscreen ||
      model.fullscreen ||
      interacting ||
      !next.width ||
      !next.height ||
      !from.width ||
      !from.height ||
      !element.animate ||
      element.closest('.reduce-motion') ||
      window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
    ) {
      return;
    }

    element.dataset.windowSizing = 'true';
    const animation = element.animate(
      [
        {
          transform: `translate(${from.x - next.x}px, ${from.y - next.y}px) scale(${from.width / next.width}, ${from.height / next.height})`,
          transformOrigin: '0 0',
        },
        { transform: 'translate(0, 0) scale(1, 1)', transformOrigin: '0 0' },
      ],
      { duration: 280, easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)', fill: 'both' },
    );
    running.current = { animation, from, to: next };
    animation.onfinish = () => {
      if (running.current?.animation === animation) {
        running.current = null;
        animation.cancel();
        delete element.dataset.windowSizing;
      }
    };
  }, [
    container,
    model.maximized,
    model.fullscreen,
    model.x,
    model.y,
    model.width,
    model.height,
    shown,
    interacting,
  ]);

  useEffect(() => {
    const element = container.current;
    const resize = () => {
      running.current?.animation.cancel();
      running.current = null;
      if (element) {
        delete element.dataset.windowSizing;
        if (previous.current) {
          previous.current.box = bounds(element);
        }
      }
    };
    window.addEventListener('resize', resize);
    return () => {
      window.removeEventListener('resize', resize);
      running.current?.animation.cancel();
      running.current = null;
      if (element) {
        delete element.dataset.windowSizing;
      }
    };
  }, [container]);
}
