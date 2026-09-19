import type { GoggleSession } from './goggle-model';
import { GOGGLE_HOME } from './goggle-model';

export function GoggleAccountMenu({
  account,
  navigate,
  logout,
  busy,
}: {
  account: NonNullable<GoggleSession['account']>;
  navigate: (address: string) => void;
  logout: () => void;
  busy: boolean;
}) {
  return (
    <div className="goggle-account-menu" aria-label="Conta Goggle">
      <strong>Conta Goggle</strong>
      <p>
        {account.displayName}
        <br />
        <span>{account.email}</span>
      </p>
      <button className="goggle-outline" onClick={() => navigate(`${GOGGLE_HOME}/account`)}>
        Gerenciar sua conta
      </button>
      <button disabled={busy} onClick={logout}>
        Sair
      </button>
    </div>
  );
}
