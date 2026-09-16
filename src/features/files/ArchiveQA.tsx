import { useEffect, useState } from 'react';
import { Desktop } from '../desktop/Desktop';
import { useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { desktopRuntime } from '../../lib/api';

/** Native development fixture: all interactions still use the real Rust IPC. */
export function ArchiveQA() {
  const [error, setError] = useState('');
  const [ready, setReady] = useState(false);
  useEffect(() => {
    if (!desktopRuntime) {
      setError('Inicie o Tauri com CYBER_WAR_ARCHIVE_QA=1.');
      return;
    }
    void useGame
      .getState()
      .refresh()
      .then(() => {
        useWindows.getState().reset();
        useWindows.getState().open('files', '/home/kali/Downloads');
        setReady(true);
      })
      .catch((e: unknown) => setError(String(e)));
  }, []);
  if (!ready) {
    return <p role="status">{error || 'Preparando sessão isolada de QA…'}</p>;
  }
  return <Desktop onMenu={() => location.reload()} />;
}
