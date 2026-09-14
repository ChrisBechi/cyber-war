import { z } from 'zod';

const taskSchema = z.object({
  pid: z.number().int().positive(),
  name: z.string(),
  user: z.string(),
  rssBytes: z.number().nonnegative(),
  groupRssBytes: z.number().nonnegative(),
  cpuPercent: z.number().min(0).max(100),
  groupCpuPercent: z.number().min(0).max(100),
  canTerminate: z.boolean(),
});
export const taskSnapshotSchema = z.object({
  tasks: z.array(taskSchema),
  cpuPercent: z.number().min(0).max(100),
  memoryUsedBytes: z.number().nonnegative(),
  memoryTotalBytes: z.number().positive(),
  metricsKind: z.literal('virtual-budget-v1'),
});
export type Task = z.infer<typeof taskSchema>;
export type TaskSnapshot = z.infer<typeof taskSnapshotSchema>;
export type TaskColumn =
  'name' | 'pid' | 'rssBytes' | 'groupRssBytes' | 'cpuPercent' | 'groupCpuPercent';
type Activity = { task: Task; kind: 'starting' | 'changing' | 'terminating'; until: number };
export type Monitor = {
  snapshot: TaskSnapshot | null;
  history: { cpu: number; memory: number }[];
  activity: Record<number, Activity>;
  error: string;
};
export const emptyMonitor: Monitor = { snapshot: null, history: [], activity: {}, error: '' };
export function acceptSnapshot(previous: Monitor, snapshot: TaskSnapshot, now: number): Monitor {
  const before = new Map(previous.snapshot?.tasks.map((task) => [task.pid, task]));
  const current = new Set(snapshot.tasks.map((task) => task.pid));
  const activity = Object.fromEntries(
    Object.entries(previous.activity).filter(([, a]) => a.until > now),
  );
  for (const task of snapshot.tasks) {
    const old = before.get(task.pid);
    if (previous.snapshot && !old) {
      activity[task.pid] = { task, kind: 'starting', until: now + 2500 };
    } else if (old && JSON.stringify(old) !== JSON.stringify(task)) {
      activity[task.pid] = { task, kind: 'changing', until: now + 2500 };
    }
  }
  for (const task of before.values()) {
    if (!current.has(task.pid)) {
      activity[task.pid] = { task, kind: 'terminating', until: now + 2500 };
    }
  }
  return {
    snapshot,
    activity,
    error: '',
    history: [
      ...previous.history,
      {
        cpu: snapshot.cpuPercent,
        memory: Math.min(100, (snapshot.memoryUsedBytes / snapshot.memoryTotalBytes) * 100),
      },
    ].slice(-120),
  };
}
export function visibleTasks(
  monitor: Monitor,
  query: string,
  pidOnly: boolean,
  allUsers: boolean,
  column: TaskColumn,
  descending: boolean,
) {
  const active = monitor.snapshot?.tasks ?? [];
  const ghosts = Object.values(monitor.activity)
    .filter((a) => a.kind === 'terminating' && !active.some((p) => p.pid === a.task.pid))
    .map((a) => a.task);
  const term = query.trim().toLocaleLowerCase();
  return [...active, ...ghosts]
    .filter(
      (task) =>
        (allUsers || task.user === 'kali') &&
        (pidOnly
          ? String(task.pid).startsWith(term)
          : `${task.name} ${task.pid} ${task.user}`.toLocaleLowerCase().includes(term)),
    )
    .sort((a, b) => {
      const left = a[column],
        right = b[column];
      const order =
        typeof left === 'string' && typeof right === 'string'
          ? left.localeCompare(right, undefined, { numeric: true })
          : Number(left) - Number(right);
      return (descending ? -order : order) || a.pid - b.pid;
    });
}
export const memoryLabel = (bytes: number) =>
  `${(bytes / 1024 ** (bytes >= 1024 ** 3 ? 3 : 2)).toFixed(1)} ${bytes >= 1024 ** 3 ? 'GiB' : 'MiB'}`;
export const percentLabel = (percent: number) => `${Number(percent.toFixed(1))}%`;
export function graphPath(values: number[]) {
  if (!values.length) {
    return '';
  }
  const height = (value: number) => 60 - Math.max(0, Math.min(100, value)) * 0.6;
  return `M ${120 - values.length} 60 ${values.map((value, index) => `L ${120 - values.length + index} ${height(value)}`).join(' ')} L 120 ${height(values[values.length - 1])} L 120 60 Z`;
}
