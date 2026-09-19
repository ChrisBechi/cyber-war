import { useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { desktopRuntime, emptySchema, request, slotSchema, worldSchema } from '../../lib/api';
import type { SaveSlot } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { audioManager } from '../../lib/audio-manager';
import { saveAppSettings, useAppSettings } from '../../lib/app-settings';
import { FadeTransition } from '../boot/FadeTransition';
import { SettingsScreen } from './SettingsScreen';
import { HowToPlay } from './HowToPlay';
import { ConfirmDialog } from './ConfirmDialog';
import { NewSystemSetup } from './NewSystemSetup';
import type { SystemSetupValues } from './NewSystemSetup';

type Page = 'home' | 'new' | 'load' | 'settings' | 'help';
export function MainMenu({ onPlay }: { onPlay: (newGame: boolean, needsLogin?: boolean) => void }) {
  const [slots, setSlots] = useState<SaveSlot[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [page, setPage] = useState<Page>('home');
  const [selected, setSelected] = useState(1);
  const [nickname, setNickname] = useState('kali');
  const [hostname, setHostname] = useState('lifeos');
  const [dialog, setDialog] = useState<'exit' | 'overwrite' | 'delete' | null>(null);
  const [pending, setPending] = useState(false);
  const [setup, setSetup] = useState(false);
  const [setupOverwrite, setSetupOverwrite] = useState(false);
  const operation = useRef(false);
  const { error, busy } = useGame();
  const settingsError = useAppSettings((state) => state.error);
  const appSettings = useAppSettings((state) => state.settings);
  useEffect(() => {
    const music = audioManager.music('menu-ambience');
    let active = true;
    if (desktopRuntime) {
      void request('list_save_slots', {}, z.array(slotSchema).length(5))
        .then((value) => {
          if (active) {
            setSlots(value.sort((a, b) => a.slotIndex - b.slotIndex));
            setLoaded(true);
          }
        })
        .catch((reason: unknown) => {
          if (active) {
            useGame.setState({ error: String(reason) });
          }
        });
    }
    return () => {
      active = false;
      audioManager.stop(music, 500);
    };
  }, []);
  const navigate = (next: Page) => {
    audioManager.play('menu-select');
    setPage(next);
    useGame.getState().clearError();
  };
  const startLoaded = async (manual = false) => {
    if (operation.current) {
      return;
    }
    operation.current = true;
    setPending(true);
    try {
      useGame.setState({ slot: selected });
      await perform('load_slot', { slotIndex: selected, manual }, worldSchema);
      audioManager.play('menu-select');
      onPlay(false, true);
    } catch {
      /* The store presents native validation and save errors. */
    } finally {
      operation.current = false;
      setPending(false);
      setDialog(null);
    }
  };
  const startNew = async (values: SystemSetupValues, overwrite: boolean) => {
    if (operation.current) {
      return;
    }
    operation.current = true;
    setPending(true);
    try {
      useGame.setState({ slot: selected });
      await perform(
        'new_game',
        { slotIndex: selected, nickname: values.username, hostname: values.hostname, overwrite },
        worldSchema,
      );
      const settings: Array<[string, string]> = [
        ['language', values.language],
        ['location', values.location],
        ['keyboard', values.keyboard],
        ['network', values.network],
        ['networkSsid', values.networkSsid],
        ['networkEnabled', String(values.networkEnabled)],
        ['domain', values.domain],
        ['displayMode', values.displayMode],
        ['displayResolution', values.resolution],
        ['disk', values.disk],
        ['partition', values.partition],
        ['partitionScheme', values.partitionScheme],
        ['storage', values.storage],
        ['partitions', JSON.stringify(values.partitions)],
        ['writeChanges', String(values.writeChanges)],
        ['softwareProfile', values.softwareProfile],
        ['desktopEnvironment', values.desktopEnvironment],
        ['softwareTools', values.softwareTools],
        ['fullName', values.fullName],
        ['timezone', values.timezone],
        ['loginUsername', values.username],
        ['loginPassword', values.password],
      ];
      for (const [key, value] of settings) {
        await perform('setting_update', { key, value }, emptySchema);
      }
      await perform('system_setup_complete', {}, emptySchema);
      await saveAppSettings({
        ...appSettings,
        fullscreen: values.displayMode === 'fullscreen',
        resolution: values.resolution,
      });
      // Promote the completed installer configuration to the manual snapshot.
      // Otherwise “recover last manual save” would reopen the pre-install state.
      await perform('save_slot', {}, emptySchema);
      audioManager.play('menu-select');
      setSetup(false);
      onPlay(true);
    } catch {
      /* The store presents native validation and save errors. */
    } finally {
      operation.current = false;
      setPending(false);
      setSetupOverwrite(false);
      setDialog(null);
    }
  };
  const deleteSave = async () => {
    if (operation.current) {
      return;
    }
    operation.current = true;
    setPending(true);
    useGame.getState().clearError();
    try {
      const remaining = await request(
        'delete_save_slot',
        { slotIndex: selected, confirmed: true },
        z.array(slotSchema).length(5),
      );
      setSlots(remaining.sort((a, b) => a.slotIndex - b.slotIndex));
      if (useGame.getState().slot === selected) {
        useGame.setState({ world: null, missions: [] });
      }
      setDialog(null);
    } catch (reason) {
      useGame.setState({ error: String(reason) });
    } finally {
      operation.current = false;
      setPending(false);
    }
  };
  const exit = async () => {
    if (operation.current) {
      return;
    }
    operation.current = true;
    setPending(true);
    try {
      await request('quit_game', {}, emptySchema);
    } catch (reason) {
      useGame.setState({ error: String(reason) });
      setDialog(null);
    } finally {
      operation.current = false;
      setPending(false);
    }
  };
  const occupied = slots.find((slot) => slot.slotIndex === selected)?.occupied ?? false;
  const locked = busy || pending;
  if (setup) {
    return (
      <main className="front-menu setup-menu" data-testid="system-setup">
        <NewSystemSetup
          initialHostname={hostname}
          initialUsername={nickname}
          onCancel={() => {
            if (!locked) {
              setSetup(false);
              setSetupOverwrite(false);
            }
          }}
          onComplete={(values) => {
            setNickname(values.username);
            setHostname(values.hostname);
            void startNew(values, setupOverwrite);
          }}
        />
        {(error || settingsError) && (
          <p role="alert" className="front-error">
            {error || settingsError}
          </p>
        )}
      </main>
    );
  }
  const home = (
    <div className="front-home">
      <div className="front-home-content">
        <p className="front-kicker">STUDIO BECHI GAMES APRESENTA</p>
        <h1 className="front-wordmark">
          CYBER WAR
          <span className="game-title-cursor" aria-hidden="true">
            _
          </span>
        </h1>
        <p className="front-tagline">Todo sistema deixa um vestígio.</p>
        <nav
          className="front-actions"
          aria-label="Menu principal"
          onKeyDown={(event) => {
            if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
              return;
            }
            event.preventDefault();
            const buttons = Array.from(
              event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'),
            );
            const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
            const index =
              event.key === 'Home'
                ? 0
                : event.key === 'End'
                  ? buttons.length - 1
                  : (current + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) %
                    buttons.length;
            buttons[index]?.focus();
          }}
        >
          {(
            [
              ['INICIAR HISTÓRIA', () => navigate('new'), false],
              [
                'CONTINUAR',
                () => {
                  setSelected(slots.find((slot) => slot.occupied)?.slotIndex ?? 1);
                  navigate('load');
                },
                !slots.some((slot) => slot.occupied),
              ],
              ['CONFIGURAÇÕES', () => navigate('settings'), false],
              ['COMO JOGAR', () => navigate('help'), false],
              [
                'SAIR',
                () => {
                  audioManager.play('menu-select');
                  setDialog('exit');
                },
                false,
              ],
            ] as const
          ).map(([label, action, disabled]) => (
            <button
              key={label}
              disabled={disabled}
              onClick={action}
              onPointerEnter={() => {
                if (!disabled) {
                  audioManager.play('menu-hover');
                }
              }}
              onFocus={() => audioManager.play('menu-hover')}
            >
              <span aria-hidden="true">&gt;</span>
              {label}
            </button>
          ))}
        </nav>
        {!desktopRuntime && (
          <p className="front-preview">
            Prévia visual · Abra o aplicativo desktop para acessar sua campanha.
          </p>
        )}
      </div>
      <div className="front-schematic" aria-hidden="true">
        <div className="front-orbit" />
        <div className="front-orbit inner" />
        <span className="front-crosshair">+</span>
        <div className="front-coordinate">
          CONEXÃO LOCAL
          <br />
          127.0.0.1 / ESTADO: AGUARDANDO
        </div>
      </div>
    </div>
  );
  return (
    <main className="front-menu" data-testid="main-menu">
      <div className="front-scanlines" aria-hidden="true" />
      <FadeTransition
        stage={page}
        render={(shown) =>
          shown === 'home' ? (
            home
          ) : (
            <section className="front-page">
              <header className="front-page-header">
                <button
                  disabled={locked}
                  onClick={() => navigate('home')}
                  aria-label="Voltar ao menu"
                >
                  ← VOLTAR
                </button>
                <span>
                  CYBER WAR <span className="front-slash">/</span>{' '}
                  {shown === 'settings'
                    ? 'CONFIGURAÇÕES'
                    : shown === 'help'
                      ? 'COMO JOGAR'
                      : 'CAMPANHA LOCAL'}
                </span>
              </header>
              {shown === 'settings' ? (
                <SettingsScreen />
              ) : shown === 'help' ? (
                <HowToPlay />
              ) : (
                <div className="front-campaign">
                  <div>
                    <p className="front-kicker">
                      {shown === 'new' ? 'UMA NOVA CONEXÃO' : 'SEUS VESTÍGIOS CONTINUAM AQUI'}
                    </p>
                    <h1>{shown === 'new' ? 'Iniciar história' : 'Continuar'}</h1>
                    <p>Escolha um dos cinco espaços da sua campanha.</p>
                  </div>
                  {!loaded ? (
                    <p role="status">
                      {desktopRuntime
                        ? 'Carregando campanhas…'
                        : 'Os saves ficam disponíveis no aplicativo desktop.'}
                    </p>
                  ) : (
                    <div className="front-slot-list" role="group" aria-label="Slots da campanha">
                      {slots.map((slot) => (
                        <div className="front-slot-row" key={slot.slotIndex}>
                          <button
                            className={`front-slot ${selected === slot.slotIndex ? 'is-selected' : ''}`}
                            aria-pressed={selected === slot.slotIndex}
                            disabled={locked || (shown === 'load' && !slot.occupied)}
                            onClick={() => {
                              setSelected(slot.slotIndex);
                              audioManager.play('menu-hover');
                            }}
                          >
                            <span className="front-slot-number">0{slot.slotIndex}</span>
                            <span className="front-slot-description">
                              <strong>{slot.occupied ? slot.label : 'Slot vazio'}</strong>
                              <small>
                                {slot.occupied
                                  ? `${Math.floor(slot.playtimeSeconds / 3600)} h ${Math.floor(slot.playtimeSeconds / 60) % 60} min · Sessão ${slot.session ?? '—'} · ${slot.currentMission ?? 'Explorando'}`
                                  : 'Uma história por começar'}
                              </small>
                            </span>
                            <span className="front-slot-date">
                              {slot.updatedAt
                                ? new Date(slot.updatedAt).toLocaleString('pt-BR', {
                                    dateStyle: 'short',
                                    timeStyle: 'short',
                                  })
                                : '—'}
                            </span>
                          </button>
                          {slot.occupied && (
                            <button
                              className="front-slot-delete"
                              disabled={locked}
                              aria-label={`Excluir save do slot 0${slot.slotIndex}: ${slot.label}`}
                              title="Excluir save"
                              onClick={() => {
                                setSelected(slot.slotIndex);
                                useGame.getState().clearError();
                                setDialog('delete');
                              }}
                            >
                              <span aria-hidden="true" />
                            </button>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                  <div className="front-page-actions">
                    <span>
                      {shown === 'new' && occupied
                        ? 'Este slot já contém uma campanha.'
                        : 'O progresso é salvo neste computador.'}
                    </span>
                    <button
                      className="front-primary"
                      disabled={locked || !loaded || (shown === 'load' && !occupied)}
                      onClick={() => {
                        if (shown === 'new' && occupied) {
                          setDialog('overwrite');
                        } else if (shown === 'new') {
                          setSetupOverwrite(false);
                          setSetup(true);
                        } else {
                          void startLoaded();
                        }
                      }}
                    >
                      {locked
                        ? 'CARREGANDO…'
                        : shown === 'new'
                          ? 'INICIAR HISTÓRIA →'
                          : 'CONTINUAR →'}
                    </button>
                  </div>
                  {shown === 'load' && occupied && (
                    <button
                      className="front-recovery"
                      disabled={locked}
                      onClick={() => {
                        void startLoaded(true);
                      }}
                    >
                      Recuperar último save manual
                    </button>
                  )}
                </div>
              )}
            </section>
          )
        }
      />
      {(error || settingsError) && dialog !== 'delete' && (
        <p role="alert" className="front-error">
          {error || settingsError}
        </p>
      )}
      <footer className="front-footer">
        <span>STUDIO BECHI GAMES</span>
        <span>CYBER WAR / v0.4.2</span>
      </footer>
      {dialog && (
        <ConfirmDialog
          title={
            dialog === 'exit'
              ? 'SAIR DO CYBER WAR?'
              : dialog === 'delete'
                ? `EXCLUIR O SAVE DO SLOT 0${selected}?`
                : `SUBSTITUIR A CAMPANHA DO SLOT 0${selected}?`
          }
          description={
            dialog === 'exit'
              ? 'Você voltará para a área de trabalho.'
              : dialog === 'delete'
                ? `O save de “${slots.find((slot) => slot.slotIndex === selected)?.label}”, incluindo progresso, autosave e checkpoints, será excluído permanentemente. Esta ação não pode ser desfeita.`
                : 'O save atual deste slot, incluindo seu progresso e checkpoints, será sobrescrito. Esta ação não pode ser desfeita.'
          }
          confirmLabel={
            dialog === 'exit'
              ? 'SAIR'
              : dialog === 'delete'
                ? 'EXCLUIR SAVE'
                : 'SUBSTITUIR E INICIAR'
          }
          busy={pending}
          error={dialog === 'delete' ? error : undefined}
          onCancel={() => setDialog(null)}
          onConfirm={() => {
            audioManager.play('menu-select');
            if (dialog === 'exit') {
              void exit();
            } else if (dialog === 'delete') {
              void deleteSave();
            } else {
              setDialog(null);
              setSetupOverwrite(true);
              setSetup(true);
            }
          }}
        />
      )}
    </main>
  );
}
