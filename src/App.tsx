import { useEffect, useState } from 'react';
import type { CSSProperties } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Desktop } from './features/desktop/Desktop';
import { BootFlow } from './features/boot/BootFlow';
import { FadeTransition } from './features/boot/FadeTransition';
import { NarrativeIntro } from './features/boot/NarrativeIntro';
import { preloadBranding, openingSources } from './features/boot/boot-assets';
import { desktopRuntime } from './lib/api';
import { useGame } from './lib/game-store';
import { endSession, startSession } from './lib/session';
import { handleFullscreenKey } from './lib/fullscreen';
import { loadAppSettings, useAppSettings } from './lib/app-settings';
import { audioManager } from './lib/audio-manager';
import { LoginScreen } from './features/boot/login/LoginScreen';

type Screen = 'boot' | 'menu' | 'login' | 'narrative' | 'desktop';
let preload: Promise<void> | undefined;
function preloadAssets(): Promise<void> {
  preload ??= Promise.race([
    Promise.all([audioManager.preload(), preloadBranding()]).then(() => undefined),
    new Promise<void>((resolve) => window.setTimeout(resolve, 1500)),
  ]);
  return preload;
}
export function App() {
  const [screen, setScreen] = useState<Screen>('boot');
  const [assetsReady, setAssetsReady] = useState(false);
  const [startAttempt, setStartAttempt] = useState(0);
  const [systemReduced, setSystemReduced] = useState(
    () => window.matchMedia('(prefers-reduced-motion: reduce)').matches,
  );
  const { settings, ready } = useAppSettings();
  const { error, clearError, sessionPending } = useGame();
  const enterDesktop = () => {
    void startSession(() => setScreen('desktop')).catch(() =>
      setStartAttempt((attempt) => attempt + 1),
    );
  };
  const returnToMenu = () => {
    void endSession(() => setScreen('menu')).catch(() => undefined);
  };
  const play = (newGame: boolean, needsLogin?: boolean) => {
    if (newGame) {
      // No session has been entered yet; preserve the new First Boot attempt.
      setScreen('narrative');
    } else if (needsLogin) {
      void endSession(() => setScreen('login')).catch(() => undefined);
    } else {
      enterDesktop();
    }
  };
  useEffect(() => {
    let active = true;
    void loadAppSettings();
    void preloadAssets().then(() => {
      if (active) {
        setAssetsReady(true);
      }
    });
    const media = window.matchMedia('(prefers-reduced-motion: reduce)');
    const change = () => setSystemReduced(media.matches);
    media.addEventListener('change', change);
    window.addEventListener('keydown', handleFullscreenKey, true);
    const unlock = () => audioManager.unlock();
    const visibility = () => {
      if (document.hidden) {
        audioManager.pause();
      } else {
        audioManager.resume();
      }
    };
    window.addEventListener('pointerdown', unlock, true);
    window.addEventListener('keydown', unlock, true);
    document.addEventListener('visibilitychange', visibility);
    // Warm the preferred local cinematic while the studio sequence is running.
    const video = document.createElement('video');
    if (openingSources[0]) {
      video.preload = 'auto';
      video.src = openingSources[0];
      video.load();
    }
    return () => {
      active = false;
      media.removeEventListener('change', change);
      window.removeEventListener('keydown', handleFullscreenKey, true);
      window.removeEventListener('pointerdown', unlock, true);
      window.removeEventListener('keydown', unlock, true);
      document.removeEventListener('visibilitychange', visibility);
      video.removeAttribute('src');
      video.load();
      audioManager.stopAll();
    };
  }, []);
  useEffect(() => {
    if (!desktopRuntime) {
      return;
    }
    const unlisten = listen<string>('save-error', (event) =>
      useGame.setState({ error: event.payload }),
    );
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, []);
  if (!ready || !assetsReady) {
    return <div className="boot-loading" aria-label="Carregando CYBER WAR" />;
  }
  return (
    <div
      className={`game-root ${systemReduced || settings.reducedMotion ? 'reduce-motion' : ''} ${settings.highContrast ? 'high-contrast' : ''}`}
      style={{ '--menu-scale': settings.uiScale / 100 } as CSSProperties}
      inert={sessionPending}
      aria-busy={sessionPending}
    >
      <FadeTransition
        stage={screen}
        render={(shown) =>
          shown === 'desktop' ? (
            <Desktop onMenu={() => setScreen('menu')} />
          ) : shown === 'login' ? (
            <LoginScreen onCancel={returnToMenu} onSuccess={enterDesktop} />
          ) : shown === 'narrative' ? (
            <NarrativeIntro key={startAttempt} onFinish={enterDesktop} />
          ) : (
            <BootFlow settings={settings} returnToMenu={shown === 'menu'} onPlay={play} />
          )
        }
      />
      {error && (screen === 'login' || screen === 'narrative') && (
        <div className="error-toast" role="alert">
          <span>{error}</span>
          <button onClick={clearError} aria-label="Fechar erro">
            ×
          </button>
        </div>
      )}
    </div>
  );
}
