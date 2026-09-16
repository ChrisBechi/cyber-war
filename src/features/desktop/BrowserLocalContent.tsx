import { useState } from 'react';
import type { BrowserLocalFile } from './browser-local-file';
import { useMediaSource } from '../media/use-media-source';
import './browser-local-file.css';

function LocalMedia({ file }: { file: BrowserLocalFile }) {
  const { source, node, error } = useMediaSource(file.path);
  const [failed, setFailed] = useState(false);
  if (error || failed) {
    return <p role="alert">{error || 'Não foi possível exibir este formato.'}</p>;
  }
  if (!node) {
    return <p role="status">Carregando arquivo…</p>;
  }
  if (!source) {
    return <p role="alert">Este arquivo não possui uma prévia disponível.</p>;
  }
  if (file.kind === 'image') {
    return <img src={source} alt={node.name} onError={() => setFailed(true)} />;
  }
  if (file.kind === 'audio') {
    return <audio src={source} controls onError={() => setFailed(true)} />;
  }
  return <video src={source} controls onError={() => setFailed(true)} />;
}

export function BrowserLocalContent({ file, title }: { file: BrowserLocalFile; title: string }) {
  return (
    <article className="browser-local-file">
      <h1>{title}</h1>
      <p>{file.path}</p>
      {file.kind === 'text' ? <pre>{file.text}</pre> : <LocalMedia key={file.path} file={file} />}
    </article>
  );
}
