import { describe, expect, it } from 'vitest';
import { OPENING_DURATION, openingScenes, sceneAt, TimelinePlayer } from './OpeningTimeline';
import { useWindows, windowApp } from '../../lib/window-store';

describe('opening playback contract', () => {
  it('covers exactly 90 seconds with continuous shots of at least three seconds', () => {
    expect(openingScenes[0].at).toBe(0);
    openingScenes.forEach((scene, index) => {
      expect(scene.end - scene.at).toBeGreaterThanOrEqual(3000);
      expect(scene.end).toBeLessThanOrEqual(90000);
      expect(sceneAt(scene.at)).toBe(scene.id);
      if (index) {
        expect(scene.at).toBe(openingScenes[index - 1].end);
      }
    });
    expect(openingScenes.at(-1)?.end).toBe(OPENING_DURATION);
  });
  it('replays every due event once in order after a delayed frame, including simultaneous actions', () => {
    const played: string[] = [];
    const player = new TimelinePlayer([
      { at: 0, id: 'start', run: () => played.push('start') },
      { at: 6000, id: 'a', run: () => played.push('a') },
      { at: 6000, id: 'b', run: () => played.push('b') },
      { at: 85000, id: 'title', run: () => played.push('title') },
      { at: 90001, id: 'outside', run: () => played.push('outside') },
    ]);
    player.advance(0);
    player.advance(12000);
    player.advance(12000);
    player.advance(95000);
    expect(played).toEqual(['start', 'a', 'b', 'title']);
    expect(player.completed).toBe(4);
    expect(() => player.advance(100)).toThrow('Restart');
  });
  it('keeps three independently managed instances of the real terminal application', () => {
    const original = useWindows.getState();
    try {
      original.reset();
      for (const id of ['terminal', 'terminal:2', 'terminal:3'] as const) {
        expect(original.newTerminal()).toBe(id);
        expect(windowApp(id)).toBe('terminal');
      }
      original.update('terminal:2', { x: 600 });
      original.close('terminal');
      expect(useWindows.getState().windows.map((w) => w.id)).toEqual(['terminal:2', 'terminal:3']);
      expect(useWindows.getState().windows[0].x).toBe(600);
    } finally {
      useWindows.setState(original, true);
    }
  });
  it('records late takes when the source footage exceeds the final-film limit', () => {
    const played: string[] = [];
    const player = new TimelinePlayer(
      [
        { at: 90800, id: 'last-take', run: () => played.push('last-take') },
        { at: 96401, id: 'outside', run: () => played.push('outside') },
      ],
      96400,
    );
    player.advance(96400);
    expect(played).toEqual(['last-take']);
  });
});
