import { useEffect, useState } from 'react';
import { z } from 'zod';
import { request } from '../../../../lib/api';
import { GoggleDialog } from './GoggleDialog';
import { GoggleIcon } from './GoggleIcon';

export function VoiceSearch({
  close,
  search,
}: {
  close: () => void;
  search: (query: string) => void;
}) {
  const [options, setOptions] = useState<string[]>([]);
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    void request('search_voice_options', {}, z.array(z.string()))
      .then((items) => {
        if (active) {
          setOptions(items);
        }
      })
      .catch((e: unknown) => {
        if (active) {
          setError(String(e));
        }
      });
    return () => {
      active = false;
    };
  }, []);
  return (
    <GoggleDialog title="Pesquisar por voz" close={close}>
      <div className="goggle-voice">
        <span className="goggle-mic-disc">
          <GoggleIcon name="mic" />
        </span>
        <h3>Fale agora</h3>
        <p>Modo simulado · escolha uma fala para pesquisar.</p>
        {error && <p role="alert">{error}</p>}
        <div>
          {options.map((option) => (
            <button key={option} onClick={() => search(option)}>
              {option}
            </button>
          ))}
        </div>
        <small>O microfone do seu computador permanece desligado.</small>
      </div>
    </GoggleDialog>
  );
}
