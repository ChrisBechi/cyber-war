import { useEffect, useState } from 'react';
import { z } from 'zod';
import { request } from '../../../../lib/api';
import { GoggleDialog } from './GoggleDialog';
import { GoggleIcon } from './GoggleIcon';

export function ImageSearch({
  close,
  search,
}: {
  close: () => void;
  search: (source: string) => void;
}) {
  const [mode, setMode] = useState<'vfs' | 'url'>('vfs');
  const [files, setFiles] = useState<string[]>([]);
  const [url, setUrl] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    void request('search_image_files', {}, z.array(z.string()))
      .then((files) => {
        if (active) {
          setFiles(files);
        }
      })
      .catch((e: unknown) => {
        if (active) {
          setError(String(e));
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, []);
  return (
    <GoggleDialog title="Pesquisar por imagem" close={close}>
      <div className="goggle-image-intro">
        <GoggleIcon name="image" />
        <p>Encontre a história por trás de uma imagem.</p>
      </div>
      <div role="tablist" aria-label="Origem da imagem" className="goggle-image-tabs">
        <button role="tab" aria-selected={mode === 'vfs'} onClick={() => setMode('vfs')}>
          Enviar imagem
        </button>
        <button role="tab" aria-selected={mode === 'url'} onClick={() => setMode('url')}>
          Usar URL
        </button>
      </div>
      {error && <p role="alert">{error}</p>}
      {mode === 'vfs' ? (
        <div role="tabpanel">
          <p>Escolha uma imagem dos arquivos virtuais.</p>
          {loading ? (
            <p role="status">Consultando arquivos…</p>
          ) : files.length ? (
            <div className="goggle-vfs-files">
              {files.map((file) => (
                <button key={file} onClick={() => search(file)}>
                  <GoggleIcon name="image" />
                  {file}
                </button>
              ))}
            </div>
          ) : (
            <p>
              Nenhuma imagem disponível no VFS. Imagens obtidas durante suas investigações
              aparecerão aqui.
            </p>
          )}
        </div>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (url.trim()) {
              search(url.trim());
            }
          }}
        >
          <label>
            URL da internet virtual
            <input
              aria-label="URL da imagem virtual"
              value={url}
              maxLength={512}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://www.orion.com/media/campus.svg"
              required
            />
          </label>
          <button className="goggle-blue" type="submit">
            Pesquisar imagem
          </button>
          <p className="goggle-note">Somente imagens cadastradas na rede do jogo.</p>
        </form>
      )}
    </GoggleDialog>
  );
}
