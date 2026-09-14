import { useMemo } from 'react';
import { z } from 'zod';
import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { ForumBoard } from '../forum/ForumBoard';
import { forumStateSchema } from '../forum/forum-model';

export function Forum() {
  const world = useGame((store) => store.world);
  const saved = world?.settings.forumState;
  const result = useMemo(() => {
    try {
      return forumStateSchema.safeParse(saved ? JSON.parse(saved) : {});
    } catch {
      return null;
    }
  }, [saved]);
  if (!world) {
    return <div className="empty-state">Carregando comunidade…</div>;
  }
  if (!result?.success) {
    return (
      <div className="empty-state" role="alert">
        Não foi possível ler os dados salvos do fórum. A gravação foi bloqueada para preservar o
        conteúdo existente.
      </div>
    );
  }
  return (
    <ForumBoard
      nickname={world.nickname}
      reputation={world.reputation}
      flags={world.flags}
      connected={world.network.connected}
      state={result.data}
      onAction={(action) => perform('forum_action', { action }, z.string().nullable())}
      onMission={() => useWindows.getState().open('missions')}
    />
  );
}
