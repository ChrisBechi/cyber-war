export const OPENING_DURATION = 90_000;
export const openingScenes = [
  { at: 0, end: 6000, id: 'opening/desktop' },
  { at: 6000, end: 12000, id: 'opening/three-terminals' },
  { at: 12000, end: 18000, id: 'opening/work' },
  { at: 18000, end: 24000, id: 'opening/files' },
  { at: 24000, end: 29000, id: 'opening/vex' },
  { at: 29000, end: 34000, id: 'opening/null-forum' },
  { at: 34000, end: 40000, id: 'opening/no-ar' },
  { at: 40000, end: 46000, id: 'opening/em-claro' },
  { at: 46000, end: 52000, id: 'opening/operation' },
  { at: 52000, end: 58000, id: 'opening/escalation' },
  { at: 58000, end: 63000, id: 'opening/null-got-you' },
  { at: 63000, end: 69000, id: 'opening/chaos' },
  { at: 69000, end: 74000, id: 'opening/final-terminal' },
  { at: 74000, end: 80000, id: 'opening/final-operation' },
  { at: 80000, end: 85000, id: 'opening/flood' },
  { at: 85000, end: 90000, id: 'opening/title' },
] as const;
export type OpeningScene = (typeof openingScenes)[number]['id'];
export const sceneAt = (time: number): OpeningScene =>
  openingScenes.find((scene) => time >= scene.at && time < scene.end)?.id ??
  (time < 0 ? 'opening/desktop' : 'opening/title');
export type OpeningEvent = { at: number; id: string; run: () => void };
/** Drains every due action in stable order, even after a delayed animation frame. */
export class TimelinePlayer {
  private next = 0;
  private time = 0;
  constructor(
    private readonly events: readonly OpeningEvent[],
    private readonly duration = OPENING_DURATION,
  ) {}
  advance(time: number): void {
    if (time < this.time) {
      throw new Error('Restart the scenario before seeking backwards');
    }
    this.time = Math.min(time, this.duration);
    while (this.next < this.events.length && this.events[this.next].at <= this.time) {
      const event = this.events[this.next++];
      event.run();
    }
  }
  get completed(): number {
    return this.next;
  }
}
