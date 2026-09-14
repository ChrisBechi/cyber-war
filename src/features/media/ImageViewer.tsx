import { useEffect, useRef, useState } from 'react';
import { useMediaSource } from './use-media-source';

export function ImageViewer({ initialPath }: { initialPath?: string }) {
  const { node, source, error: sourceError } = useMediaSource(initialPath);
  const [zoom, setZoom] = useState(1);
  const [rotation, setRotation] = useState(0);
  const [fit, setFit] = useState(true);
  const [fullscreen, setFullscreen] = useState(false);
  const [error, setError] = useState('');
  const viewerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setZoom(1);
    setRotation(0);
    setFit(true);
    setError('');
  }, [initialPath]);

  useEffect(() => {
    const onFullscreen = () => setFullscreen(document.fullscreenElement === viewerRef.current);
    document.addEventListener('fullscreenchange', onFullscreen);
    return () => document.removeEventListener('fullscreenchange', onFullscreen);
  }, []);

  const path = initialPath ?? '';
  const toggleFullscreen = () => {
    if (document.fullscreenElement) {
      void document.exitFullscreen().catch(() => setError('Não foi possível sair da tela cheia.'));
    } else {
      void viewerRef.current?.requestFullscreen().catch(() => setFullscreen(false));
    }
  };

  if (!initialPath) {
    return <div className="image-empty">Abra uma imagem para iniciar o Ristretto.</div>;
  }
  if (error || sourceError) {
    return <div className="image-empty error">{error || sourceError}</div>;
  }

  return (
    <div
      ref={viewerRef}
      className={`image-viewer ${fit ? 'fit-image' : ''} ${fullscreen ? 'is-fullscreen' : ''}`}
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget) {
          return;
        }
        if (event.key === '+' || event.key === '=') {
          setFit(false);
          setZoom((value) => Math.min(5, value + 0.25));
        } else if (event.key === '-') {
          setFit(false);
          setZoom((value) => Math.max(0.25, value - 0.25));
        } else if (event.key === '0') {
          setFit(false);
          setZoom(1);
        } else if (event.key.toLowerCase() === 'r') {
          setRotation((value) => (value + 90) % 360);
        } else if (event.key.toLowerCase() === 'f') {
          toggleFullscreen();
        }
      }}
    >
      <div className="image-toolbar">
        <button
          type="button"
          onClick={() => {
            setFit(false);
            setZoom((value) => Math.max(0.25, value - 0.25));
          }}
          aria-label="Reduzir zoom"
        >
          −
        </button>
        <span>{Math.round(zoom * 100)}%</span>
        <button
          type="button"
          onClick={() => {
            setFit(false);
            setZoom((value) => Math.min(5, value + 0.25));
          }}
          aria-label="Aumentar zoom"
        >
          +
        </button>
        <button
          type="button"
          onClick={() => {
            setFit(true);
            setZoom(1);
          }}
          aria-pressed={fit}
        >
          Ajustar
        </button>
        <button
          type="button"
          onClick={() => setRotation((value) => (value + 90) % 360)}
          aria-label="Girar imagem"
        >
          ↻
        </button>
        <button type="button" onClick={toggleFullscreen} aria-label="Tela cheia">
          ⛶
        </button>
      </div>
      <div className="image-stage">
        {source ? (
          <img
            src={source}
            alt={path.split('/').pop()}
            style={{ transform: `scale(${zoom}) rotate(${rotation}deg)` }}
            onDoubleClick={toggleFullscreen}
            onError={() => setError('Imagem inválida ou formato não suportado por este WebView.')}
          />
        ) : (
          <div className="image-unavailable">
            <strong>Imagem virtual sem prévia decodificável</strong>
            <span>
              Importe uma imagem no gerenciador. A prévia depende do formato e deste WebView.
            </span>
          </div>
        )}
      </div>
      <footer className="app-status">
        {path.split('/').pop()} ·{' '}
        {node?.blob
          ? `${node.blob.size} bytes`
          : `${node?.content.length ?? 0} caracteres virtuais`}
        <span>+ − zoom · 0 tamanho real · R girar · F tela cheia</span>
      </footer>
    </div>
  );
}
