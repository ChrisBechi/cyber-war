import { useCallback, useEffect, useRef } from 'react';
import { audioManager } from '../../lib/audio-manager';

export function useAdvance(onFinish: () => void, delay: number, keys = ['Escape', 'Enter', ' ']) {
  const done = useRef(false);
  const started = useRef(0);
  const finishRef = useRef(onFinish);
  finishRef.current = onFinish;
  const finish = useCallback(() => {
    if (!done.current) {
      done.current = true;
      finishRef.current();
    }
  }, []);
  const skip = useCallback(() => {
    audioManager.unlock();
    if (performance.now() - started.current >= delay) {
      finish();
    }
  }, [delay, finish]);
  const allowedKeys = keys.join('|');
  useEffect(() => {
    started.current = performance.now();
    const key = (event: KeyboardEvent) => {
      if (
        event.defaultPrevented ||
        event.repeat ||
        event.ctrlKey ||
        event.altKey ||
        event.metaKey ||
        !allowedKeys.split('|').includes(event.key)
      ) {
        return;
      }
      event.preventDefault();
      skip();
    };
    window.addEventListener('keydown', key);
    return () => window.removeEventListener('keydown', key);
  }, [skip, allowedKeys]);
  return { finish, skip };
}
