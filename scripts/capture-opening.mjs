/* global document */
// Run the Vite development server, export the Rust fixture, then run this script.
// PLAYWRIGHT_MODULE may point at an existing local Playwright installation.
import { createRequire } from 'node:module';
import { mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { execFileSync } from 'node:child_process';
const require = createRequire(import.meta.url);
const { chromium } = require(
  process.env.PLAYWRIGHT_MODULE ??
    join(
      process.env.USERPROFILE,
      '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright',
    ),
);
const bookends = process.argv.includes('--bookends');
const montage = process.argv.includes('--montage');
const montageScenes = JSON.parse(
  readFileSync('src/features/opening/scenarios/montage-scenes.json', 'utf8'),
);
const duration = bookends
  ? 26.5
  : montage
    ? montageScenes.reduce((sum, scene) => sum + scene.duration, 0)
    : 90;
const width = bookends || montage ? 1600 : 1440;
const height = 900;
const folder = resolve(
  bookends
    ? 'artifacts/opening/bookends'
    : montage
      ? 'artifacts/opening/montage'
      : 'artifacts/opening',
);
mkdirSync(join(folder, 'frames'), { recursive: true });
const browser = await chromium.launch({
  executablePath:
    process.env.CHROME_PATH ?? 'C:/Program Files/Google/Chrome/Application/chrome.exe',
  headless: true,
  args: [
    '--autoplay-policy=no-user-gesture-required',
    '--disable-background-timer-throttling',
    '--disable-renderer-backgrounding',
  ],
});
try {
  const page = await browser.newPage({
    viewport: { width, height },
    deviceScaleFactor: 1,
  });
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') {
      const error = `${message.text()} ${message.location().url}`;
      errors.push(error);
      process.stdout.write(`${error}\n`);
    }
  });
  page.on('response', (response) => {
    if (response.status() >= 400) {
      errors.push(`${response.status()} ${response.url()}`);
    }
  });
  await page.clock.setFixedTime(new Date('2026-09-13T02:17:00Z'));
  await page.goto(
    `http://127.0.0.1:1420/?capture=1#opening/${bookends ? 'edit' : montage ? 'montage' : 'desktop'}`,
  );
  await page.locator('[data-opening-ready="true"]').waitFor();
  await page.evaluate(() => document.fonts.ready);
  const cdp = await page.context().newCDPSession(page);
  const frames = [];
  let start = 0;
  cdp.on('Page.screencastFrame', (event) => {
    void cdp.send('Page.screencastFrameAck', { sessionId: event.sessionId }).catch(() => {});
    if (!start) {
      return;
    }
    const time = Math.max(0, event.metadata.timestamp - start);
    if (time > duration) {
      return;
    }
    const path = `frames/${String(frames.length).padStart(5, '0')}.jpg`;
    writeFileSync(join(folder, path), Buffer.from(event.data, 'base64'));
    frames.push({ time, path });
  });
  await cdp.send('Page.startScreencast', {
    format: 'jpeg',
    quality: 96,
    maxWidth: width,
    maxHeight: height,
    everyNthFrame: 1,
  });
  start = Date.now() / 1000;
  await page.evaluate(() => {
    document.querySelector('.scenario-start').click();
  });
  const scenes = bookends
    ? [1.6, 2.8, 4, 5.8, 7.5, 10, 14, 18, 22, 26]
    : montage
      ? montageScenes.map(
          (scene, i) =>
            montageScenes.slice(0, i).reduce((sum, s) => sum + s.duration, 0) + scene.duration - 1,
        )
      : [5, 11, 17, 23, 28, 33, 39, 45, 51, 57, 62, 68, 73.8, 79, 84.8, 87, 89.4];
  const checks = [];
  for (const time of scenes) {
    await page.waitForFunction(
      (ms) => Number(document.querySelector('.opening-scenario').dataset.openingTime) >= ms,
      time * 1000,
      { timeout: 20000 },
    );
    const check = await page.evaluate(() => ({
      scene: document.querySelector('.opening-scenario').dataset.openingState,
      errors: Array.from(document.querySelectorAll('[role="alert"]')).map((el) => el.textContent),
      dialogs: Array.from(document.querySelectorAll('[role="dialog"][aria-hidden="false"]')).map(
        (el) => el.getAttribute('aria-label'),
      ),
    }));
    checks.push({ time, ...check });
    if (montage && check.scene !== montageScenes[checks.length - 1].id) {
      errors.push(`Wrong take at ${time}: ${check.scene}`);
    }
    await page.screenshot({ path: join(folder, `scene-${String(time).padStart(4, '0')}.png`) });
    process.stdout.write(`${time}s ${check.scene}\n`);
  }
  await page.locator('[data-opening-done="true"]').waitFor({ timeout: 15000 });
  await cdp.send('Page.stopScreencast');
  const manifest = JSON.parse(await page.locator('#opening-capture-manifest').textContent());
  Object.assign(manifest, { errors, checks, frames: frames.length });
  writeFileSync(join(folder, 'manifest.json'), JSON.stringify(manifest, null, 2));
  writeFileSync(join(folder, 'frames.json'), JSON.stringify(frames));
  if (manifest.failures.length || errors.length || checks.some((check) => check.errors.length)) {
    throw new Error(`Capture failed validation. See ${folder}/manifest.json`);
  }
  const first = frames[0];
  if (!first) {
    throw new Error('No captured frames');
  }
  first.time = 0;
  writeFileSync(
    join(folder, 'frames.ffconcat'),
    'ffconcat version 1.0\n' +
      frames
        .map(
          (frame, i) =>
            `file '${frame.path}'\nduration ${Math.max(0.001, (frames[i + 1]?.time ?? duration) - frame.time).toFixed(6)}\n`,
        )
        .join('') +
      `file '${frames.at(-1).path}'\n`,
  );
  execFileSync('ffmpeg', [
    '-y',
    '-hide_banner',
    '-loglevel',
    'error',
    '-safe',
    '0',
    '-i',
    join(folder, 'frames.ffconcat'),
    '-vf',
    'fps=30,scale=in_range=pc:out_range=tv:in_color_matrix=bt601:out_color_matrix=bt709,format=yuv420p',
    '-color_range',
    'tv',
    '-colorspace',
    'bt709',
    '-color_trc',
    'bt709',
    '-color_primaries',
    'bt709',
    '-t',
    String(duration),
    '-c:v',
    'libx264',
    '-preset',
    'fast',
    '-crf',
    '18',
    '-pix_fmt',
    'yuv420p',
    join(folder, 'gameplay-silent.mp4'),
  ]);
  process.stdout.write(`Captured ${frames.length} gameplay frames; all timeline actions passed.\n`);
} finally {
  await browser.close();
}
