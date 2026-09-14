import { useEffect, useRef, useState } from 'react';
import { act, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { audioManager } from '../../lib/audio-manager';
import { softwareById } from '../../lib/software-catalog';
import type { SoftwareEntry } from '../../lib/software-catalog';
import { KaliIcon } from './KaliIcon';
import { useDismissOutside } from '../../lib/use-dismiss-outside';

type Props = {
  launcher: boolean;
  onToggle: () => void;
  onLaunch: (entry: SoftwareEntry) => void;
  onLock: () => void;
  onLogout: () => void;
};

export function KaliPanel({ launcher, onToggle, onLaunch, onLock, onLogout }: Props) {
  const { world, revision, busy } = useGame();
  const { workspace, windows, open } = useWindows();
  const [time, setTime] = useState(new Date());
  const [popover, setPopover] = useState('');
  const [volume, setVolume] = useState(Number(world?.settings.volume ?? 70));
  const [activity, setActivity] = useState<number[]>(Array.from({ length: 64 }, () => 1));
  const panel = useRef<HTMLElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const lastRevision = useRef(revision);
  const current = useRef({ revision, busy });
  current.current = { revision, busy };
  const savedVolume = world?.settings.volume;
  useEffect(() => {
    if (savedVolume === undefined) {
      return;
    }
    const next = Number(savedVolume);
    setVolume(next);
    audioManager.setVolumes({ ...audioManager.getVolumes(), master: next });
  }, [savedVolume]);
  useEffect(() => {
    const interval = window.setInterval(() => {
      setTime(new Date());
      const active = current.current;
      const sample = active.busy ? 95 : active.revision !== lastRevision.current ? 55 : 1;
      lastRevision.current = active.revision;
      setActivity((values) => [...values.slice(1), sample]);
    }, 1000);
    return () => window.clearInterval(interval);
  }, []);
  useDismissOutside(
    !!popover,
    (target) =>
      !!popup.current?.contains(target) ||
      !!panel.current?.querySelector(`[data-panel-menu="${popover}"]`)?.contains(target),
    () => setPopover(''),
  );
  if (!world) {
    return null;
  }
  const launch = (id: string) => {
    const entry = softwareById.get(id);
    if (entry) {
      onLaunch(entry);
    }
    setPopover('');
  };
  const toggle = (id: string) => setPopover((value) => (value === id ? '' : id));
  const unread = world.messages.filter((message) => !message.read).length;
  const showDesktop = () => {
    const currentWindows = windows.filter((window) => window.workspace === workspace);
    const minimize = currentWindows.some((window) => !window.minimized);
    currentWindows.forEach((window) =>
      useWindows.getState().update(window.id, { minimized: minimize }),
    );
  };
  return (
    <header
      ref={panel}
      className="top-panel kali-panel"
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          setPopover('');
        }
      }}
    >
      <button
        data-launcher-toggle
        aria-label="Abrir menu de aplicativos"
        aria-expanded={launcher}
        aria-pressed={launcher}
        aria-haspopup="true"
        className={`kali-launcher-button ${launcher ? 'selected' : ''}`}
        onClick={() => {
          setPopover('');
          onToggle();
        }}
      >
        <KaliIcon name="kali-panel-menu" size={26} />
      </button>
      <span className="panel-separator" />
      <div className="panel-launchers">
        <button
          title="Mostrar área de trabalho"
          aria-label="Mostrar área de trabalho"
          onClick={showDesktop}
        >
          <KaliIcon name="user-desktop" size={24} />
        </button>
        <button
          title="File Manager"
          aria-label="File Manager"
          onClick={() => launch('file-manager')}
        >
          <KaliIcon name="system-file-manager" size={24} />
        </button>
        <button title="Text Editor" aria-label="Text Editor" onClick={() => launch('text-editor')}>
          <KaliIcon name="accessories-text-editor" size={24} />
        </button>
        <button title="Web Browser" aria-label="Web Browser" onClick={() => launch('web-browser')}>
          <KaliIcon name="firefox" size={24} />
        </button>
        <button
          title="Tor Browser"
          aria-label="Tor Browser"
          onClick={() => {
            open('tor-browser');
            setPopover('');
          }}
        >
          <KaliIcon name="firefox" size={24} />
        </button>
        <button
          title="Terminal Emulator"
          aria-label="Terminal Emulator"
          onClick={() => launch('terminal')}
        >
          <KaliIcon name="utilities-terminal" size={24} />
        </button>
        <button
          className="panel-terminal-arrow"
          title="Terminais"
          aria-label="Escolher terminal"
          data-panel-menu="terminal"
          aria-expanded={popover === 'terminal'}
          onClick={() => toggle('terminal')}
        >
          <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
            <path d="m4 6 4 4 4-4" fill="none" stroke="currentColor" strokeWidth="1.5" />
          </svg>
        </button>
      </div>
      <span className="panel-separator" />
      <div className="workspaces">
        {[1, 2, 3, 4].map((n) => (
          <button
            key={n}
            aria-label={`Área de trabalho ${n}`}
            aria-pressed={workspace === n}
            className={workspace === n ? 'selected' : ''}
            onClick={() => useWindows.setState({ workspace: n })}
          >
            {n}
          </button>
        ))}
      </div>
      <span className="panel-separator" />
      <div className="panel-tray">
        <button
          className="panel-activity"
          aria-label="Monitor de atividade do jogo"
          title="Atividade do sistema virtual"
          onClick={() => open('processes')}
        >
          <svg viewBox="0 0 132 34" preserveAspectRatio="none" aria-hidden="true">
            <rect width="5" height="34" fill="#00e5e5" />
            <path
              d={activity.map((value, index) => `M${6 + index * 2} 34v-${value * 0.33}`).join('')}
              stroke="#00ccff"
              strokeWidth="1"
            />
          </svg>
        </button>
        <button
          aria-label="Rede"
          data-panel-menu="network"
          title={world.network.connected ? 'Conexão cabeada ativa' : 'Rede desconectada'}
          aria-expanded={popover === 'network'}
          onClick={() => toggle('network')}
        >
          <KaliIcon name="network-wired-symbolic" symbolic size={18} />
        </button>
        <button
          aria-label="Áudio"
          data-panel-menu="audio"
          title={`Volume: ${volume}%`}
          aria-expanded={popover === 'audio'}
          onClick={() => toggle('audio')}
        >
          <KaliIcon
            name={volume ? 'audio-volume-high-symbolic' : 'audio-volume-muted-symbolic'}
            symbolic
            size={18}
          />
        </button>
        <button
          aria-label={`Notificações, ${unread} não lidas`}
          title="Notificações"
          onClick={() => open('messages')}
        >
          <KaliIcon name="notification-symbolic" symbolic size={18} />
          {unread > 0 && <i className="notification-dot" />}
        </button>
        <button
          aria-label="Energia"
          data-panel-menu="power"
          title="Computador virtual · carga completa"
          aria-expanded={popover === 'power'}
          onClick={() => toggle('power')}
        >
          <KaliIcon name="battery-full-charged-symbolic" symbolic size={16} />
        </button>
        <button
          className="panel-clock"
          aria-label="Calendário"
          data-panel-menu="calendar"
          aria-expanded={popover === 'calendar'}
          title={time.toLocaleDateString('pt-BR', { dateStyle: 'full' })}
          onClick={() => toggle('calendar')}
        >
          <time>{time.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })}</time>
        </button>
        <span className="panel-separator" />
        <button aria-label="Bloquear sessão" title="Bloquear sessão" onClick={onLock}>
          <KaliIcon name="system-lock-screen-symbolic" symbolic size={18} />
        </button>
        <button
          aria-label="Sair da sessão"
          title="Encerrar sessão e sair"
          disabled={busy}
          onClick={onLogout}
        >
          <KaliIcon name="system-log-out-symbolic" symbolic size={18} />
        </button>
      </div>
      {popover && (
        <div ref={popup} className={`panel-popover panel-popover-${popover}`}>
          {popover === 'terminal' && (
            <>
              {['terminal', 'root-terminal'].map((id) => (
                <button key={id} onClick={() => launch(id)}>
                  <KaliIcon name={softwareById.get(id)?.icon ?? 'utilities-terminal'} size={24} />
                  {softwareById.get(id)?.name}
                </button>
              ))}
            </>
          )}
          {popover === 'network' && (
            <>
              <h3>Rede cabeada</h3>
              <p>{world.network.connected ? 'Conectada' : 'Desconectada'} · eth0</p>
              <p className="muted">Gateway virtual: {world.network.gateway}</p>
              <button
                onClick={() => {
                  open('settings');
                  setPopover('');
                }}
              >
                Configurações de rede
              </button>
            </>
          )}
          {popover === 'audio' && (
            <>
              <label>
                Volume da sessão · {volume}%
                <input
                  aria-label="Volume da sessão"
                  type="range"
                  min="0"
                  max="100"
                  value={volume}
                  onChange={(event) => {
                    const next = Number(event.target.value);
                    setVolume(next);
                    audioManager.setVolumes({ ...audioManager.getVolumes(), master: next });
                  }}
                  onPointerUp={() =>
                    act('setting_update', { key: 'volume', value: String(volume) })
                  }
                  onKeyUp={() => act('setting_update', { key: 'volume', value: String(volume) })}
                />
              </label>
              <button
                onClick={() => {
                  const next = volume ? 0 : 70;
                  setVolume(next);
                  audioManager.setVolumes({ ...audioManager.getVolumes(), master: next });
                  act('setting_update', { key: 'volume', value: String(next) });
                }}
              >
                {volume ? 'Silenciar' : 'Ativar áudio'}
              </button>
            </>
          )}
          {popover === 'power' && (
            <>
              <h3>Energia</h3>
              <p>Computador virtual · 100%</p>
              <button
                onClick={() => {
                  open('settings');
                  setPopover('');
                }}
              >
                Configurações da sessão
              </button>
            </>
          )}
          {popover === 'calendar' && <Calendar date={time} />}
        </div>
      )}
    </header>
  );
}

function Calendar({ date }: { date: Date }) {
  const offset = new Date(date.getFullYear(), date.getMonth(), 1).getDay();
  const days = new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
  return (
    <>
      <h3>{date.toLocaleDateString('pt-BR', { month: 'long', year: 'numeric' })}</h3>
      <div className="panel-calendar">
        {['D', 'S', 'T', 'Q', 'Q', 'S', 'S'].map((day, index) => (
          <b key={`day-${index}`}>{day}</b>
        ))}
        {Array.from({ length: offset + days }, (_, index) => (
          <span key={index} className={index - offset + 1 === date.getDate() ? 'today' : ''}>
            {index < offset ? '' : index - offset + 1}
          </span>
        ))}
      </div>
    </>
  );
}
