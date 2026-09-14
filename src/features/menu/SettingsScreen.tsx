import { useEffect, useState } from 'react';
import { desktopRuntime } from '../../lib/api';
import { appSettingsSchema, saveAppSettings, useAppSettings } from '../../lib/app-settings';
import type { AppSettings } from '../../lib/app-settings';
import { audioManager } from '../../lib/audio-manager';

const tabs = ['Geral', 'Áudio', 'Vídeo', 'Controles', 'Acessibilidade'] as const;
export function SettingsScreen() {
  const { settings, saving } = useAppSettings();
  const [draft, setDraft] = useState(settings);
  const [tab, setTab] = useState<(typeof tabs)[number]>('Geral');
  const [saved, setSaved] = useState(false);
  useEffect(() => () => audioManager.setVolumes(useAppSettings.getState().settings), []);
  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    const next = { ...draft, [key]: value };
    setDraft(next);
    setSaved(false);
    audioManager.setVolumes(next);
  };
  const apply = async () => {
    try {
      await saveAppSettings(draft);
      setSaved(true);
    } catch {
      /* Menu displays the persistence error. */
    }
  };
  const check = (
    key: 'skipStudioIntro' | 'skipTrailer' | 'reducedMotion' | 'highContrast',
    title: string,
    description: string,
  ) => (
    <label className="front-setting-row">
      <span>
        <strong>{title}</strong>
        <small>{description}</small>
      </span>
      <input
        type="checkbox"
        checked={draft[key]}
        onChange={(event) => update(key, event.target.checked)}
      />
    </label>
  );
  return (
    <div className="front-settings">
      <p className="front-kicker">PREFERÊNCIAS DESTE COMPUTADOR</p>
      <h1>Configurações</h1>
      <div
        className="front-tabs"
        role="tablist"
        aria-label="Configurações"
        onKeyDown={(event) => {
          if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
            return;
          }
          event.preventDefault();
          const index =
            event.key === 'Home'
              ? 0
              : event.key === 'End'
                ? tabs.length - 1
                : (tabs.indexOf(tab) + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) %
                  tabs.length;
          setTab(tabs[index]);
          event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="tab"]')[index]?.focus();
        }}
      >
        {tabs.map((title) => (
          <button
            key={title}
            role="tab"
            id={`settings-tab-${title}`}
            aria-selected={tab === title}
            aria-controls="settings-panel"
            tabIndex={tab === title ? 0 : -1}
            onClick={() => {
              setTab(title);
              audioManager.play('menu-hover');
            }}
          >
            {title}
          </button>
        ))}
      </div>
      <fieldset
        disabled={saving}
        id="settings-panel"
        className="front-settings-panel"
        role="tabpanel"
        aria-labelledby={`settings-tab-${tab}`}
      >
        {tab === 'Geral' && (
          <>
            {check(
              'skipStudioIntro',
              'Pular intro do estúdio ao abrir',
              'A tela “Clique para iniciar” continua aparecendo.',
            )}
            {check(
              'skipTrailer',
              'Pular cinemática ao abrir',
              'Esta preferência vale para as próximas aberturas.',
            )}
            <label className="front-setting-row">
              <strong>Idioma</strong>
              <select
                value={draft.language}
                onChange={(event) =>
                  update('language', appSettingsSchema.shape.language.parse(event.target.value))
                }
              >
                <option value="pt-BR">Português (Brasil)</option>
              </select>
            </label>
          </>
        )}
        {tab === 'Áudio' && (
          <>
            {(
              [
                ['master', 'Volume geral'],
                ['music', 'Música'],
                ['sfx', 'Efeitos sonoros'],
                ['voice', 'Voz'],
              ] as const
            ).map(([key, title]) => (
              <label key={key} className="front-setting-row front-volume">
                <strong>{title}</strong>
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={draft[key]}
                  onChange={(event) => update(key, Number(event.target.value))}
                />
                <output>{draft[key]}%</output>
              </label>
            ))}
            <button onClick={() => audioManager.play('menu-select')}>Testar efeitos sonoros</button>
            <p className="front-setting-note">
              A prévia de volume é imediata. Aplique para manter suas alterações.
            </p>
          </>
        )}
        {tab === 'Vídeo' && (
          <>
            <label className="front-setting-row">
              <strong>Modo de exibição</strong>
              <select
                value={draft.fullscreen ? 'fullscreen' : 'windowed'}
                onChange={(event) => update('fullscreen', event.target.value === 'fullscreen')}
              >
                <option value="fullscreen">Tela cheia</option>
                <option value="windowed">Janela</option>
              </select>
            </label>
            <label className="front-setting-row">
              <span>
                <strong>Resolução da janela</strong>
                <small>Em tela cheia, o jogo usa toda a tela.</small>
              </span>
              <select
                disabled={draft.fullscreen}
                value={draft.resolution}
                onChange={(event) =>
                  update('resolution', appSettingsSchema.shape.resolution.parse(event.target.value))
                }
              >
                {appSettingsSchema.shape.resolution.options.map((size) => (
                  <option key={size}>{size}</option>
                ))}
              </select>
            </label>
            <div className="front-setting-row">
              <span>
                <strong>VSync</strong>
                <small>Sincronização gerenciada pelo renderizador do sistema.</small>
              </span>
              <span>Automático (WebView2)</span>
            </div>
            <label className="front-setting-row">
              <strong>Escala dos menus</strong>
              <select
                value={draft.uiScale}
                onChange={(event) =>
                  update(
                    'uiScale',
                    appSettingsSchema.shape.uiScale.parse(Number(event.target.value)),
                  )
                }
              >
                {[90, 100, 110, 125].map((scale) => (
                  <option key={scale} value={scale}>
                    {scale}%
                  </option>
                ))}
              </select>
            </label>
          </>
        )}
        {tab === 'Controles' && (
          <dl className="front-controls">
            <dt>Alternar tela cheia</dt>
            <dd>F11</dd>
            <dt>Pular intro / cinemática</dt>
            <dd>Esc · Enter · Espaço · Clique</dd>
            <dt>Navegar no menu</dt>
            <dd>↑ ↓ · Tab · Enter</dd>
            <dt>Completar no terminal</dt>
            <dd>Tab</dd>
            <dt>Histórico do terminal</dt>
            <dd>↑ ↓</dd>
            <dt>Mover o cursor</dt>
            <dd>← → · Home · End</dd>
            <dt>Limpar linha / terminal</dt>
            <dd>Ctrl+C / Ctrl+L</dd>
          </dl>
        )}
        {tab === 'Acessibilidade' && (
          <>
            {check(
              'reducedMotion',
              'Reduzir movimento',
              'Desativa glitches, digitação rápida e movimentos de fundo.',
            )}
            {check(
              'highContrast',
              'Alto contraste',
              'Aumenta o contraste dos textos e controles dos menus.',
            )}
            <p className="front-setting-note">
              A preferência de movimento reduzido do sistema também é respeitada.
            </p>
          </>
        )}
      </fieldset>
      <div className="front-page-actions">
        <span role="status">
          {saved
            ? 'Preferências salvas.'
            : desktopRuntime
              ? 'Suas preferências são independentes da campanha.'
              : 'Abra o aplicativo desktop para salvar preferências.'}
        </span>
        <div>
          <button
            disabled={saving}
            onClick={() => {
              setDraft(settings);
              audioManager.setVolumes(settings);
              setSaved(false);
            }}
          >
            REVERTER
          </button>{' '}
          <button
            className="front-primary"
            disabled={saving || !desktopRuntime}
            onClick={() => {
              void apply();
            }}
          >
            {saving ? 'SALVANDO…' : 'APLICAR'}
          </button>
        </div>
      </div>
    </div>
  );
}
