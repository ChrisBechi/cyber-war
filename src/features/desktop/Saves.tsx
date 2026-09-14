import { useEffect, useState } from 'react';
import { z } from 'zod';
import { checkpointSchema, emptySchema, request, worldSchema } from '../../lib/api';
import type { Checkpoint } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { endSession } from '../../lib/session';

export function Saves({ onMenu }: { onMenu: () => void }) {
  const { slot, revision, busy } = useGame();
  const [checkpoints, setCheckpoints] = useState<Checkpoint[]>([]);
  const [restoring, setRestoring] = useState<Checkpoint | null>(null);
  const [notice, setNotice] = useState('');
  useEffect(() => {
    let active = true;
    void request('list_checkpoints', { slotIndex: slot }, z.array(checkpointSchema))
      .then((rows) => {
        if (active) {
          setCheckpoints(rows);
        }
      })
      .catch((e: unknown) => setNotice(String(e)));
    return () => {
      active = false;
    };
  }, [slot, revision]);
  return (
    <div className="app-scroll">
      <div className="section-heading">
        <div className="eyebrow">CAMPANHA / SLOT 0{slot}</div>
        <h2>Você pode voltar.</h2>
        <p className="muted">
          Arquivos, configurações e serviços são persistidos. Ao sair, uma missão em andamento é
          encerrada; salvar agora cria um save manual explícito.
        </p>
        <div className="toolbar">
          <button
            className="primary"
            disabled={busy}
            onClick={() => {
              void perform('save_slot', {}, emptySchema)
                .then(() => setNotice('Save manual gravado.'))
                .catch(() => undefined);
            }}
          >
            Salvar agora
          </button>
          <button
            disabled={busy}
            onClick={() => {
              void endSession(onMenu).catch(() => undefined);
            }}
          >
            Encerrar sessão e voltar ao menu
          </button>
        </div>
        {notice && <p role="status">{notice}</p>}
      </div>
      {restoring && (
        <div className="confirm-panel" role="alertdialog" aria-label="Restaurar checkpoint">
          <p>
            Voltar para “{restoring.label}”? O progresso posterior e os checkpoints seguintes serão
            substituídos.
          </p>
          <button onClick={() => setRestoring(null)}>Cancelar</button>
          <button
            className="primary"
            disabled={busy}
            onClick={() => {
              void perform('restore_checkpoint', { checkpointId: restoring.id }, worldSchema)
                .then(() => {
                  setRestoring(null);
                  useWindows.getState().reset();
                  useWindows.getState().open('missions');
                })
                .catch(() => undefined);
            }}
          >
            Restaurar
          </button>
        </div>
      )}
      <div className="checkpoint-list">
        {checkpoints.map((c) => (
          <button key={c.id} onClick={() => setRestoring(c)}>
            <span>
              <b>{c.label}</b>
              <small>
                {c.checkpointType} · {c.missionId || 'Campanha'}
              </small>
            </span>
            <time>{new Date(c.createdAt).toLocaleString('pt-BR')}</time>
            <span>↶</span>
          </button>
        ))}
      </div>
    </div>
  );
}
