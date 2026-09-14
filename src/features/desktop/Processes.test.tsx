import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import type * as Api from '../../lib/api';
import type * as Store from '../../lib/game-store';
import { request } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { Processes } from './Processes';
import { acceptSnapshot, emptyMonitor, visibleTasks } from './task-manager-model';
import type { Task, TaskSnapshot } from './task-manager-model';

vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  request: vi.fn(),
}));
vi.mock('../../lib/game-store', async (original) => ({
  ...(await original<typeof Store>()),
  perform: vi.fn(),
}));
const task = (pid: number, name: string, rssBytes = 1024 ** 2): Task => ({
  pid,
  name,
  user: pid === 1 ? 'root' : 'kali',
  rssBytes,
  groupRssBytes: rssBytes,
  cpuPercent: 0,
  groupCpuPercent: 0,
  canTerminate: pid !== 1,
});
const sample = (
  tasks = [task(1, 'session'), task(22, 'zeta'), task(3, 'alpha', 2 * 1024 ** 2)],
): TaskSnapshot => ({
  tasks,
  cpuPercent: 3,
  memoryUsedBytes: 512 * 1024 ** 2,
  memoryTotalBytes: 8 * 1024 ** 3,
  metricsKind: 'virtual-budget-v1',
});
beforeEach(() => {
  useGame.setState({ revision: 0 });
  vi.mocked(request).mockResolvedValue(sample());
  vi.mocked(perform).mockResolvedValue(null);
});
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
  vi.useRealTimers();
});

it('renders the reference columns, live summary, two graphs and legend without screenshot processes', async () => {
  render(<Processes />);
  await screen.findByText('alpha');
  expect(
    screen.getAllByRole('columnheader').map((cell) => cell.textContent?.replace('▾', '')),
  ).toEqual(['Task', 'PID', 'RSS', 'Group RSS', 'CPU', 'Group CPU']);
  expect(screen.getByText('Processes: 3')).toBeInTheDocument();
  expect(screen.getByText('CPU: 3%')).toBeInTheDocument();
  expect(screen.getAllByRole('img')).toHaveLength(2);
  expect(screen.getByText('Terminating task')).toBeInTheDocument();
  expect(screen.queryByText('agent')).toBeNull();
});
it('sorts numerically, filters by name/PID and keeps internal options open', async () => {
  const { container } = render(<Processes />);
  await screen.findByText('alpha');
  fireEvent.click(screen.getByRole('button', { name: 'PID' }));
  expect(
    [...container.querySelectorAll('tbody tr')].map((row) => row.getAttribute('data-task-pid')),
  ).toEqual(['1', '3', '22']);
  fireEvent.change(screen.getByRole('textbox', { name: 'Pesquisar processos' }), {
    target: { value: 'ZE' },
  });
  expect(screen.queryByText('alpha')).toBeNull();
  expect(screen.getByText('zeta')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Pesquisar por PID' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Pesquisar processos' }), {
    target: { value: '3' },
  });
  expect(screen.getByText('alpha')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Opções do Task Manager' }));
  fireEvent.click(screen.getByRole('checkbox', { name: 'Mostrar legenda de estados' }));
  expect(screen.queryByText('Starting task')).toBeNull();
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  fireEvent.pointerDown(document.body);
  expect(screen.queryByRole('dialog')).toBeNull();
});
it('requires confirmation, checks permissions, preserves failed confirmation and retries', async () => {
  render(<Processes />);
  await screen.findByText('alpha');
  fireEvent.contextMenu(screen.getByText('session'));
  expect(screen.getByRole('menuitem', { name: 'Encerrar processo' })).toBeDisabled();
  fireEvent.pointerDown(document.body);
  fireEvent.contextMenu(screen.getByText('alpha'));
  fireEvent.click(screen.getByRole('menuitem', { name: 'Encerrar processo' }));
  expect(perform).not.toHaveBeenCalled();
  expect(screen.getByRole('button', { name: 'Cancelar' })).toHaveFocus();
  vi.mocked(perform).mockRejectedValueOnce(new Error('permission denied'));
  fireEvent.click(screen.getByRole('button', { name: 'Encerrar' }));
  await screen.findByRole('alert');
  expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  vi.mocked(request).mockResolvedValue(sample([task(1, 'session'), task(22, 'zeta')]));
  fireEvent.click(
    within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Encerrar' }),
  );
  await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull());
  expect(perform).toHaveBeenLastCalledWith(
    'task_manager_terminate',
    { pid: 3, expectedName: 'alpha', force: false },
    expect.anything(),
  );
  await waitFor(() =>
    expect(screen.getByText('alpha').closest('tr')).toHaveAttribute('data-activity', 'terminating'),
  );
});
it('ignores responses after unmount and cancels polling', async () => {
  let resolve!: (value: unknown) => void;
  vi.mocked(request).mockImplementation(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const { unmount } = render(<Processes />);
  await waitFor(() => expect(resolve).toBeDefined());
  unmount();
  await act(async () => {
    resolve(sample());
    await Promise.resolve();
  });
  expect(screen.queryByText('alpha')).toBeNull();
  expect(request).toHaveBeenCalledOnce();
});
it('tracks actual added, changed and removed tasks and bounds graph history', () => {
  let state = acceptSnapshot(emptyMonitor, sample([task(1, 'session')]), 0);
  expect(state.activity).toEqual({});
  state = acceptSnapshot(state, sample([task(1, 'session'), task(2, 'work')]), 10);
  expect(state.activity[2].kind).toBe('starting');
  state = acceptSnapshot(state, sample([task(1, 'session'), task(2, 'work', 2)]), 20);
  expect(state.activity[2].kind).toBe('changing');
  state = acceptSnapshot(state, sample([task(1, 'session')]), 30);
  expect(state.activity[2].kind).toBe('terminating');
  expect(visibleTasks(state, '', false, true, 'pid', false)).toHaveLength(2);
  for (let n = 0; n < 130; n++) {
    state = acceptSnapshot(state, sample([task(1, 'session')]), 3000 + n);
  }
  expect(state.history).toHaveLength(120);
  expect(state.activity).toEqual({});
});
