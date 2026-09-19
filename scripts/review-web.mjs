// Render real Rust page DTOs with production components. Disposable visual QA only.
import { readFile, writeFile, mkdir, readdir } from 'node:fs/promises';
import { createServer } from 'vite';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
const directory = 'artifacts/web-review';
await mkdir(directory, { recursive: true });
const vite = await createServer({
  server: { middlewareMode: true, watch: null },
  appType: 'custom',
  logLevel: 'warn',
});
try {
  const { VirtualWebSite } = await vite.ssrLoadModule(
    '/src/features/browser/virtual-web/VirtualWebSite.tsx',
  );
  const css =
    (await readFile('src/features/browser/virtual-web/virtual-web.css', 'utf8')) +
    (await readFile('src/features/browser/virtual-web/community.css', 'utf8')) +
    (await readFile('src/features/browser/virtual-web/brand-identity.css', 'utf8'));
  const scenes = [
    'techbyte',
    'redditor',
    'shopnow',
    'wipedia',
    'cozinha',
    'educa',
    'nexora',
    'viewtube',
    'blog',
    'b1',
    'product',
    'recipe',
    'course',
    'thread',
    'article',
    '404',
    'offer-base',
    'offer-live',
    'offer-low',
    'offer-empty',
    'offer-restock',
    'social',
    'social-profile',
    'social-post',
    'classifieds',
    'classified',
    'archive',
    'capture',
    ...(await readdir(directory))
      .filter((name) => name.startsWith('local-') && name.endsWith('.json'))
      .map((name) => name.slice(0, -5)),
  ];
  for (const name of scenes) {
    const page = JSON.parse(await readFile(`${directory}/${name}.json`, 'utf8'));
    const html = renderToStaticMarkup(
      createElement(VirtualWebSite, { initialPage: page, navigate: () => {} }),
    );
    await writeFile(
      `${directory}/${name}.html`,
      `<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${page.brand.name} — revisão visual</title><style>html,body{margin:0}*{box-sizing:border-box}${css}</style><body>${html}</body></html>`,
    );
  }
  await writeFile(
    `${directory}/index.html`,
    `<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>Virtual Web — revisão visual</title><style>body{font:14px Arial;margin:0;background:#dedee3}header{padding:14px;background:#fff;display:flex;gap:14px;flex-wrap:wrap}select,button{padding:8px}iframe{display:block;border:0;background:#fff;margin:20px auto;max-width:100%}</style><header><label>Página <select id="scene">${scenes.map((s) => `<option>${s}</option>`).join('')}</select></label><label>Dimensão <select id="size">${['1024x640', '1280x720', '1366x768', '1920x1080', '2560x1440'].map((s) => `<option>${s}</option>`).join('')}</select></label><button id="apply">Mostrar página</button><span>Renderização dos dados reais do Rust · controles internos sem IPC nesta revisão</span></header><iframe title="Página em revisão" id="preview" src="techbyte.html" width="1024" height="640"></iframe><script>document.getElementById('apply').onclick=()=>{const frame=document.getElementById('preview');const size=document.getElementById('size').value.split('x');frame.width=size[0];frame.height=size[1];frame.src=document.getElementById('scene').value+'.html';};</script></html>`,
  );
  console.log(`Rendered ${scenes.length} real platform scenes in ${directory}`);
} finally {
  await vite.close();
}
