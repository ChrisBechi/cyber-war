import { useEffect } from 'react';
import { z } from 'zod';
import { request } from '../../../lib/api';
import { useGame } from '../../../lib/game-store';

export function useWebClock(enabled: boolean) {
  useEffect(() => {
    if (!enabled) {
      return;
    }
    let active = true;
    let pending = false;
    const timer = window.setInterval(() => {
      if (pending) {
        return;
      }
      pending = true;
      void request('web_tick', {}, z.boolean())
        .then(async (changed) => {
          if (active && changed) {
            await useGame.getState().refresh();
          }
        })
        .catch((error: unknown) => {
          if (active) {
            useGame.setState({ error: String(error) });
          }
        })
        .finally(() => {
          pending = false;
        });
    }, 10000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [enabled]);
}
