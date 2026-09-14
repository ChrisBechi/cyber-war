import { useEffect, useRef, useState } from 'react';
import { audioManager } from '../../lib/audio-manager';
import { openingSources } from '../boot/boot-assets';
import { useAdvance } from '../boot/use-advance';

/** Captured gameplay; music and the shared interface sounds retain independent volume buses. */
export function OpeningCinematic({
  onFinish,
  sources = openingSources,
}: {
  onFinish: () => void;
  sources?: string[];
}) {
  const [source, setSource] = useState(0);
  const [cursorHidden, setCursorHidden] = useState(false);
  const video = useRef<HTMLVideoElement>(null);
  const effects = useRef<HTMLAudioElement>(null);
  const cursorTimer = useRef<number | undefined>(undefined);
  const { finish, skip } = useAdvance(onFinish, 1000);
  const move = () => {
    setCursorHidden(false);
    window.clearTimeout(cursorTimer.current);
    cursorTimer.current = window.setTimeout(() => setCursorHidden(true), 2200);
  };
  useEffect(() => {
    const media = video.current;
    const sfx = effects.current;
    if (!sources[source] || !media || !sfx) {
      finish();
      return;
    }
    const detach = audioManager.attach(media, 'music');
    const detachEffects = audioManager.attach(sfx, 'sfx');
    let active = true;
    let watchdog: number;
    const fallback = () => {
      if (active) {
        setSource((index) => (index === source ? source + 1 : index));
      }
    };
    const armWatchdog = () => {
      window.clearTimeout(watchdog);
      watchdog = window.setTimeout(fallback, 12000);
    };
    const sync = () => {
      armWatchdog();
      if (media.currentTime >= 90) {
        finish();
        return;
      }
      if (Number.isFinite(sfx.duration) && Math.abs(sfx.currentTime - media.currentTime) > 0.12) {
        sfx.currentTime = media.currentTime;
      }
      if (!media.paused && sfx.paused) {
        void sfx.play().catch(() => undefined);
      }
    };
    const pause = () => sfx.pause();
    const visibility = () => {
      if (document.hidden) {
        window.clearTimeout(watchdog);
      } else {
        armWatchdog();
      }
    };
    document.addEventListener('visibilitychange', visibility);
    media.addEventListener('playing', sync);
    media.addEventListener('timeupdate', sync);
    media.addEventListener('waiting', pause);
    media.addEventListener('pause', pause);
    armWatchdog();
    void media.play().catch(() => {
      if (!active) {
        return;
      }
      media.muted = true;
      return media.play().catch(fallback);
    });
    cursorTimer.current = window.setTimeout(() => setCursorHidden(true), 2200);
    return () => {
      active = false;
      window.clearTimeout(watchdog);
      window.clearTimeout(cursorTimer.current);
      media.removeEventListener('playing', sync);
      media.removeEventListener('timeupdate', sync);
      media.removeEventListener('waiting', pause);
      media.removeEventListener('pause', pause);
      document.removeEventListener('visibilitychange', visibility);
      detach();
      detachEffects();
    };
  }, [source, sources, finish]);
  return (
    <section
      className={`boot-screen opening-cinematic ${cursorHidden ? 'cursor-hidden' : ''}`}
      aria-label="Abertura CYBER WAR"
      onClick={skip}
      onPointerMove={move}
    >
      {sources[source] && (
        <video
          key={sources[source]}
          ref={video}
          src={sources[source]}
          autoPlay
          playsInline
          preload="auto"
          controls={false}
          onEnded={finish}
          onError={() => setSource((index) => (index === source ? source + 1 : index))}
        />
      )}
      <audio ref={effects} src="/assets/audio/cyber-war-opening-sfx.ogg" preload="auto" />
    </section>
  );
}
