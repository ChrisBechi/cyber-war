export type AudioBus = 'music' | 'sfx' | 'voice';
export type AudioVolumes = { master: number; music: number; sfx: number; voice: number };
export type AudioCue =
  | 'ui-click'
  | 'ui-key'
  | 'ui-window'
  | 'ui-message'
  | 'ui-notification'
  | 'ui-download'
  | 'ui-alert'
  | 'ui-terminal'
  | 'studio-intro-ambience'
  | 'keyboard-loop'
  | 'logo-reveal'
  | 'terminal-clear'
  | 'menu-hover'
  | 'menu-select'
  | 'menu-ambience';
const cues: AudioCue[] = [
  'ui-click',
  'ui-key',
  'ui-window',
  'ui-message',
  'ui-notification',
  'ui-download',
  'ui-alert',
  'ui-terminal',
  'studio-intro-ambience',
  'keyboard-loop',
  'logo-reveal',
  'terminal-clear',
  'menu-hover',
  'menu-select',
  'menu-ambience',
];
type Track = {
  media: HTMLMediaElement;
  bus: AudioBus;
  level: number;
  startLevel: number;
  target: number;
  started: number;
  duration: number;
  removeAfterFade: boolean;
  loop: boolean;
};

export class AudioManager {
  private observers = new Set<(cue: AudioCue, bus: AudioBus) => void>();
  observe(listener: (cue: AudioCue, bus: AudioBus) => void): () => void {
    this.observers.add(listener);
    return () => {
      this.observers.delete(listener);
    };
  }
  private volumes: AudioVolumes = { master: 80, music: 70, sfx: 80, voice: 80 };
  private tracks = new Map<string, Track>();
  private templates = new Map<AudioCue, HTMLAudioElement>();
  private frame = 0;
  private counter = 0;
  private paused = false;
  private hoverTime = 0;
  preload(): Promise<void> {
    return Promise.all(
      cues.map(
        (cue) =>
          new Promise<void>((resolve) => {
            const media = new Audio(`/assets/audio/${cue}.ogg`);
            this.templates.set(cue, media);
            media.preload = 'auto';
            const timeout = window.setTimeout(done, 1200);
            function done() {
              window.clearTimeout(timeout);
              media.removeEventListener('canplaythrough', done);
              media.removeEventListener('error', done);
              resolve();
            }
            media.addEventListener('canplaythrough', done, { once: true });
            media.addEventListener('error', done, { once: true });
            media.load();
          }),
      ),
    ).then(() => undefined);
  }
  setVolumes(volumes: AudioVolumes): void {
    this.volumes = volumes;
    for (const track of this.tracks.values()) {
      this.volume(track);
    }
  }
  getVolumes(): AudioVolumes {
    return { ...this.volumes };
  }
  setMediaLevel(media: HTMLMediaElement, level: number): void {
    for (const track of this.tracks.values()) {
      if (track.media === media) {
        track.level = Math.max(0, Math.min(1, level));
        this.volume(track);
      }
    }
  }
  private volume(track: Track): void {
    track.media.volume = Math.max(
      0,
      Math.min(1, (((this.volumes.master / 100) * this.volumes[track.bus]) / 100) * track.level),
    );
  }
  private playMedia(track: Track): void {
    if (!this.paused) {
      void track.media.play().catch(() => {
        // Failed one-shots have nothing to resume. Keep loops available for a user gesture.
        if (!track.loop) {
          for (const [id, current] of this.tracks) {
            if (current === track) {
              this.stop(id);
              break;
            }
          }
        }
      });
    }
  }
  unlock(): void {
    for (const track of this.tracks.values()) {
      if (track.loop && track.media.paused && !track.removeAfterFade) {
        this.playMedia(track);
      }
    }
  }
  play(cue: AudioCue, bus: AudioBus = 'sfx', loop = false, fade = 0): string {
    this.observers.forEach((observer) => observer(cue, bus));
    if (cue === 'menu-hover' && performance.now() - this.hoverTime < 85) {
      return '';
    }
    if (cue === 'menu-hover') {
      this.hoverTime = performance.now();
    }
    const media =
      (this.templates.get(cue)?.cloneNode(true) as HTMLAudioElement | undefined) ??
      new Audio(`/assets/audio/${cue}.ogg`);
    media.loop = loop;
    const id = `${cue}:${++this.counter}`;
    const track: Track = {
      media,
      bus,
      level: fade ? 0 : 1,
      startLevel: 0,
      target: 1,
      started: performance.now(),
      duration: fade,
      removeAfterFade: false,
      loop,
    };
    this.tracks.set(id, track);
    media.addEventListener('ended', () => this.stop(id), { once: true });
    media.addEventListener('error', () => this.stop(id), { once: true });
    this.volume(track);
    this.playMedia(track);
    if (fade) {
      this.animate();
    }
    return id;
  }
  attach(media: HTMLMediaElement, bus: AudioBus): () => void {
    const id = `external:${++this.counter}`;
    const track: Track = {
      media,
      bus,
      level: 1,
      startLevel: 1,
      target: 1,
      started: 0,
      duration: 0,
      removeAfterFade: false,
      loop: true,
    };
    this.tracks.set(id, track);
    this.volume(track);
    return () => this.stop(id);
  }
  music(cue: AudioCue): string {
    for (const [id, track] of this.tracks) {
      if (track.bus === 'music') {
        this.stop(id, 500);
      }
    }
    return this.play(cue, 'music', true, 700);
  }
  stop(id: string, fade = 0): void {
    const track = this.tracks.get(id);
    if (!track) {
      return;
    }
    if (fade) {
      track.startLevel = track.level;
      track.target = 0;
      track.started = performance.now();
      track.duration = fade;
      track.removeAfterFade = true;
      this.animate();
    } else {
      track.media.pause();
      this.tracks.delete(id);
    }
  }
  stopAll(fade = 0): void {
    for (const id of this.tracks.keys()) {
      this.stop(id, fade);
    }
  }
  pause(): void {
    this.paused = true;
    for (const track of this.tracks.values()) {
      track.media.pause();
    }
  }
  resume(): void {
    this.paused = false;
    this.unlock();
  }
  private animate(): void {
    if (this.frame) {
      return;
    }
    this.frame = requestAnimationFrame(() => {
      this.frame = 0;
      let pending = false;
      for (const [id, track] of this.tracks) {
        if (!track.duration) {
          continue;
        }
        const fraction = Math.min(1, (performance.now() - track.started) / track.duration);
        track.level = track.startLevel + (track.target - track.startLevel) * fraction;
        this.volume(track);
        if (fraction === 1) {
          track.duration = 0;
          if (track.removeAfterFade) {
            this.stop(id);
          }
        } else {
          pending = true;
        }
      }
      if (pending) {
        this.animate();
      }
    });
  }
}
export const audioManager = new AudioManager();
