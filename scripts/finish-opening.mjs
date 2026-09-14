// Publish the current edited cut to the game's local media assets, with separate audio buses.
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { openingDuration, openingRevision } from './opening-edit-plan.mjs';
const folder = 'artifacts/opening';
const audioOnly = process.argv.includes('--audio-only');
const edit = JSON.parse(readFileSync(`${folder}/edit-manifest.json`, 'utf8'));
const capture = JSON.parse(readFileSync(`${folder}/bookends/manifest.json`, 'utf8'));
if (audioOnly) {
  const previous = JSON.parse(readFileSync(`${folder}/media-verification.json`, 'utf8'));
  const webm = previous.find((asset) => asset.path.endsWith('.webm'));
  if (
    webm?.revision !== openingRevision ||
    webm.sha256 !== createHash('sha256').update(readFileSync(webm.path)).digest('hex')
  ) {
    throw new Error('Audio-only export requires the verified video from this revision.');
  }
}
if (
  edit.revision !== openingRevision ||
  Math.abs(edit.duration / 1000 - openingDuration) > 0.001 ||
  edit.failures.length ||
  capture.failures.length ||
  capture.errors.length ||
  !capture.frames ||
  capture.checks.some((check) => check.errors.length)
) {
  throw new Error('Fix capture or editing failures before publishing media.');
}
const ffmpeg = (args) =>
  execFileSync('ffmpeg', ['-y', '-hide_banner', '-loglevel', 'error', ...args], {
    stdio: 'inherit',
  });
execFileSync(process.execPath, ['scripts/generate-opening-audio.mjs'], { stdio: 'inherit' });
mkdirSync('public/assets/video', { recursive: true });
const base = 'public/assets/video/cyber-war-opening';
ffmpeg([
  '-i',
  `${folder}/edited-silent.mp4`,
  '-i',
  `${folder}/master.wav`,
  '-map',
  '0:v:0',
  '-map',
  '1:a:0',
  '-c:v',
  'copy',
  '-c:a',
  'aac',
  '-af',
  'volume=0.7071',
  '-b:a',
  '320k',
  '-t',
  String(openingDuration),
  '-movflags',
  '+faststart',
  `${base}.mp4`,
]);
ffmpeg([
  '-i',
  audioOnly ? `${base}.webm` : `${folder}/edited-silent.mp4`,
  '-i',
  `${folder}/master.wav`,
  '-map',
  '0:v:0',
  '-map',
  '1:a:0',
  '-af',
  `volume=0.7071,atrim=end=${openingDuration - 0.05}`,
  // Standard video range avoids the Windows VP9 decoder failure with full-range input.
  ...(audioOnly
    ? ['-c:v', 'copy']
    : [
        '-vf',
        'format=yuv420p',
        '-c:v',
        'libvpx-vp9',
        '-row-mt',
        '1',
        '-threads',
        '4',
        '-cpu-used',
        '4',
        '-b:v',
        '0',
        '-crf',
        '25',
      ]),
  '-color_range',
  'tv',
  '-colorspace',
  'bt709',
  '-color_trc',
  'bt709',
  '-color_primaries',
  'bt709',
  '-c:a',
  'libopus',
  '-b:a',
  '160k',
  '-t',
  String(openingDuration),
  `${folder}/remastered.webm`,
]);
copyFileSync(`${folder}/remastered.webm`, `${base}.webm`);
ffmpeg([
  '-i',
  `${folder}/edited-silent.mp4`,
  '-i',
  `${folder}/master.wav`,
  '-i',
  `${folder}/effects.wav`,
  '-filter_complex',
  '[1:a][2:a]amix=inputs=2:normalize=0,alimiter=limit=0.63:level=false,volume=0.7071[a]',
  '-map',
  '0:v:0',
  '-map',
  '[a]',
  '-c:v',
  'copy',
  '-c:a',
  'aac',
  '-b:a',
  '320k',
  '-t',
  String(openingDuration),
  '-movflags',
  '+faststart',
  `${folder}/cyber-war-opening-preview.mp4`,
]);
const assets = [`${base}.mp4`, `${base}.webm`, 'public/assets/audio/cyber-war-opening-sfx.ogg'];
const report = assets.map((path) => {
  const probe = JSON.parse(
    execFileSync('ffprobe', ['-v', 'error', '-show_format', '-show_streams', '-of', 'json', path], {
      encoding: 'utf8',
    }),
  );
  if (Math.abs(Number(probe.format.duration) - openingDuration) > 0.06) {
    throw new Error(`Media duration mismatch: ${path}`);
  }
  return {
    path,
    revision: openingRevision,
    sha256: createHash('sha256').update(readFileSync(path)).digest('hex'),
    probe,
  };
});
writeFileSync(`${folder}/media-verification.json`, JSON.stringify(report, null, 2));
const peaks = [...assets, `${folder}/cyber-war-opening-preview.mp4`].map((path) => {
  const result = spawnSync(
    'ffmpeg',
    [
      '-hide_banner',
      '-i',
      path,
      '-af',
      'astats=metadata=0:reset=0',
      '-vn',
      '-f',
      'null',
      process.platform === 'win32' ? 'NUL' : '/dev/null',
    ],
    { encoding: 'utf8' },
  );
  const matches = [...result.stderr.matchAll(/Peak level dB: ([-\d.]+)/g)];
  const peak = Number(matches.at(-1)?.[1]);
  if (result.status || !Number.isFinite(peak) || peak > -0.2)
    throw new Error(`Audio peak exceeds headroom: ${path}: ${peak} dBFS`);
  return { path, peakDbfs: peak };
});
writeFileSync(
  `${folder}/audio-peak-verification.json`,
  JSON.stringify({ revision: openingRevision, peaks }, null, 2),
);
process.stdout.write(
  `Opening revision ${openingRevision} verified: ${openingDuration}s, ${edit.shots.length} full-frame shots, stereo music and separate SFX.\n`,
);
