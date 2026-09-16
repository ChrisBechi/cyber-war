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
  options?: { signal: AbortSignal },
): Promise<T> {
  // Session transitions must reach waiting shells before the mutation queue.
  const cancellation = [
    'end_session',
    'load_slot',
    'restore_checkpoint',
    'quit_game',
    'new_game',
  ].includes(command)
    ? request('terminal_cancel_all', {}, emptySchema)
    : Promise.resolve();
  const result = queue.then(async () => {
    await cancellation;
    options?.signal.throwIfAborted();
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
  // A rejected cancellation is reported by result, even while another mutation runs.
  void cancellation.catch(() => undefined);
  queue = result.then(
    () => undefined,
    () => undefined,
  );
  return result;
}
export function act(command: string, args: Record<string, unknown> = {}): void {
  void perform(command, args, emptySchema).catch(() => undefined);
}
