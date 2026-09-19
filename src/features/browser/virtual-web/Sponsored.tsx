import { z } from 'zod';
import { useState } from 'react';
import { perform } from '../../../lib/game-store';
import type { adSchema } from './web-model';

export function Sponsored({
  ads,
  navigate,
}: {
  ads: z.infer<typeof adSchema>[];
  navigate: (url: string) => void;
}) {
  const [error, setError] = useState('');
  const [pending, setPending] = useState(false);
  if (!ads.length) {
    return null;
  }
  const open = async (id: string) => {
    if (pending) {
      return;
    }
    setPending(true);
    setError('');
    try {
      const url = await perform('web_ad_click', { id }, z.string());
      navigate(url);
    } catch (e) {
      setError(String(e));
    } finally {
      setPending(false);
    }
  };
  return (
    <aside className="web-sponsored" aria-label="Publicidade">
      {ads.map((ad) => (
        <article key={ad.id}>
          <small>Patrocinado · {ad.advertiser}</small>
          <h3>
            <button disabled={pending} onClick={() => void open(ad.id)}>
              {ad.title}
            </button>
          </h3>
          <p>{ad.description}</p>
          <span>{ad.url.replace('https://www.', '')}</span>
        </article>
      ))}
      {error && <p role="alert">{error}</p>}
    </aside>
  );
}
