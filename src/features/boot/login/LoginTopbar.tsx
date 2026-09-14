import { useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';

type Panel = 'session' | 'language' | 'accessibility' | 'clock' | 'power';
const zones: Record<string, string> = {
  'America/Sao_Paulo': 'Brasília',
  'America/Manaus': 'Manaus',
  'America/Belem': 'Belém',
};
const keyboards: Record<string, string> = {
  'br-abnt2': 'Português brasileiro (ABNT2)',
  'br-abnt': 'Português brasileiro (ABNT)',
  'us-intl': 'Inglês americano (internacional)',
};

export function LoginTopbar({
  hostname,
  settings,
  language,
  largeText,
  highContrast,
  showPassword,
  onLargeText,
  onHighContrast,
  onShowPassword,
  onLogin,
  onSwitchUser,
  onCancel,
  onRestart,
  onShutdown,
}: {
  hostname: string;
  settings: Record<string, string>;
  language: string;
  largeText: boolean;
  highContrast: boolean;
  showPassword: boolean;
  onLargeText: () => void;
  onHighContrast: () => void;
  onShowPassword: () => void;
  onLogin: () => void;
  onSwitchUser: () => void;
  onCancel: () => void;
  onRestart: () => void;
  onShutdown: () => void;
}) {
  const [now, setNow] = useState(() => new Date());
  const [panel, setPanel] = useState<Panel | null>(null);
  const [powerAction, setPowerAction] = useState<'restart' | 'shutdown' | null>(null);
  const bar = useRef<HTMLElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement | null>(null);
  // Read the saved locale, with the installed language as fallback for legacy saves.
  const configuredLocale = settings.language || language;
  const locale = configuredLocale === 'pt-BR' ? configuredLocale : 'pt-BR';
  const timeZone = Object.hasOwn(zones, settings.timezone)
    ? settings.timezone
    : 'America/Sao_Paulo';
  const parts = new Intl.DateTimeFormat(locale, {
    timeZone,
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).formatToParts(now);
  const part = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((value) => value.type === type)?.value ?? '';
  const dateLabel = `${part('day')} ${part('month').replace('.', '')}, ${part('hour')}:${part('minute')}`;
  const fullDate = new Intl.DateTimeFormat(locale, { timeZone, dateStyle: 'full' }).format(now);

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  useEffect(() => {
    if (!panel) {
      return;
    }
    popup.current?.querySelector<HTMLButtonElement>('button')?.focus();
    const outside = (event: PointerEvent) => {
      if (event.target instanceof Node && !bar.current?.contains(event.target)) {
        setPanel(null);
      }
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') {
        return;
      }
      event.preventDefault();
      setPanel(null);
      trigger.current?.focus();
    };
    document.addEventListener('pointerdown', outside);
    document.addEventListener('keydown', escape);
    return () => {
      document.removeEventListener('pointerdown', outside);
      document.removeEventListener('keydown', escape);
    };
  }, [panel]);

  const close = () => {
    setPanel(null);
    trigger.current?.focus();
  };
  const act = (action: () => void) => {
    setPanel(null);
    action();
  };
  const control = (id: Panel, label: string, content: ReactNode) => (
    <button
      type="button"
      className={`login-tray-button login-tray-${id}`}
      aria-label={label}
      title={label}
      aria-haspopup="dialog"
      aria-expanded={panel === id}
      aria-controls={panel === id ? 'login-panel' : undefined}
      onClick={(event) => {
        trigger.current = event.currentTarget;
        setPowerAction(null);
        setPanel(panel === id ? null : id);
      }}
    >
      {content}
    </button>
  );
  const titles: Record<Panel, string> = {
    session: 'Opções de sessão',
    language: 'Idioma e região',
    accessibility: 'Acessibilidade',
    clock: 'Data e hora',
    power: 'Energia',
  };

  return (
    <header className="login-topbar" ref={bar}>
      <span className="login-topbar-host" title={hostname}>
        {hostname}
      </span>
      <div className="login-topbar-tray" aria-label="Indicadores do sistema">
        {control(
          'session',
          'Opções de sessão',
          <span className="login-session-icon" aria-hidden="true" />,
        )}
        {control(
          'language',
          'Idioma: Português (Brasil)',
          <span lang={locale}>{locale.split('-')[0]}</span>,
        )}
        {control(
          'accessibility',
          'Acessibilidade',
          <span className="login-accessibility-icon" aria-hidden="true" />,
        )}
        {control(
          'clock',
          `Data e hora: ${fullDate}, ${dateLabel.slice(-5)}. Fuso de ${zones[timeZone]}`,
          <time dateTime={now.toISOString()}>{dateLabel}</time>,
        )}
        {control('power', 'Energia', <span className="login-power-icon" aria-hidden="true" />)}
      </div>
      {panel && (
        <div
          className="login-panel"
          id="login-panel"
          ref={popup}
          role="dialog"
          aria-label={titles[panel]}
        >
          <div className="login-panel-heading">
            <strong>{titles[panel]}</strong>
            <button type="button" aria-label="Fechar painel" onClick={close}>
              ×
            </button>
          </div>
          {panel === 'session' && (
            <>
              <button type="button" className="login-panel-action" onClick={() => act(onLogin)}>
                Entrar no Kali Linux
              </button>
              <button
                type="button"
                className="login-panel-action"
                onClick={() => act(onSwitchUser)}
              >
                Trocar usuário
              </button>
              <button type="button" className="login-panel-action" onClick={() => act(onCancel)}>
                Voltar ao menu do jogo
              </button>
            </>
          )}
          {panel === 'language' && (
            <>
              <p>Português (Brasil) · {locale}</p>
              <dl className="login-regional-details">
                <dt>Localidade</dt>
                <dd>{settings.location || 'Brasil'}</dd>
                <dt>Teclado</dt>
                <dd>{keyboards[settings.keyboard] || keyboards['br-abnt2']}</dd>
                <dt>Fuso horário</dt>
                <dd>{zones[timeZone]}</dd>
              </dl>
              <p className="login-panel-note">
                Configuração escolhida na instalação. Português (Brasil) é o idioma disponível nesta
                versão.
              </p>
            </>
          )}
          {panel === 'accessibility' && (
            <>
              <label className="login-panel-option">
                <input type="checkbox" checked={largeText} onChange={onLargeText} />
                Texto maior
              </label>
              <label className="login-panel-option">
                <input type="checkbox" checked={highContrast} onChange={onHighContrast} />
                Alto contraste
              </label>
              <label className="login-panel-option">
                <input type="checkbox" checked={showPassword} onChange={onShowPassword} />
                Mostrar senha
              </label>
            </>
          )}
          {panel === 'clock' && (
            <>
              <p className="login-clock-time">{dateLabel.slice(-5)}</p>
              <p>{fullDate}</p>
              <p className="login-panel-note">
                Fuso de {zones[timeZone]} · {timeZone}
              </p>
              <p className="login-panel-note">Data e hora atualizadas automaticamente.</p>
            </>
          )}
          {panel === 'power' &&
            (powerAction ? (
              <>
                <p>
                  {powerAction === 'restart'
                    ? 'Reiniciar o sistema e voltar ao login?'
                    : 'Desligar o sistema e sair do jogo?'}
                </p>
                <div className="login-panel-confirm">
                  <button type="button" onClick={() => setPowerAction(null)}>
                    Cancelar
                  </button>
                  <button
                    type="button"
                    onClick={() => act(powerAction === 'restart' ? onRestart : onShutdown)}
                  >
                    {powerAction === 'restart' ? 'Confirmar reinício' : 'Confirmar desligamento'}
                  </button>
                </div>
              </>
            ) : (
              <>
                <button
                  type="button"
                  className="login-panel-action"
                  onClick={() => setPowerAction('restart')}
                >
                  Reiniciar sistema…
                </button>
                <button
                  type="button"
                  className="login-panel-action"
                  onClick={() => setPowerAction('shutdown')}
                >
                  Desligar…
                </button>
                <button type="button" className="login-panel-action" onClick={() => act(onCancel)}>
                  Voltar ao menu do jogo
                </button>
              </>
            ))}
        </div>
      )}
    </header>
  );
}
