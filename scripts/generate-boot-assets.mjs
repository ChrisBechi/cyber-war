// Original studio and menu audio. Opening media is captured by capture-opening.mjs.
import { mkdirSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
for (const directory of [
  'public/assets/audio',
  'public/assets/video',
  'public/assets/branding',
  'artifacts/boot-assets',
]) {
  mkdirSync(directory, { recursive: true });
}
const rate = 22050;
let seed = 8237;
const noise = () => {
  seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
  return seed / 2147483648 - 1;
};
const wave = (hz, time) => Math.sin(2 * Math.PI * hz * time);
const cues = {
  'studio-intro-ambience': [6, (t) => 0.15 * wave(55, t) + 0.055 * wave(110, t) + 0.025 * noise()],
  'keyboard-loop': [
    1,
    (t) => {
      const phase = t % 0.092;
      return phase < 0.018 ? 0.32 * noise() * Math.exp(-phase * 190) : 0;
    },
  ],
  'logo-reveal': [
    0.65,
    (t) => (0.24 * wave(170 + t * 410, t) + 0.055 * noise()) * Math.exp(-t * 7),
  ],
  'terminal-clear': [
    0.48,
    (t) =>
      (t < 0.025 ? 0.4 * noise() : 0.14 * noise() + 0.16 * wave(1150 - t * 1400, t)) *
      Math.exp(-t * 9),
  ],
  'menu-hover': [0.07, (t) => 0.18 * wave(820, t) * Math.exp(-t * 65)],
  'menu-select': [0.22, (t) => 0.25 * (wave(520, t) + 0.3 * wave(1040, t)) * Math.exp(-t * 18)],
  'menu-ambience': [
    8,
    (t) =>
      0.1 * wave(55, t) +
      0.045 * wave(82.5, t) +
      0.012 * noise() +
      0.025 * wave(220, t) * (0.5 + 0.5 * wave(0.125, t)),
  ],
};
for (const [name, [seconds, sample]] of Object.entries(cues)) {
  const frames = Math.round(seconds * rate);
  const buffer = Buffer.alloc(44 + frames * 2);
  buffer.write('RIFF');
  buffer.writeUInt32LE(buffer.length - 8, 4);
  buffer.write('WAVEfmt ', 8);
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20);
  buffer.writeUInt16LE(1, 22);
  buffer.writeUInt32LE(rate, 24);
  buffer.writeUInt32LE(rate * 2, 28);
  buffer.writeUInt16LE(2, 32);
  buffer.writeUInt16LE(16, 34);
  buffer.write('data', 36);
  buffer.writeUInt32LE(frames * 2, 40);
  for (let i = 0; i < frames; i++) {
    const edge = Math.min(1, i / 150, (frames - i) / 150);
    buffer.writeInt16LE(
      Math.round(Math.max(-0.95, Math.min(0.95, sample(i / rate))) * edge * 32767),
      44 + i * 2,
    );
  }
  const wav = `artifacts/boot-assets/${name}.wav`;
  writeFileSync(wav, buffer);
  execFileSync('ffmpeg', [
    '-y',
    '-hide_banner',
    '-loglevel',
    'error',
    '-i',
    wav,
    '-c:a',
    'libvorbis',
    '-q:a',
    '4',
    `public/assets/audio/${name}.ogg`,
  ]);
}
