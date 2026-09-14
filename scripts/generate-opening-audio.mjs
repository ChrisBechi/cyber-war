// Original, deterministic score and the same interface samples used by gameplay.
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { openingEdit, openingDuration } from './opening-edit-plan.mjs';
import { tensionScore, tensionBpm } from './opening-tension-score.mjs';
mkdirSync('artifacts/opening', { recursive: true });
mkdirSync('public/assets/audio', { recursive: true });
const rate = 44100;
let seed = 840291;
const noise = () => {
  seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
  return seed / 2147483648 - 1;
};
const sine = (hz, t) => Math.sin(2 * Math.PI * hz * t);
const samples = new Map();
const cues = {
  'edit-impact': [0.36, (t) => (sine(52, t) * 0.3 + noise() * 0.12) * Math.exp(-t * 17)],
  'edit-sweep': [
    0.18,
    (t) =>
      noise() * Math.sin((Math.PI * t) / 0.18) ** 2 * 0.13 +
      sine(440 - t * 1600, t) * Math.exp(-t * 24) * 0.025,
  ],
  'ui-click': [
    0.06,
    (t) => noise() * 0.21 * Math.exp(-t * 90) + sine(1300, t) * 0.07 * Math.exp(-t * 120),
  ],
  'ui-key': [0.045, (t) => (noise() * 0.18 + sine(750, t) * 0.06) * Math.exp(-t * 155)],
  'ui-terminal': [0.13, (t) => sine(440, t) * 0.1 * Math.exp(-t * 32)],
  'ui-window': [0.2, (t) => (sine(185 + 800 * t, t) * 0.07 + noise() * 0.03) * Math.exp(-t * 20)],
  'ui-message': [0.3, (t) => (sine(660, t) + 0.3 * sine(990, t)) * 0.12 * Math.exp(-t * 18)],
  'ui-notification': [
    0.65,
    (t) => (sine(t < 0.14 ? 660 : 880, t) + sine(1320, t) * 0.25) * 0.18 * Math.exp(-t * 9),
  ],
  'ui-download': [
    0.4,
    (t) => sine(t < 0.12 ? 520 : t < 0.24 ? 660 : 880, t) * 0.14 * Math.exp(-t * 8),
  ],
  'ui-alert': [0.65, (t) => (sine(164.8, t) + sine(174.6, t) * 0.4) * 0.18 * Math.exp(-t * 5)],
};
function wav(path, array, channels = 1) {
  const buffer = Buffer.alloc(44 + array.length * 2);
  buffer.write('RIFF');
  buffer.writeUInt32LE(buffer.length - 8, 4);
  buffer.write('WAVEfmt ', 8);
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20);
  buffer.writeUInt16LE(channels, 22);
  buffer.writeUInt32LE(rate, 24);
  buffer.writeUInt32LE(rate * 2 * channels, 28);
  buffer.writeUInt16LE(2 * channels, 32);
  buffer.writeUInt16LE(16, 34);
  buffer.write('data', 36);
  buffer.writeUInt32LE(array.length * 2, 40);
  array.forEach((value, i) =>
    buffer.writeInt16LE(Math.round(Math.max(-0.95, Math.min(0.95, value)) * 32767), 44 + i * 2),
  );
  writeFileSync(path, buffer);
}
const ffmpeg = (args) =>
  execFileSync('ffmpeg', ['-y', '-hide_banner', '-loglevel', 'error', ...args]);
for (const [name, [seconds, sample]] of Object.entries(cues)) {
  const array = Float32Array.from(
    { length: Math.round(seconds * rate) },
    (_, i) => sample(i / rate) * Math.min(1, i / 70),
  );
  samples.set(name, array);
  const path = `artifacts/opening/${name}.wav`;
  wav(path, array);
  ffmpeg(['-i', path, '-c:a', 'libvorbis', '-q:a', '5', `public/assets/audio/${name}.ogg`]);
}
const totalFrames = Math.round(rate * openingDuration);
const score = tensionScore(rate, openingDuration, openingEdit);
wav('artifacts/opening/score.wav', score, 2);
ffmpeg([
  '-i',
  'artifacts/opening/score.wav',
  '-af',
  'highpass=f=28,lowpass=f=15000,loudnorm=I=-13:TP=-4.5:LRA=7',
  '-ar',
  '44100',
  '-c:a',
  'pcm_s24le',
  'artifacts/opening/master.wav',
]);
ffmpeg([
  '-i',
  'artifacts/opening/master.wav',
  '-c:a',
  'libvorbis',
  '-q:a',
  '6',
  'public/assets/audio/cyber-war-opening-music.ogg',
]);
const manifest = 'artifacts/opening/edit-manifest.json';
if (existsSync(manifest)) {
  const effects = new Float32Array(totalFrames);
  for (const event of JSON.parse(readFileSync(manifest, 'utf8')).audio) {
    const sample = samples.get(event.cue);
    if (!sample) {
      continue;
    }
    const offset = Math.round((event.at * rate) / 1000);
    sample.forEach((value, i) => {
      if (offset + i < effects.length) {
        effects[offset + i] += value;
      }
    });
  }
  wav('artifacts/opening/effects.wav', effects);
  ffmpeg([
    '-i',
    'artifacts/opening/effects.wav',
    '-c:a',
    'libvorbis',
    '-q:a',
    '5',
    'public/assets/audio/cyber-war-opening-sfx.ogg',
  ]);
}
process.stdout.write(
  `Generated ${tensionBpm} BPM tension score, ${openingDuration}s, and synchronized transition effects.\n`,
);
