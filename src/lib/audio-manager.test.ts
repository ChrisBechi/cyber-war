import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { AudioManager } from './audio-manager';

const play = vi.fn<() => Promise<void>>();
const pause = vi.fn<() => void>();
beforeEach(() => {
  play.mockReset().mockResolvedValue();
  pause.mockReset();
  vi.spyOn(HTMLMediaElement.prototype, 'play').mockImplementation(play);
  vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(pause);
});
afterEach(() => vi.restoreAllMocks());
it('multiplies master by the independent buses, including trailer audio and mute', () => {
  const manager = new AudioManager();
  const video = document.createElement('video');
  const sfx = document.createElement('audio');
  const voice = document.createElement('audio');
  const detach = manager.attach(video, 'music');
  manager.attach(sfx, 'sfx');
  manager.attach(voice, 'voice');
  manager.setVolumes({ master: 50, music: 60, sfx: 40, voice: 80 });
  expect(video.volume).toBeCloseTo(0.3);
  expect(sfx.volume).toBeCloseTo(0.2);
  expect(voice.volume).toBeCloseTo(0.4);
  manager.setVolumes({ master: 0, music: 100, sfx: 100, voice: 100 });
  expect(video.volume).toBe(0);
  expect(sfx.volume).toBe(0);
  expect(voice.volume).toBe(0);
  detach();
  manager.stopAll();
  expect(pause).toHaveBeenCalledTimes(3);
});
it('fades music in and out and does not restart a stopped loop', async () => {
  vi.useFakeTimers({ toFake: ['requestAnimationFrame', 'cancelAnimationFrame', 'performance'] });
  const media = document.createElement('audio');
  vi.stubGlobal('Audio', function () {
    return media;
  });
  const manager = new AudioManager();
  try {
    const id = manager.music('menu-ambience');
    expect(media.volume).toBe(0);
    await vi.advanceTimersByTimeAsync(720);
    expect(media.volume).toBeCloseTo(0.56);
    manager.stop(id, 500);
    await vi.advanceTimersByTimeAsync(256);
    expect(media.volume).toBeGreaterThan(0);
    expect(media.volume).toBeLessThan(0.56);
    await vi.advanceTimersByTimeAsync(300);
    expect(media.volume).toBe(0);
    const plays = play.mock.calls.length;
    manager.unlock();
    expect(play).toHaveBeenCalledTimes(plays);
  } finally {
    manager.stopAll();
    vi.useRealTimers();
    vi.unstubAllGlobals();
  }
});
