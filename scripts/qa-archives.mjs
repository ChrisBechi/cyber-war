// Native Tauri E2E. Start the debug app with CYBER_WAR_ARCHIVE_QA=1 and
// WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222.
// Node 22+; no mock transport, browser package, or host archive commands.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const out = new URL('../artifacts/archive-qa/', import.meta.url);
await fs.mkdir(out, { recursive: true });
let page;
for (let attempt = 0; attempt < 100 && !page; attempt++) {
  const pages = await fetch('http://127.0.0.1:9222/json/list')
    .then((response) => response.json())
    .catch(() => []);
  page = pages.find((p) => p.url.endsWith('#archive/qa'));
  if (!page) await new Promise((resolve) => setTimeout(resolve, 200));
}
assert(page, 'Start the isolated native QA session first');
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  ws.onopen = resolve;
  ws.onerror = reject;
});
let serial = 0;
const waiting = new Map();
ws.onmessage = (e) => {
  const r = JSON.parse(e.data);
  waiting.get(r.id)?.(r);
  waiting.delete(r.id);
};
const call = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = ++serial;
    const timer = setTimeout(() => reject(new Error(`Timeout: ${method}`)), 20000);
    waiting.set(id, (r) => {
      clearTimeout(timer);
      r.error ? reject(new Error(JSON.stringify(r.error))) : resolve(r.result);
    });
    ws.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expression) => {
  const r = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails)
    throw new Error(
      r.exceptionDetails.exception?.description ?? JSON.stringify(r.exceptionDetails),
    );
  return r.result?.value;
};
const invoke = (name, args = {}) =>
  evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(name)},${JSON.stringify(args)})`);
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const until = async (expression) => {
  for (let n = 0; n < 100; n++) {
    if (await evaluate(expression)) return;
    await pause(100);
  }
  throw new Error(`Condition not reached: ${expression}`);
};
const click = (text, selector = 'button', double = false) =>
  evaluate(`(() => {
  const e = [...document.querySelectorAll(${JSON.stringify(selector)})].find(e => e.textContent.trim() === ${JSON.stringify(text)});
  if (!e) throw new Error('Missing button: ' + ${JSON.stringify(text)});
  e.dispatchEvent(new MouseEvent(${JSON.stringify(double ? 'dblclick' : 'click')},{bubbles:true}));
})()`);
const fill = (selector, text) =>
  evaluate(`(() => {
  const e = document.querySelector(${JSON.stringify(selector)}); if (!e) throw new Error('Missing input');
  const proto = e.tagName === 'SELECT' ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(proto, 'value').set.call(e,${JSON.stringify(text)});
  e.dispatchEvent(new Event('input',{bubbles:true})); e.dispatchEvent(new Event('change',{bubbles:true}));
})()`);
const shot = async (name) => {
  await pause(150);
  const r = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
  await fs.writeFile(new URL(`${name}.png`, out), Buffer.from(r.data, 'base64'));
};
const matrix = async (name) => {
  await evaluate(
    `document.querySelectorAll('.window').forEach(e => e.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true })))`,
  );
  await pause(300);
  for (const [width, height] of [
    [1024, 640],
    [1280, 800],
    [1440, 900],
    [1920, 1080],
  ]) {
    await call('Emulation.setDeviceMetricsOverride', {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await shot(`${name}-${width}x${height}`);
  }
  await call('Emulation.setDeviceMetricsOverride', {
    width: 1280,
    height: 800,
    deviceScaleFactor: 1,
    mobile: false,
  });
};
const results = [];
const record = (name) => {
  results.push(name);
  console.log(`PASS ${name}`);
};
try {
  await until(`typeof window.__TAURI_INTERNALS__?.invoke === 'function'`);
  const world = await invoke('world_get');
  assert.equal(world.nickname, 'archive-qa', 'Never run against a player save');
  await until(
    `!![...document.querySelectorAll('.file-manager button')].find(e => e.textContent.endsWith('firmware.tar.gz'))`,
  );
  await matrix('file-manager');
  // File labels include SVG text; locate by the rendered filename span.
  await evaluate(`globalThis.qaOpen = (name) => {
    const e = [...document.querySelectorAll('.file-manager button')].find(e => e.textContent.endsWith(name));
    if (!e) throw new Error('Missing file '+name); e.dispatchEvent(new MouseEvent('dblclick',{bubbles:true}));
  }`);
  await evaluate(`qaOpen('firmware.tar.gz')`);
  await until(`!!document.querySelector('.archive-viewer table')`);
  assert.equal(
    Object.keys((await invoke('world_get')).vfs.nodes).length,
    Object.keys(world.vfs.nodes).length,
  );
  record('Opening archive reads index without extracting');
  await click('▸ firmware');
  for (const [width, height] of [
    [1024, 640],
    [1280, 800],
    [1440, 900],
    [1920, 1080],
  ]) {
    await call('Emulation.setDeviceMetricsOverride', {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await shot(`viewer-${width}x${height}`);
  }
  await call('Emulation.setDeviceMetricsOverride', {
    width: 1280,
    height: 800,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await click('Propriedades', '.archive-viewer summary');
  await matrix('properties');
  await click('Propriedades', '.archive-viewer summary');
  await click('Extrair…');
  await until(`!!document.querySelector('.archive-dialog')`);
  await shot('extract-dialog');
  await matrix('extract-dialog');
  await click('Extrair', '.archive-dialog button');
  await until(`!document.querySelector('.archive-dialog')`);
  assert.match(
    await invoke('vfs_read', { path: '/home/kali/Downloads/firmware/README', asRoot: false }),
    /AXR550/,
  );
  record('GUI extraction creates real readable VFS files');
  await evaluate(
    `document.querySelector('[aria-label="Fechar Archive Viewer"]').click();qaOpen('private.zip')`,
  );
  await until(`!!document.querySelector('.archive-viewer table')`);
  await click('Extrair…');
  await until(`!!document.querySelector('.archive-dialog input[type="password"]')`);
  await fill('.archive-dialog input[type="password"]', 'wrong-password');
  await shot('password-dialog');
  await matrix('password-dialog');
  await click('Extrair', '.archive-dialog button');
  await until(`!!document.querySelector('.archive-dialog [role="alert"]')`);
  assert.match(
    await evaluate(`document.querySelector('.archive-dialog [role="alert"]').textContent`),
    /password/,
  );
  assert.equal(await evaluate(`document.querySelector('input[type="password"]').value`), '');
  await shot('wrong-password');
  await matrix('wrong-password');
  await fill('.archive-dialog input[type="password"]', 'qa-secret');
  await click('Extrair', '.archive-dialog button');
  await until(`!document.querySelector('.archive-dialog')`);
  assert.match(
    await invoke('vfs_read', { path: '/home/kali/Downloads/qa/notes.txt', asRoot: false }),
    /Archive QA/,
  );
  record('Wrong password is rejected and cleared; correct password extracts');
  await evaluate(
    `document.querySelector('[aria-label="Fechar Archive Viewer"]').click();qaOpen('broken.zip')`,
  );
  await until(`!!document.querySelector('.archive-viewer [role="alert"]')`);
  await shot('corrupt-archive');
  await matrix('corrupt-archive');
  record('Corrupted archive shows an error without writing');
  await evaluate(`document.querySelector('[aria-label="Fechar Archive Viewer"]').click()`);
  await click('kali', '.file-manager button');
  await until(
    `document.querySelector('.file-manager footer')?.textContent.endsWith('Computador / /home/kali')`,
  );
  await until(
    `!![...document.querySelectorAll('.file-manager button')].find(e => e.textContent === 'qa')`,
  );
  await click('qa', '.file-manager button');
  await click('Compactar…', '.file-manager button');
  await until(`!!document.querySelector('.archive-dialog')`);
  await fill('.archive-dialog input', 'cancelled-native');
  await fill('.archive-dialog select', 'TAR_GZIP');
  await shot('compress-dialog');
  await matrix('compress-dialog');
  await click('Criar', '.archive-dialog button');
  await until(
    `document.querySelector('.archive-dialog [role="status"]')?.textContent.includes('%')`,
  );
  await pause(800);
  await shot('progress');
  await matrix('progress');
  await click('Cancelar', '.archive-dialog button');
  await until(`!!document.querySelector('.archive-dialog [role="alert"]')`);
  assert.equal(
    (await invoke('world_get')).vfs.nodes['/home/kali/cancelled-native.tar.gz'],
    undefined,
  );
  record('Large native job progresses and cancellation leaves no output');
  await click('Cancelar', '.archive-dialog button');
  const run = async (input) => {
    const r = await invoke('execute_terminal', { command: input });
    assert.equal(r.exitCode, 0, `${input}: ${r.stderr}`);
    return r;
  };
  await run('gzip -c qa/notes.txt > qa/logs.gz');
  assert.equal((await run('zcat qa/logs.gz | grep admin')).stdout, 'admin connected\n');
  await run('tar -xzf Downloads/firmware.tar.gz -C /tmp');
  assert.equal(
    await invoke('vfs_read', { path: '/tmp/firmware/README', asRoot: false }),
    await invoke('vfs_read', { path: '/home/kali/Downloads/firmware/README', asRoot: false }),
  );
  const prompt = await run('unzip Downloads/private.zip -d /tmp/private');
  assert.equal(prompt.archivePrompt.secret, true);
  const answer = await invoke('archive_terminal_input', { value: 'qa-secret', cancel: false });
  assert.equal(answer.exitCode, 0, answer.stderr);
  record('Native terminal IPC: binary redirection, pipe, tar/GUI equivalence and hidden password');
  const messages = (await invoke('world_get')).messages;
  const mail = messages.find((m) => m.attachments?.length);
  await invoke('attachment_download', { messageId: mail.id, index: 0 });
  await run('unzip Downloads/documentos.zip -d /tmp/mail');
  record('Attachment download and terminal extraction share the VFS');
  await evaluate(`(async () => {
    const { useWindows } = await import('/src/lib/window-store.ts');
    useWindows.getState().open('browser', 'https://www.kikolouro.com.br/');
    useWindows.getState().update('browser', { maximized: true });
  })()`);
  await until(
    `document.querySelector('.browser-network-error h1')?.textContent === 'Servidor não encontrado'`,
  );
  assert.equal(
    await evaluate(`document.querySelector('.browser-network-error strong').textContent`),
    'www.kikolouro.com.br.',
  );
  await until(`document.querySelector('.browser-network-error img')?.naturalWidth > 0`);
  for (const [width, height] of [
    [1920, 1080],
    [1024, 640],
  ]) {
    await call('Emulation.setDeviceMetricsOverride', {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await shot(`browser-not-found-${width}x${height}`);
  }
  await evaluate(`(async () => {
    const { useWindows } = await import('/src/lib/window-store.ts');
    useWindows.getState().update('browser', { maximized: false, x: 20, y: 30, width: 560, height: 540 });
  })()`);
  await shot('browser-not-found-narrow');
  await click('Tentar novamente', '.browser-network-error button');
  await until(`!!document.querySelector('.browser-network-error')`);
  assert.equal(
    await evaluate(`document.querySelector('[aria-label="Endereço"]').value`),
    'https://www.kikolouro.com.br',
  );
  await click('Saiba mais...', '.browser-network-error button');
  assert.equal(
    await evaluate(`document.querySelector('.browser-network-error-details').hidden`),
    false,
  );
  assert.match(await evaluate('location.href'), /localhost:1420/);
  record(
    'Firefox missing-site page: local illustration, hostname, retry, help and responsive layout',
  );
  await fs.writeFile(
    new URL('results.json', out),
    JSON.stringify(
      { date: new Date().toISOString(), transport: 'real Tauri WebView2 IPC', results },
      null,
      2,
    ),
  );
} finally {
  await call('Emulation.clearDeviceMetricsOverride').catch(() => {});
  ws.close();
}
