import type { SearchResponse } from './goggle-model';
import { assets } from './goggle-model';
import { GoggleIcon } from './GoggleIcon';
import { WebVisual } from '../../virtual-web/WebVisual';

export function GoggleImages({
  results,
  navigate,
  imageSearch,
}: {
  results: SearchResponse;
  navigate: (address: string) => void;
  imageSearch: (source: string) => void;
}) {
  return (
    <div className="goggle-image-grid">
      {results.documents.map((document) => {
        const image = results.images.find((item) => item.id === document.imageId);
        const source = image && assets[image.asset];
        return (
          <article key={document.id}>
            <button className="goggle-image-card" onClick={() => navigate(document.url)}>
              {image?.asset.startsWith('web-') ? (
                <WebVisual kind={image.asset.slice(4)} />
              ) : source ? (
                <img src={source} alt={document.title} loading="lazy" />
              ) : (
                <span className="goggle-image-unavailable">
                  <GoggleIcon name="image" />
                  Prévia indisponível
                </span>
              )}
              <strong>{document.title}</strong>
              <small>{document.domain}</small>
            </button>
            {image && (
              <button
                className="goggle-reverse-button"
                aria-label={`Buscar origem: ${document.title}`}
                onClick={() => imageSearch(image.virtualUrl)}
              >
                <GoggleIcon name="image" />
                Buscar origem
              </button>
            )}
          </article>
        );
      })}
    </div>
  );
}
