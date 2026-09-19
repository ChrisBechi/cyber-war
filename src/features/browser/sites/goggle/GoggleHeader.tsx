import { useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { GoggleAccountMenu } from './GoggleAccountMenu';
import { GoggleAppsMenu } from './GoggleAppsMenu';
import { GoggleLogo } from './GoggleLogo';
import { GOGGLE_HOME, assets } from './goggle-model';
import type { GoggleSession } from './goggle-model';

export function GoggleHeader({
  account,
  navigate,
  logout,
  busy,
  children,
  home,
}: {
  account: GoggleSession['account'];
  navigate: (address: string) => void;
  logout: () => void;
  busy: boolean;
  children?: ReactNode;
  home: boolean;
}) {
  const [panel, setPanel] = useState('');
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!panel) {
      return;
    }
    const dismiss = (e: PointerEvent) => {
      if (e.target instanceof Node && !ref.current?.contains(e.target)) {
        setPanel('');
      }
    };
    const escape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setPanel('');
      }
    };
    document.addEventListener('pointerdown', dismiss);
    document.addEventListener('keydown', escape);
    return () => {
      document.removeEventListener('pointerdown', dismiss);
      document.removeEventListener('keydown', escape);
    };
  }, [panel]);
  const go = (address: string) => {
    setPanel('');
    navigate(address);
  };
  const color = account
    ? Array.from(account.displayName).reduce((sum, c) => sum + c.charCodeAt(0), 0) % 360
    : 0;
  return (
    <header className={`goggle-header ${children ? 'goggle-header-search' : ''}`}>
      {!home && (
        <button
          className="goggle-logo-link"
          aria-label="Página inicial Goggle"
          onClick={() => go(GOGGLE_HOME)}
        >
          <GoggleLogo small />
        </button>
      )}
      {children}
      <div className="goggle-header-account" ref={ref}>
        {account ? (
          <>
            <button className="goggle-text-link" onClick={() => go(`${GOGGLE_HOME}/apps/mail`)}>
              Mail
            </button>
            <button className="goggle-text-link" onClick={() => go(`${GOGGLE_HOME}/images`)}>
              Imagens
            </button>
            <button
              className="goggle-icon-button"
              aria-label="Aplicativos Goggle"
              aria-expanded={panel === 'apps'}
              onClick={() => setPanel(panel === 'apps' ? '' : 'apps')}
            >
              <span className="goggle-app-grid" aria-hidden="true">
                {Array.from({ length: 9 }, (_, i) => (
                  <i key={i} />
                ))}
              </span>
            </button>
            <button
              className="goggle-avatar"
              aria-label={`Conta Goggle: ${account.displayName}`}
              aria-expanded={panel === 'account'}
              style={{ background: `hsl(${color} 42% 36%)` }}
              onClick={() => setPanel(panel === 'account' ? '' : 'account')}
            >
              {account.avatar && assets[account.avatar] ? (
                <img src={assets[account.avatar]} alt="" />
              ) : (
                account.displayName.slice(0, 1).toUpperCase()
              )}
            </button>
            {panel === 'apps' && <GoggleAppsMenu navigate={go} />}
            {panel === 'account' && (
              <GoggleAccountMenu
                account={account}
                navigate={go}
                busy={busy}
                logout={() => {
                  setPanel('');
                  logout();
                }}
              />
            )}
          </>
        ) : (
          <button className="goggle-blue" onClick={() => go(`${GOGGLE_HOME}/login`)}>
            Fazer login
          </button>
        )}
      </div>
    </header>
  );
}
