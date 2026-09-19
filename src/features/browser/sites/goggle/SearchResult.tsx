import type { SearchDocument } from './goggle-model';
import { money, stockLabel } from '../../virtual-web/web-model';

export function SearchResult({
  document,
  navigate,
}: {
  document: SearchDocument;
  navigate: (address: string) => void;
}) {
  return (
    <article className="goggle-result">
      <div className="goggle-result-site">
        <span className="goggle-favicon" aria-hidden="true">
          {document.domain
            .replace(/^www\./, '')
            .slice(0, 1)
            .toUpperCase()}
        </span>
        <div>
          <span>{document.domain.replace(/^www\./, '')}</span>
          <small>{document.url}</small>
        </div>
      </div>
      <h2>
        <button onClick={() => navigate(document.url)}>{document.title}</button>
      </h2>
      <p>{document.description}</p>
      {document.offer && (
        <p className="goggle-result-offer">
          <strong>{money(document.offer.priceCents)}</strong> · {stockLabel(document.offer.stock)}
        </p>
      )}
    </article>
  );
}
