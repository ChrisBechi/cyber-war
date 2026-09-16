import { expect, it, vi } from 'vitest';
import { z } from 'zod';
import { request } from './api';
import { perform, useGame } from './game-store';
import type * as ApiModule from './api';

vi.mock('./api', async (original) => ({
  ...(await original<typeof ApiModule>()),
  request: vi.fn(),
}));

it('never starts a queued terminal command after its window cancels it', async () => {
  vi.mocked(request).mockReset();
  let finishFirst!: () => void;
  const firstReply = new Promise<null>((resolve) => {
    finishFirst = () => resolve(null);
  });
  vi.mocked(request).mockImplementationOnce(() => firstReply);
  const refresh = useGame.getState().refresh;
  useGame.setState({ refresh: () => Promise.resolve() });
  try {
    const first = perform('first', {}, z.null());
    const controller = new AbortController();
    const cancelled = perform('execute_terminal', { command: 'cat' }, z.null(), {
      signal: controller.signal,
    });
    const rejected = expect(cancelled).rejects.toMatchObject({ name: 'AbortError' });
    controller.abort();
    finishFirst();
    await first;
    await rejected;
    expect(request).toHaveBeenCalledTimes(1);
    expect(request).toHaveBeenCalledWith('first', {}, expect.anything());
  } finally {
    useGame.setState({ refresh });
  }
});

it('cancels waiting shell input before a session transition enters the mutation queue', async () => {
  vi.mocked(request).mockReset();
  let finishFirst!: () => void;
  const firstReply = new Promise<null>((resolve) => {
    finishFirst = () => resolve(null);
  });
  vi.mocked(request).mockImplementation((command) =>
    command === 'execute_terminal' ? firstReply : Promise.resolve(null),
  );
  const refresh = useGame.getState().refresh;
  useGame.setState({ refresh: () => Promise.resolve() });
  try {
    const first = perform('execute_terminal', { command: 'cat' }, z.null());
    await vi.waitFor(() =>
      expect(request).toHaveBeenCalledWith(
        'execute_terminal',
        { command: 'cat' },
        expect.anything(),
      ),
    );
    const ending = perform('end_session', {}, z.null());
    expect(request).toHaveBeenLastCalledWith('terminal_cancel_all', {}, expect.anything());
    expect(vi.mocked(request).mock.calls.some(([command]) => command === 'end_session')).toBe(
      false,
    );
    finishFirst();
    await first;
    await ending;
    expect(request).toHaveBeenLastCalledWith('end_session', {}, expect.anything());
  } finally {
    useGame.setState({ refresh });
  }
});
