import { useEffect, useState } from 'react';
import { z } from 'zod';
import { desktopRuntime, request } from './api';

const batterySchema = z
  .object({
    percent: z.number().int().min(0).max(100).nullable(),
    charging: z.boolean(),
    pluggedIn: z.boolean(),
  })
  .nullable();

export function useHostBattery() {
  const [battery, setBattery] = useState<z.infer<typeof batterySchema>>(null);
  useEffect(() => {
    if (!desktopRuntime) {
      return;
    }
    let active = true;
    let pending = false;
    const refresh = async () => {
      if (pending || document.hidden) {
        return;
      }
      pending = true;
      try {
        const next = await request('host_battery_status', {}, batterySchema);
        if (active) {
          setBattery(next);
        }
      } catch {
        if (active) {
          setBattery(null);
        }
      } finally {
        pending = false;
      }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), 30000);
    const onVisible = () => void refresh();
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      active = false;
      window.clearInterval(timer);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, []);
  return battery;
}
