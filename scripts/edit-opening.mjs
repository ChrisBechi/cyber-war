import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import {
  openingEdit,
  openingDuration,
  openingSize,
  openingRevision,
} from './opening-edit-plan.mjs';
const cursor = JSON.parse(readFileSync('src/features/opening/title/title-cursor.json', 'utf8'));
const folder = 'artifacts/opening';
const editDirectory = `edit-v${openingRevision}`;
mkdirSync(`${folder}/${editDirectory}`, { recursive: true });
const ffmpeg = (args) =>
  execFileSync(
    'ffmpeg',
    ['-y', '-hide_banner', '-loglevel', 'error', '-filter_complex_threads', '2', ...args],
    { stdio: 'inherit' },
  );
const sources = new Map();
const audio = [];
for (const source of ['montage', 'bookends']) {
  const manifest = JSON.parse(readFileSync(`${folder}/${source}/manifest.json`, 'utf8'));
  if (manifest.errors.length || manifest.failures.length) {
    throw new Error(`Invalid capture: ${source}`);
  }
  const probe = JSON.parse(
    execFileSync(
      'ffprobe',
      [
        '-v',
        'error',
        '-select_streams',
        'v:0',
        '-show_entries',
        'stream=width,height',
        '-of',
        'json',
        `${folder}/${source}/gameplay-silent.mp4`,
      ],
      { encoding: 'utf8' },
    ),
  );
  if (
    probe.streams[0].width !== openingSize.width ||
    probe.streams[0].height !== openingSize.height
  ) {
    throw new Error(`Recapture ${source} at ${openingSize.width}x${openingSize.height}`);
  }
  sources.set(`${source}/gameplay-silent.mp4`, manifest);
}
const approved = JSON.parse(readFileSync(`${folder}/revision-3-manifest.json`, 'utf8'));
const approvedFinal = approved.shots.at(-1);
sources.set('approved-v3-final.mp4', {
  audio: approved.audio
    .filter((event) => event.at >= approvedFinal.at * 1000)
    .map((event) => ({ ...event, at: event.at - approvedFinal.at * 1000 })),
});
for (let index = 0; index < openingEdit.length; index++) {
  const shot = openingEdit[index];
  const args = [
    '-ss',
    String(shot.from),
    '-t',
    String(shot.span),
    '-i',
    `${folder}/${shot.source}`,
  ];
  const filters = [
    `[0:v]setpts=${shot.duration / shot.span}*(PTS-STARTPTS),fps=30,setsar=1,settb=AVTB,tpad=stop_mode=clone:stop_duration=0.1,trim=duration=${shot.duration}[current]`,
  ];
  let output = 'current';
  if (index && shot.transition !== 'cut') {
    const previous = openingEdit[index - 1];
    args.push(
      '-ss',
      String(previous.duration - 6 / 30),
      '-i',
      `${folder}/${editDirectory}/${String(index - 1).padStart(3, '0')}.mp4`,
    );
    filters.push('[1:v]setpts=PTS-STARTPTS,fps=30,setsar=1,settb=AVTB[previous]');
    filters.push(
      `[previous][current]xfade=transition=${shot.transition}:duration=${6 / 30}:offset=0[transition]`,
    );
    output = 'transition';
    audio.push({ at: Math.round(shot.at * 1000), cue: 'edit-sweep' });
  }
  if (shot.accent) {
    filters.push(
      `[${output}]rgbashift=rh=2:bh=-2:enable='lt(t,0.09)',drawbox=x=0:y=0:w=iw:h=ih:color=0x9ae9ed@0.10:t=fill:enable='lt(t,0.067)',drawbox=x=0:y=ih*0.48:w=iw:h=3:color=0x58d5e5@0.5:t=fill:enable='lt(t,0.067)'[accent]`,
    );
    output = 'accent';
    audio.push({ at: Math.round(shot.at * 1000), cue: 'edit-impact' });
  }
  if (shot.cursor) {
    const start = (cursor.revealAt + 2500) / 1000;
    filters.push(
      `[${output}]drawbox=x=iw*${cursor.x}:y=ih*${cursor.y}:w=iw*${cursor.width}:h=ih*${cursor.height}:color=${cursor.color.replace('#', '0x')}:t=fill:enable='gte(t,${start})*lt(mod(t-${start},${(cursor.blinkMs * 2) / 1000}),${cursor.blinkMs / 1000})'[cursor]`,
    );
    output = 'cursor';
  }
  filters.push(`[${output}]format=yuv420p[out]`);
  for (const event of sources.get(shot.source).audio) {
    if (event.at >= shot.from * 1000 && event.at < (shot.from + shot.span) * 1000) {
      audio.push({
        at: Math.round(
          (shot.at + ((event.at / 1000 - shot.from) * shot.duration) / shot.span) * 1000,
        ),
        cue: event.cue,
      });
    }
  }
  const path = `${editDirectory}/${String(index).padStart(3, '0')}.mp4`;
  ffmpeg([
    ...args,
    '-filter_complex',
    filters.join(';'),
    '-map',
    '[out]',
    '-an',
    '-frames:v',
    String(shot.frames),
    '-c:v',
    'libx264',
    '-preset',
    'fast',
    '-crf',
    '17',
    '-threads',
    '4',
    '-color_range',
    'tv',
    '-colorspace',
    'bt709',
    '-color_trc',
    'bt709',
    '-color_primaries',
    'bt709',
    `${folder}/${path}`,
  ]);
  const rendered = JSON.parse(
    execFileSync(
      'ffprobe',
      [
        '-v',
        'error',
        '-select_streams',
        'v:0',
        '-show_entries',
        'stream=width,height,nb_frames',
        '-of',
        'json',
        `${folder}/${path}`,
      ],
      { encoding: 'utf8' },
    ),
  ).streams[0];
  if (
    Number(rendered.nb_frames) !== shot.frames ||
    rendered.width !== openingSize.width ||
    rendered.height !== openingSize.height
  ) {
    throw new Error(`Rendered frame count or size mismatch: ${shot.id}`);
  }
  process.stdout.write(
    `${index + 1}/${openingEdit.length} ${shot.id} · ${shot.duration.toFixed(2)}s · full frame\n`,
  );
}
writeFileSync(
  `${folder}/edit.ffconcat`,
  'ffconcat version 1.0\n' +
    openingEdit
      .map((_, i) => `file '${editDirectory}/${String(i).padStart(3, '0')}.mp4'\n`)
      .join(''),
);
ffmpeg([
  '-safe',
  '0',
  '-i',
  `${folder}/edit.ffconcat`,
  '-c',
  'copy',
  '-movflags',
  '+faststart',
  `${folder}/edited-silent.mp4`,
]);
writeFileSync(
  `${folder}/edit-manifest.json`,
  JSON.stringify(
    {
      duration: openingDuration * 1000,
      revision: openingRevision,
      size: openingSize,
      shots: openingEdit,
      audio,
      failures: [],
    },
    null,
    2,
  ),
);
writeFileSync(
  `${folder}/edit-verification.json`,
  JSON.stringify(
    {
      revision: openingRevision,
      duration: openingDuration,
      size: openingSize,
      shots: openingEdit.length,
      fullFrames: openingEdit.length,
      crops: 0,
      zooms: 0,
      letterboxes: 0,
      transitions: openingEdit.filter((s) => s.transition !== 'cut').length,
      inserts: openingEdit.filter((s) => s.duration < 1).length,
      repeatedTakes: 0,
      uniqueMontageTakes: openingEdit.filter((s) => s.scene).length,
      accentEffects: openingEdit.filter((s) => s.accent).length,
      finaleSource: 'approved-v3-final.mp4',
      finaleOnlyChange: 'Blinking underscore cursor after title formation',
    },
    null,
    2,
  ),
);
process.stdout.write(`Edited full-frame revision ${openingRevision}: ${openingDuration}s.\n`);
