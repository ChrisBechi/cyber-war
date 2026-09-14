import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { InstallerList, InstallerProgress } from './InstallerWidgets';

afterEach(() => vi.useRealTimers());
describe('InstallerProgress', () => {
  it('reveals a fixed fill from 0 to 100 and waits at the end before completing', async () => {
    vi.useFakeTimers();
    const done = vi.fn();
    const { container, rerender } = render(
      <InstallerProgress
        label="Formatando"
        details={['Iniciando', 'Concluído']}
        onComplete={done}
      />,
    );
    const track = screen.getByRole('progressbar');
    const fill = container.querySelector('.installer-progress-fill');
    expect(track).toHaveAttribute('aria-valuenow', '0');
    expect(fill).toHaveStyle({ clipPath: 'inset(0 100% 0 0)' });
    await act(() => vi.advanceTimersByTimeAsync(1904));
    expect(track).toHaveAttribute('aria-valuenow', '50');
    expect(fill?.getAttribute('style')).not.toMatch(/transform|width|scale/);
    rerender(
      <InstallerProgress
        label="Formatando"
        details={['Iniciando', 'Concluído']}
        onComplete={done}
      />,
    );
    await act(() => vi.advanceTimersByTimeAsync(1896));
    expect(track).toHaveAttribute('aria-valuenow', '100');
    expect(fill).toHaveStyle({ clipPath: 'inset(0 0% 0 0)' });
    expect(done).not.toHaveBeenCalled();
    await act(() => vi.advanceTimersByTimeAsync(199));
    expect(done).not.toHaveBeenCalled();
    await act(() => vi.advanceTimersByTimeAsync(1));
    expect(done).toHaveBeenCalledOnce();
  });
  it('cleans up all stage timers on cancellation', async () => {
    vi.useFakeTimers();
    const done = vi.fn();
    const { unmount } = render(
      <InstallerProgress label="Rede" details={['DHCP']} onComplete={done} />,
    );
    await act(() => vi.advanceTimersByTimeAsync(2000));
    unmount();
    await act(() => vi.advanceTimersByTimeAsync(10000));
    expect(done).not.toHaveBeenCalled();
    expect(vi.getTimerCount()).toBe(0);
  });
});
it('selects list options by keyboard and activates only when requested', () => {
  const change = vi.fn();
  const activate = vi.fn();
  render(
    <InstallerList
      label="Rede"
      options={[
        ['eth0', 'Cabo'],
        ['wlan0', 'Wi-Fi'],
      ]}
      value="eth0"
      onChange={change}
      onActivate={activate}
    />,
  );
  fireEvent.keyDown(screen.getByRole('listbox'), { key: 'ArrowDown' });
  expect(change).toHaveBeenCalledWith('wlan0');
  expect(activate).not.toHaveBeenCalled();
  fireEvent.doubleClick(screen.getByRole('option', { name: 'Wi-Fi' }));
  expect(activate).toHaveBeenCalledWith('wlan0');
});
