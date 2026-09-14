import { useEffect, useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { desktopRuntime, emptySchema, request } from '../../../lib/api';
import { useAppSettings } from '../../../lib/app-settings';
import { useGame } from '../../../lib/game-store';
import { LoginTopbar } from './LoginTopbar';

export function LoginScreen({
  onCancel,
  onSuccess,
}: {
  onCancel: () => void;
  onSuccess: () => void;
}) {
  const world = useGame((state) => state.world);
  const appSettings = useAppSettings((state) => state.settings);
  const expectedUsername = world?.settings.loginUsername || world?.nickname || 'kali';
  const expectedPassword = world?.settings.loginPassword || 'kali';
  const [username, setUsername] = useState(expectedUsername);
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [largeText, setLargeText] = useState(false);
  const [highContrast, setHighContrast] = useState(appSettings.highContrast);
  const [showPassword, setShowPassword] = useState(false);
  const [powerState, setPowerState] = useState<'restart' | 'shutdown' | null>(null);
  const usernameInput = useRef<HTMLInputElement>(null);
  const passwordInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (powerState !== 'restart') {
      return;
    }
    const timer = window.setTimeout(() => {
      setUsername(expectedUsername);
      setPassword('');
      setShowPassword(false);
      setError('');
      setPowerState(null);
    }, 3000);
    return () => window.clearTimeout(timer);
  }, [powerState, expectedUsername]);

  const shutdown = async () => {
    setPowerState('shutdown');
    try {
      if (desktopRuntime) {
        await request('quit_game', {}, emptySchema);
      } else {
        onCancel();
      }
    } catch {
      setPowerState(null);
      setError('Não foi possível desligar o sistema. Tente novamente.');
    }
  };
  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (powerState) {
      return;
    }
    if (username.trim() !== expectedUsername || password !== expectedPassword) {
      setError('Usuário ou senha incorretos. Tente novamente.');
      setPassword('');
      passwordInput.current?.focus();
      return;
    }
    onSuccess();
  };

  return (
    <main
      className={`login-screen${largeText ? ' login-large-text' : ''}${highContrast ? ' login-high-contrast' : ''}`}
      aria-label="Login do Kali Linux"
      aria-busy={powerState !== null}
    >
      {powerState ? (
        <div className="login-power-status" role="status">
          {powerState === 'restart' ? 'Reiniciando Kali Linux…' : 'Desligando…'}
        </div>
      ) : (
        <>
          <LoginTopbar
            hostname={world?.hostname || 'kali'}
            settings={world?.settings ?? {}}
            language={appSettings.language}
            largeText={largeText}
            highContrast={highContrast}
            showPassword={showPassword}
            onLargeText={() => setLargeText((value) => !value)}
            onHighContrast={() => setHighContrast((value) => !value)}
            onShowPassword={() => setShowPassword((value) => !value)}
            onLogin={() => passwordInput.current?.focus()}
            onSwitchUser={() => {
              setUsername('');
              setPassword('');
              setError('');
              setShowPassword(false);
              usernameInput.current?.focus();
            }}
            onCancel={onCancel}
            onRestart={() => setPowerState('restart')}
            onShutdown={() => void shutdown()}
          />
          <form className="login-card" onSubmit={submit}>
            <div className="login-card-body">
              <img
                className="login-card-logo"
                src="/assets/kali/kali-login-logo.png"
                alt="Kali Linux"
              />
              <div className="login-card-fields">
                <label>
                  <span className="sr-only">Usuário</span>
                  <input
                    ref={usernameInput}
                    autoFocus
                    autoComplete="username"
                    autoCapitalize="none"
                    spellCheck={false}
                    value={username}
                    placeholder="Digite seu usuário"
                    onChange={(event) => {
                      setUsername(event.target.value);
                      setError('');
                    }}
                  />
                </label>
                <label>
                  <span className="sr-only">Senha</span>
                  <input
                    ref={passwordInput}
                    type={showPassword ? 'text' : 'password'}
                    autoComplete="current-password"
                    aria-invalid={Boolean(error)}
                    aria-describedby={error ? 'login-error' : undefined}
                    placeholder="Digite sua senha"
                    value={password}
                    onChange={(event) => {
                      setPassword(event.target.value);
                      setError('');
                    }}
                  />
                </label>
              </div>
            </div>
            {error && (
              <p className="login-error" id="login-error" role="alert">
                {error}
              </p>
            )}
            <div className="login-actions">
              <button type="button" onClick={onCancel}>
                Cancelar
              </button>
              <button type="submit" className="login-submit-button">
                Entrar
              </button>
            </div>
          </form>
        </>
      )}
    </main>
  );
}
