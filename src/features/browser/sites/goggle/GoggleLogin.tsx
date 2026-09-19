import { useState } from 'react';
import { emptySchema } from '../../../../lib/api';
import { perform } from '../../../../lib/game-store';
import { GoggleLogo } from './GoggleLogo';

export function GoggleLogin({ complete }: { complete: () => void }) {
  const [register, setRegister] = useState(false);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [name, setName] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const submit = async () => {
    setBusy(true);
    setError('');
    try {
      await perform(
        'goggle_authenticate',
        { email, password, displayName: register ? name : null },
        emptySchema,
      );
      setPassword('');
      complete();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <main className="goggle-login">
      <GoggleLogo small />
      <h1>{register ? 'Crie sua conta Goggle' : 'Fazer login'}</h1>
      <p>Uma conta para explorar seu mundo.</p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        {register && (
          <label>
            Seu nome
            <input
              required
              value={name}
              maxLength={80}
              onChange={(e) => setName(e.target.value)}
              autoComplete="off"
            />
          </label>
        )}
        <label>
          E-mail Goggle
          <input
            type="email"
            required
            placeholder="player@goggle.com"
            value={email}
            maxLength={51}
            onChange={(e) => setEmail(e.target.value)}
            autoComplete="off"
          />
        </label>
        <label>
          Senha fictícia
          <input
            type="password"
            required
            minLength={4}
            maxLength={128}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="off"
          />
        </label>
        <p className="goggle-note">Use credenciais fictícias para esta campanha.</p>
        {error && <p role="alert">{error}</p>}
        <div className="goggle-login-actions">
          <button
            type="button"
            disabled={busy}
            onClick={() => {
              setRegister(!register);
              setError('');
            }}
          >
            {register ? 'Já tenho uma conta' : 'Criar conta'}
          </button>
          <button type="submit" className="goggle-blue" disabled={busy}>
            {busy ? 'Aguarde…' : register ? 'Criar conta' : 'Entrar'}
          </button>
        </div>
      </form>
    </main>
  );
}
