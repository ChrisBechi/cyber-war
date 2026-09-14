import { create } from 'zustand';
import { z } from 'zod';
import { emptySchema, missionSchema, request, worldSchema } from './api';
import type { Mission, World } from './api';

type GameStore = {
  world: World | null;
  missions: Mission[];
  slot: number;
  revision: number;
  error: string;
  busy: boolean;
  sessionPending: boolean;
  refresh: () => Promise<void>;
  clearError: () => void;
};
export const useGame = create<GameStore>((set) => ({
  world: null,
  missions: [],
  slot: 1,
  revision: 0,
  error: '',
  busy: false,
  sessionPending: false,
  clearError: () => set({ error: '' }),
  refresh: async () => {
    const [world, missions] = await Promise.all([
      request('world_get', {}, worldSchema),
      request('mission_get_state', {}, z.array(missionSchema)),
    ]);
    set((state) => ({ world, missions, revision: state.revision + 1 }));
  },
}));

let queue = Promise.resolve();
export function perform<T>(
  command: string,
  args: Record<string, unknown>,
  schema: z.ZodType<T>,
): Promise<T> {
  const result = queue.then(async () => {
    useGame.setState({ busy: true, error: '' });
    try {
      const value = await request(command, args, schema);
      await useGame.getState().refresh();
      return value;
    } catch (error) {
      useGame.setState({ error: String(error) });
      throw error;
    } finally {
      useGame.setState({ busy: false });
    }
  });
  queue = result.then(
    () => undefined,
    () => undefined,
  );
  return result;
}
export function act(command: string, args: Record<string, unknown> = {}): void {
  void perform(command, args, emptySchema).catch(() => undefined);
}
