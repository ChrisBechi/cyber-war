import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { emptySchema, request } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { useDismissOutside } from '../../lib/use-dismiss-outside';
import { ChromeIcon } from './ChromeIcon';
import {
  acceptSnapshot,
  emptyMonitor,
  graphPath,
  memoryLabel,
  percentLabel,
  taskSnapshotSchema,
  visibleTasks,
} from './task-manager-model';
import type { Task, TaskColumn } from './task-manager-model';
import './task-manager.css';

const columns: [TaskColumn, string][] = [
  ['name', 'Task'],
  ['pid', 'PID'],
  ['rssBytes', 'RSS'],
  ['groupRssBytes', 'Group RSS'],
  ['cpuPercent', 'CPU'],
  ['groupCpuPercent', 'Group CPU'],
];
const metricHelp =
  'Orçamentos simulados do LifeOS, não medições do Windows. Cada processo virtual é seu próprio grupo.';

export function Processes() {
  const revision = useGame((s) => s.revision);
  const [monitor, setMonitor] = useState(emptyMonitor);
  const [query, setQuery] = useState('');
  const [pidOnly, setPidOnly] = useState(false);
  const [allUsers, setAllUsers] = useState(true);
  const [legend, setLegend] = useState(true);
  const [interval, setIntervalMs] = useState(1000);
  const [refresh, setRefresh] = useState(0);
  const [sort, setSort] = useState<{ column: TaskColumn; descending: boolean }>({
    column: 'name',
    descending: false,
  });
  const [settings, setSettings] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);
  const [context, setContext] = useState<{ task: Task; x: number; y: number } | null>(null);
  const [confirm, setConfirm] = useState<{ task: Task; force: boolean } | null>(null);
  const [actionError, setActionError] = useState('');
  const [pending, setPending] = useState(false);
  const search = useRef<HTMLInputElement>(null);
  const settingsButton = useRef<HTMLButtonElement>(null);
  const settingsMenu = useRef<HTMLDivElement>(null);
  const contextMenu = useRef<HTMLDivElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  useDismissOutside(
    settings,
    (target) =>
      !!settingsMenu.current?.contains(target) || !!settingsButton.current?.contains(target),
    () => setSettings(false),
  );
  useDismissOutside(
    !!context,
    (target) => !!contextMenu.current?.contains(target),
    () => setContext(null),
  );
  useEffect(() => {
    let current = true;
    let reading = false;
    const poll = async () => {
      if (reading) {
        return;
      }
      reading = true;
      try {
        const snapshot = await request('task_manager_snapshot', {}, taskSnapshotSchema);
        if (current) {
          setMonitor((previous) => acceptSnapshot(previous, snapshot, Date.now()));
        }
      } catch (error) {
        if (current) {
          setMonitor((previous) => ({ ...previous, error: String(error) }));
        }
      } finally {
        reading = false;
      }
    };
    void poll();
    const timer = interval
      ? window.setInterval(() => {
          void poll();
        }, interval)
      : undefined;
    return () => {
      current = false;
      window.clearInterval(timer);
    };
  }, [revision, interval, refresh]);
  useEffect(() => {
    if (confirm) {
      cancel.current?.focus();
    }
  }, [confirm]);
  useEffect(() => {
    const marks = Object.values(monitor.activity);
    if (!marks.length) {
      return;
    }
    const timer = window.setTimeout(
      () =>
        setMonitor((previous) => ({
          ...previous,
          activity: Object.fromEntries(
            Object.entries(previous.activity).filter(([, a]) => a.until > Date.now()),
          ),
        })),
      Math.max(0, Math.min(...marks.map((a) => a.until)) - Date.now()) + 5,
    );
    return () => window.clearTimeout(timer);
  }, [monitor.activity]);
  const tasks = visibleTasks(monitor, query, pidOnly, allUsers, sort.column, sort.descending);
  const snapshot = monitor.snapshot;
  const canTerminate = (task: Task) =>
    task.canTerminate && !!snapshot?.tasks.some((p) => p.pid === task.pid);
  const confirmTermination = (task: Task, force: boolean) => {
    setContext(null);
    setActionError('');
    if (canTerminate(task)) {
      setConfirm({ task, force });
    }
  };
  const terminate = async () => {
    if (!confirm || pending) {
      return;
    }
    setPending(true);
    try {
      await perform(
        'task_manager_terminate',
        { pid: confirm.task.pid, expectedName: confirm.task.name, force: confirm.force },
        emptySchema,
      );
      setConfirm(null);
      setRefresh((value) => value + 1);
    } catch (error) {
      setActionError(String(error));
    } finally {
      setPending(false);
    }
  };
  const showActions = (task: Task, x: number, y: number) => {
    setSelected(task.pid);
    setContext({
      task,
      x: Math.max(4, Math.min(x, window.innerWidth - 202)),
      y: Math.max(4, Math.min(y, window.innerHeight - 80)),
    });
  };
  return (
    <div
      className="task-manager"
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          setContext(null);
          setSettings(false);
          if (!pending) {
            setConfirm(null);
          }
        }
        if (event.ctrlKey && event.key.toLowerCase() === 'f') {
          event.preventDefault();
          search.current?.focus();
        }
      }}
    >
      <div className="task-manager-toolbar">
        <button
          ref={settingsButton}
          aria-label="Opções do Task Manager"
          aria-expanded={settings}
          title="Opções"
          onClick={() => setSettings((value) => !value)}
        >
          <ChromeIcon name="settings" />
        </button>
        <button
          aria-label="Pesquisar por PID"
          aria-pressed={pidOnly}
          title="Pesquisar por PID"
          onClick={() => {
            setPidOnly((value) => !value);
            search.current?.focus();
          }}
        >
          <ChromeIcon name="target" />
        </button>
        <div className="task-manager-search">
          <ChromeIcon name="search" />
          <input
            ref={search}
            aria-label="Pesquisar processos"
            autoComplete="off"
            spellCheck={false}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={pidOnly ? 'PID' : ''}
          />
          <button
            aria-label="Limpar pesquisa"
            disabled={!query}
            onClick={() => {
              setQuery('');
              search.current?.focus();
            }}
          >
            <ChromeIcon name="close" />
          </button>
        </div>
      </div>
      {settings && (
        <div
          ref={settingsMenu}
          className="task-manager-options"
          role="dialog"
          aria-label="Opções do Task Manager"
        >
          <label>
            <input
              type="checkbox"
              checked={allUsers}
              onChange={(e) => setAllUsers(e.target.checked)}
            />
            Mostrar processos de todos os usuários
          </label>
          <label>
            <input type="checkbox" checked={legend} onChange={(e) => setLegend(e.target.checked)} />
            Mostrar legenda de estados
          </label>
          <label>
            Atualização
            <select
              aria-label="Intervalo de atualização"
              value={interval}
              onChange={(e) => setIntervalMs(Number(e.target.value))}
            >
              <option value="1000">1 segundo</option>
              <option value="2000">2 segundos</option>
              <option value="5000">5 segundos</option>
              <option value="0">Manual</option>
            </select>
          </label>
          <button
            onClick={() => {
              setRefresh((value) => value + 1);
              setSettings(false);
            }}
          >
            Atualizar agora
          </button>
          <p>{metricHelp}</p>
        </div>
      )}
      <div className="task-manager-graphs" title={metricHelp}>
        {(['cpu', 'memory'] as const).map((kind) => (
          <svg
            key={kind}
            className={`task-manager-graph task-manager-${kind}`}
            viewBox="0 0 120 60"
            preserveAspectRatio="none"
            role="img"
            aria-label={
              kind === 'cpu' ? 'Histórico de CPU virtual' : 'Histórico de memória virtual'
            }
          >
            {[15, 30, 45].map((y) => (
              <line
                key={y}
                x1="0"
                x2="120"
                y1={y}
                y2={y}
                className="task-manager-grid"
                vectorEffect="non-scaling-stroke"
              />
            ))}
            <path
              d={graphPath(monitor.history.map((sample) => sample[kind]))}
              vectorEffect="non-scaling-stroke"
            />
          </svg>
        ))}
      </div>
      <div className="task-manager-summary" title={metricHelp}>
        <span className="task-manager-cpu">
          CPU: {snapshot ? percentLabel(snapshot.cpuPercent) : '—'}
        </span>
        <span>Processes: {snapshot?.tasks.length ?? '—'}</span>
        <span className="task-manager-memory">
          Memory:{' '}
          {snapshot
            ? `${Math.round((snapshot.memoryUsedBytes / snapshot.memoryTotalBytes) * 100)}% (${memoryLabel(snapshot.memoryUsedBytes)} / ${memoryLabel(snapshot.memoryTotalBytes)})`
            : '—'}
        </span>
      </div>
      <div className="task-manager-list">
        <table aria-label="Processos virtuais">
          <colgroup>
            {columns.map(([column]) => (
              <col key={column} className={`task-manager-col-${column}`} />
            ))}
          </colgroup>
          <thead>
            <tr>
              {columns.map(([column, label]) => (
                <th
                  key={column}
                  scope="col"
                  aria-sort={
                    sort.column === column ? (sort.descending ? 'descending' : 'ascending') : 'none'
                  }
                >
                  <button
                    onClick={() =>
                      setSort({ column, descending: sort.column === column && !sort.descending })
                    }
                  >
                    {label}
                    {sort.column === column && (
                      <span aria-hidden="true">{sort.descending ? '▴' : '▾'}</span>
                    )}
                  </button>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {tasks.map((task) => (
              <tr
                key={task.pid}
                tabIndex={0}
                aria-selected={selected === task.pid}
                className={selected === task.pid ? 'is-selected' : ''}
                data-task-pid={task.pid}
                data-activity={monitor.activity[task.pid]?.kind}
                onClick={() => setSelected(task.pid)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' || event.key === ' ') {
                    event.preventDefault();
                    setSelected(task.pid);
                  }
                  if (event.key === 'Delete') {
                    event.preventDefault();
                    confirmTermination(task, false);
                  }
                  if (event.key === 'ContextMenu' || (event.shiftKey && event.key === 'F10')) {
                    event.preventDefault();
                    const rect = event.currentTarget.getBoundingClientRect();
                    showActions(task, rect.left + 15, rect.top + 18);
                  }
                }}
                onContextMenu={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  showActions(task, event.clientX, event.clientY);
                }}
              >
                <td title={`${task.name} · ${task.user}`}>{task.name}</td>
                <td>{task.pid}</td>
                <td>{memoryLabel(task.rssBytes)}</td>
                <td>{memoryLabel(task.groupRssBytes)}</td>
                <td>{percentLabel(task.cpuPercent)}</td>
                <td>{percentLabel(task.groupCpuPercent)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {!tasks.length && (
          <p className="task-manager-empty">
            {snapshot
              ? 'Nenhum processo encontrado.'
              : monitor.error
                ? 'Não foi possível carregar os processos.'
                : 'Carregando processos…'}
          </p>
        )}
      </div>
      {monitor.error && (
        <div className="task-manager-error" role="alert">
          {monitor.error}
          <button onClick={() => setRefresh((value) => value + 1)}>Tentar novamente</button>
        </div>
      )}
      {legend && (
        <div className="task-manager-legend">
          <span>
            <i className="starting" />
            Starting task
          </span>
          <span>
            <i className="changing" />
            Changing task
          </span>
          <span>
            <i className="terminating" />
            Terminating task
          </span>
        </div>
      )}
      {context &&
        createPortal(
          <div
            ref={contextMenu}
            role="menu"
            aria-label="Ações do processo"
            className="task-manager-actions"
            style={{ left: context.x, top: context.y }}
          >
            <button
              role="menuitem"
              disabled={!canTerminate(context.task)}
              onClick={() => confirmTermination(context.task, false)}
            >
              Encerrar processo
            </button>
            <button
              role="menuitem"
              disabled={!canTerminate(context.task)}
              onClick={() => confirmTermination(context.task, true)}
            >
              Forçar encerramento
            </button>
          </div>,
          document.body,
        )}
      {confirm && (
        <div className="task-manager-confirm-backdrop">
          <div
            role="alertdialog"
            aria-modal="true"
            aria-label="Encerrar processo"
            className="task-manager-confirm"
            onKeyDown={(event) => {
              if (event.key === 'Tab') {
                const buttons = Array.from(
                  event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'),
                );
                const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
                event.preventDefault();
                buttons[
                  (current + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length
                ]?.focus();
              }
            }}
          >
            <strong>{confirm.force ? 'Forçar encerramento?' : 'Encerrar processo?'}</strong>
            <p>
              {confirm.task.name} · PID {confirm.task.pid}
            </p>
            <p>Somente o processo virtual será encerrado. Trabalho não salvo pode ser perdido.</p>
            {actionError && <p role="alert">{actionError}</p>}
            <footer>
              <button ref={cancel} disabled={pending} onClick={() => setConfirm(null)}>
                Cancelar
              </button>
              <button
                disabled={pending}
                onClick={() => {
                  void terminate();
                }}
              >
                {pending ? 'Encerrando…' : 'Encerrar'}
              </button>
            </footer>
          </div>
        </div>
      )}
    </div>
  );
}
