import { renderHook } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import * as api from '../../../lib/api';
import { useGame } from '../../../lib/game-store';
import { useWebClock } from './use-web-clock';

const originalRefresh = useGame.getState().refresh;
const refresh = vi.fn(() => Promise.resolve());
beforeEach(() => {
  vi.useFakeTimers();
  vi.spyOn(api, 'request').mockResolvedValue(false);
  useGame.setState({ refresh, error: '' });
});
afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
  refresh.mockClear();
  useGame.setState({ refresh: originalRefresh, error: '' });
});

it('refreshes open views only when the native clock reports a committed change', async () => {
  vi.mocked(api.request).mockResolvedValueOnce(false).mockResolvedValueOnce(true);
  const view = renderHook(() => useWebClock(true));
  await vi.advanceTimersByTimeAsync(10000);
  expect(refresh).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(10000);
  expect(refresh).toHaveBeenCalledTimes(1);
  expect(api.request).toHaveBeenCalledWith('web_tick', {}, expect.anything());
  view.unmount();
});

it('does not overlap pending ticks or refresh an unmounted session', async () => {
  let finish!: (value: boolean) => void;
  vi.mocked(api.request).mockReturnValue(
    new Promise<boolean>((resolve) => {
      finish = resolve;
    }),
  );
  const view = renderHook(() => useWebClock(true));
  await vi.advanceTimersByTimeAsync(30000);
  expect(api.request).toHaveBeenCalledTimes(1);
  view.unmount();
  finish(true);
  await vi.advanceTimersByTimeAsync(10000);
  expect(refresh).not.toHaveBeenCalled();
  expect(api.request).toHaveBeenCalledTimes(1);
});

it('stops when disabled and reports a failed transaction without a false refresh', async () => {
  vi.mocked(api.request).mockRejectedValueOnce(new Error('Falha ao salvar evento'));
  const view = renderHook(({ enabled }) => useWebClock(enabled), {
    initialProps: { enabled: false },
  });
  await vi.advanceTimersByTimeAsync(10000);
  expect(api.request).not.toHaveBeenCalled();
  view.rerender({ enabled: true });
  await vi.advanceTimersByTimeAsync(10000);
  expect(useGame.getState().error).toContain('Falha ao salvar evento');
  expect(refresh).not.toHaveBeenCalled();
  view.rerender({ enabled: false });
  await vi.advanceTimersByTimeAsync(10000);
  expect(api.request).toHaveBeenCalledTimes(1);
  view.unmount();
});
