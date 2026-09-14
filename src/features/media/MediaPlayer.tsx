import { useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { request } from '../../lib/api';
import { useMediaSource } from './use-media-source';
import { audioManager } from '../../lib/audio-manager';
import { associationForPath, subtitlePathFor } from '../../lib/file-associations';

function formatTime(value: number): string {
  if (!Number.isFinite(value) || value < 0) {
    return '00:00';
  }
  const total = Math.floor(value);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
    : `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

function toVtt(content: string): string {
  if (content.trimStart().startsWith('WEBVTT')) {
    return content;
  }
  return `WEBVTT\n\n${content.replace(/(\d{2}:\d{2}:\d{2}),(\d{3})/g, '$1.$2')}`;
}

export function MediaPlayer({ initialPath }: { initialPath?: string }) {
  const { node, source, error: sourceError } = useMediaSource(initialPath);
  const [error, setError] = useState('');
  const [current, setCurrent] = useState(0);
  const [duration, setDuration] = useState(0);
  const [volume, setVolume] = useState(0.85);
  const [muted, setMuted] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [loop, setLoop] = useState(false);
  const [speed, setSpeed] = useState(1);
  const [subtitleUrl, setSubtitleUrl] = useState<string | null>(null);
  const [subtitles, setSubtitles] = useState(true);
  const [fullscreen, setFullscreen] = useState(false);
  const mediaRef = useRef<HTMLMediaElement>(null);
  const playerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setError('');
    setCurrent(0);
    setDuration(0);
    setPlaying(false);
  }, [initialPath]);

  const path = initialPath ?? '';
  const association = associationForPath(path, node ?? undefined);
  const isVideo = association.kind === 'video';

  useEffect(() => {
    if (!isVideo || !path) {
      setSubtitleUrl(null);
      return;
    }
    let currentRequest = true;
    setSubtitleUrl(null);
    void request('vfs_read', { path: subtitlePathFor(path) }, z.string())
      .then((content) => {
        if (currentRequest) {
          setSubtitleUrl(`data:text/vtt;charset=utf-8,${encodeURIComponent(toVtt(content))}`);
        }
      })
      .catch(() =>
        request('vfs_read', { path: path.replace(/\.[^/.]+$/, '.srt') }, z.string()).then(
          (content) => {
            if (currentRequest) {
              setSubtitleUrl(`data:text/vtt;charset=utf-8,${encodeURIComponent(toVtt(content))}`);
            }
          },
          () => {
            if (currentRequest) {
              setSubtitleUrl(null);
            }
          },
        ),
      )
      .catch(() => {
        if (currentRequest) {
          setSubtitleUrl(null);
        }
      });
    return () => {
      currentRequest = false;
    };
  }, [isVideo, path]);

  useEffect(() => {
    const media = mediaRef.current;
    if (media) {
      return audioManager.attach(media, 'music');
    }
  }, [source]);

  useEffect(() => {
    const media = mediaRef.current;
    if (!media) {
      return;
    }
    audioManager.setMediaLevel(media, volume);
    media.muted = muted;
    media.loop = loop;
    media.playbackRate = speed;
    if (media.textTracks.length > 0) {
      media.textTracks[0].mode = subtitles ? 'showing' : 'hidden';
    }
  }, [loop, muted, speed, subtitles, volume, source, subtitleUrl]);

  useEffect(() => {
    const onFullscreen = () => setFullscreen(document.fullscreenElement === playerRef.current);
    document.addEventListener('fullscreenchange', onFullscreen);
    return () => document.removeEventListener('fullscreenchange', onFullscreen);
  }, []);

  useEffect(() => {
    const media = mediaRef.current;
    const element = media?.querySelector('track');
    if (!media || !element) {
      return;
    }
    const update = () => {
      for (const track of Array.from(media.textTracks)) {
        track.mode = subtitles ? 'showing' : 'hidden';
      }
    };
    update();
    element.addEventListener('load', update);
    return () => element.removeEventListener('load', update);
  }, [source, subtitleUrl, subtitles]);

  const togglePlayback = () => {
    const media = mediaRef.current;
    if (!media) {
      return;
    }
    if (media.paused) {
      void media.play().catch(() => setError('O navegador não conseguiu reproduzir esta mídia.'));
    } else {
      media.pause();
    }
  };
  const seek = (seconds: number) => {
    const media = mediaRef.current;
    if (media && Number.isFinite(media.duration) && media.duration > 0) {
      media.currentTime = Math.max(
        0,
        Math.min(media.duration || duration, media.currentTime + seconds),
      );
      setCurrent(media.currentTime);
    }
  };
  const toggleFullscreen = () => {
    if (document.fullscreenElement) {
      void document.exitFullscreen().catch(() => setError('Não foi possível sair da tela cheia.'));
    } else {
      void playerRef.current?.requestFullscreen().catch(() => setFullscreen(false));
    }
  };

  if (!initialPath) {
    return (
      <div className="media-empty">Abra um arquivo de áudio ou vídeo para iniciar o Parole.</div>
    );
  }
  if (error || sourceError) {
    return <div className="media-empty error">{error || sourceError}</div>;
  }
  if (association.kind !== 'audio' && association.kind !== 'video') {
    return <div className="media-empty">Este arquivo não é uma mídia de áudio ou vídeo.</div>;
  }

  return (
    <div
      ref={playerRef}
      className={`media-player ${isVideo ? 'is-video' : 'is-audio'} ${fullscreen ? 'is-fullscreen' : ''}`}
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget) {
          return;
        }
        if (event.key === ' ') {
          event.preventDefault();
          togglePlayback();
        } else if (event.key === 'ArrowLeft') {
          event.preventDefault();
          seek(-10);
        } else if (event.key === 'ArrowRight') {
          event.preventDefault();
          seek(10);
        } else if (event.key.toLowerCase() === 'm') {
          setMuted((value) => !value);
        } else if (event.key.toLowerCase() === 'f') {
          toggleFullscreen();
        }
      }}
    >
      <div className="media-stage">
        {source ? (
          isVideo ? (
            <video
              key={source}
              ref={(element) => {
                mediaRef.current = element;
              }}
              src={source}
              preload="metadata"
              onLoadedMetadata={(event) =>
                setDuration(
                  Number.isFinite(event.currentTarget.duration) ? event.currentTarget.duration : 0,
                )
              }
              onError={() => setError('Vídeo inválido ou codec não suportado por este WebView.')}
              onTimeUpdate={(event) => setCurrent(event.currentTarget.currentTime)}
              onPlay={() => setPlaying(true)}
              onPause={() => setPlaying(false)}
              onEnded={() => setPlaying(false)}
              onDoubleClick={toggleFullscreen}
              aria-label={path.split('/').pop()}
            >
              {subtitleUrl && (
                <track
                  kind="subtitles"
                  src={subtitleUrl}
                  srcLang="pt-BR"
                  label="Português"
                  default
                />
              )}
            </video>
          ) : (
            <div className="audio-art" aria-hidden="true">
              <span>♪</span>
              <span>♫</span>
              <span>♬</span>
            </div>
          )
        ) : (
          <div className="media-unavailable">
            <strong>Arquivo virtual sem fonte decodificável</strong>
            <span>
              Importe um arquivo no gerenciador. A reprodução depende do formato e dos codecs
              disponíveis neste WebView.
            </span>
          </div>
        )}
        {!isVideo && source && (
          <audio
            key={source}
            ref={(element) => {
              mediaRef.current = element;
            }}
            src={source}
            preload="metadata"
            onLoadedMetadata={(event) =>
              setDuration(
                Number.isFinite(event.currentTarget.duration) ? event.currentTarget.duration : 0,
              )
            }
            onError={() => setError('Áudio inválido ou codec não suportado por este WebView.')}
            onTimeUpdate={(event) => setCurrent(event.currentTarget.currentTime)}
            onPlay={() => setPlaying(true)}
            onPause={() => setPlaying(false)}
            onEnded={() => setPlaying(false)}
            aria-label={path.split('/').pop()}
          />
        )}
      </div>
      <div className="media-info">
        <strong>{path.split('/').pop()}</strong>
        <span>
          {isVideo ? 'Vídeo' : 'Áudio'} · {association.mime}
        </span>
      </div>
      <div className="media-timeline">
        <span>{formatTime(current)}</span>
        <input
          aria-label="Barra de reprodução"
          type="range"
          min={0}
          max={duration || 0}
          step={0.1}
          value={Math.min(current, duration || 0)}
          disabled={!source || duration <= 0}
          onChange={(event) => {
            const value = Math.max(0, Math.min(duration, Number(event.target.value)));
            if (mediaRef.current) {
              mediaRef.current.currentTime = value;
            }
            setCurrent(value);
          }}
        />
        <span>{formatTime(duration)}</span>
      </div>
      <div className="media-controls">
        <button
          type="button"
          onClick={togglePlayback}
          aria-label={playing ? 'Pausar' : 'Reproduzir'}
        >
          {playing ? '❚❚' : '▶'}
        </button>
        <button type="button" onClick={() => seek(-10)} aria-label="Voltar 10 segundos">
          −10
        </button>
        <button type="button" onClick={() => seek(10)} aria-label="Avançar 10 segundos">
          +10
        </button>
        <button
          type="button"
          onClick={() => setLoop((value) => !value)}
          aria-pressed={loop}
          aria-label="Repetir"
        >
          ↻
        </button>
        <label className="media-volume">
          <button
            type="button"
            onClick={() => setMuted((value) => !value)}
            aria-label={muted ? 'Ativar som' : 'Silenciar'}
          >
            {muted || volume === 0 ? '🔇' : '🔊'}
          </button>
          <input
            aria-label="Volume"
            type="range"
            min={0}
            max={1}
            step={0.01}
            value={muted ? 0 : volume}
            onChange={(event) => {
              setVolume(Number(event.target.value));
              setMuted(false);
            }}
          />
        </label>
        <label className="media-speed">
          Velocidade
          <select
            aria-label="Velocidade de reprodução"
            value={speed}
            onChange={(event) => setSpeed(Number(event.target.value))}
          >
            {[0.5, 0.75, 1, 1.25, 1.5, 2].map((value) => (
              <option key={value} value={value}>
                {value}×
              </option>
            ))}
          </select>
        </label>
        {isVideo && subtitleUrl && (
          <button
            type="button"
            onClick={() => setSubtitles((value) => !value)}
            aria-pressed={subtitles}
            aria-label="Alternar legendas"
          >
            CC
          </button>
        )}
        <button
          type="button"
          onClick={toggleFullscreen}
          aria-pressed={fullscreen}
          aria-label="Tela cheia"
        >
          ⛶
        </button>
      </div>
      <footer className="app-status">
        {source ? 'Fonte local do sistema virtual' : 'Prévia sem fonte'}
        <span>Espaço reproduz · ← → 10s · M volume · F tela cheia</span>
      </footer>
    </div>
  );
}
