// Render production React components and CSS to disposable layout-review pages.
// This is visual QA only: no JavaScript search engine or gameplay transport.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'vite';
import { createElement as h } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { JSDOM } from 'jsdom';

const vite = await createServer({ server: { middlewareMode: true }, appType: 'custom' });
try {
  const root = '/src/features/browser/sites/goggle/';
  const component = async (file) => (await vite.ssrLoadModule(`${root}${file}.tsx`))[file];
  const [
    GoggleSite,
    GoggleHeader,
    GoggleHome,
    GoggleFooter,
    GoggleAppsMenu,
    GoggleImages,
    SearchBox,
    SearchResult,
  ] = await Promise.all(
    [
      'GoggleSite',
      'GoggleHeader',
      'GoggleHome',
      'GoggleFooter',
      'GoggleAppsMenu',
      'GoggleImages',
      'SearchBox',
      'SearchResult',
    ].map(component),
  );
  const css = (
    await Promise.all(
      [
        'src/styles/global.css',
        'src/styles/kali.css',
        'src/features/desktop/browser-chrome.css',
        'src/features/browser/sites/goggle/goggle.css',
      ].map((path) => readFile(path, 'utf8')),
    )
  )
    .join('\n')
    .replace(/@import[^;]+;/g, '');
  const docs = JSON.parse(await readFile('content/search/documents.json', 'utf8'));
  const noop = () => {};
  const common = { navigate: noop, search: noop, imageSearch: noop, logout: noop, busy: false };
  const account = {
    id: 'visual-qa',
    displayName: 'Chris',
    email: 'chris@goggle.com',
    avatar: null,
    apps: ['images', 'account'],
  };
  const page = { title: 'Goggle', body: 'Encontre o que conecta o seu mundo.', action: null };
  const shell = (...children) => h('div', { className: 'goggle-site' }, ...children);
  const scenes = {
    home: h(GoggleSite, { address: 'https://www.goggle.com', page, navigate: noop }),
    authenticated: shell(
      h(GoggleHeader, { ...common, account, home: true }),
      h(GoggleHome, common),
      h(GoggleFooter, common),
    ),
    apps: shell(
      h(GoggleHeader, { ...common, account, home: true }),
      h(GoggleHome, common),
      h('div', { style: { position: 'absolute', top: 50, right: 20 } }, h(GoggleAppsMenu, common)),
      h(GoggleFooter, common),
    ),
    login: h(GoggleSite, { address: 'https://www.goggle.com/login', page, navigate: noop }),
    search: shell(
      h(
        GoggleHeader,
        { ...common, account, home: false },
        h(SearchBox, { ...common, initialQuery: 'Orion', compact: true }),
      ),
      h(
        'main',
        { className: 'goggle-results' },
        h(
          'nav',
          { className: 'goggle-result-tabs' },
          h('button', { 'aria-current': 'page' }, 'Todos'),
          h('button', {}, 'Imagens'),
        ),
        h('p', { className: 'goggle-results-status' }, 'Páginas da internet virtual'),
        h(
          'div',
          { className: 'goggle-result-list' },
          ...docs
            .filter((d) => ['orion', 'orion-campus'].includes(d.id))
            .map((document) => h(SearchResult, { key: document.id, document, navigate: noop })),
        ),
      ),
      h(GoggleFooter, common),
    ),
    images: shell(
      h(
        GoggleHeader,
        { ...common, account, home: false },
        h(SearchBox, { ...common, initialQuery: '', compact: true }),
      ),
      h(
        'main',
        { className: 'goggle-results' },
        h(
          'nav',
          { className: 'goggle-result-tabs' },
          h('button', {}, 'Todos'),
          h('button', { 'aria-current': 'page' }, 'Imagens'),
        ),
        h('p', { className: 'goggle-results-status' }, 'Imagens da internet virtual'),
        h(GoggleImages, {
          ...common,
          results: {
            documents: docs.filter((d) => d.type === 'IMAGE'),
            images: [
              { id: 'orion-campus', asset: 'orion-campus' },
              { id: 'archive-records', asset: 'archive-records' },
            ],
          },
        }),
      ),
      h(GoggleFooter, common),
    ),
  };
  await mkdir('artifacts/goggle-review', { recursive: true });
  const { Browser } = await vite.ssrLoadModule('/src/features/desktop/Browser.tsx');
  const browserMarkup = renderToStaticMarkup(h(Browser));
  const browserDocument = new JSDOM(browserMarkup).window.document;
  const content = browserDocument.querySelector('.browser-zoom-page');
  if (!content) {
    throw new Error('Browser review insertion point changed');
  }
  for (const [name, scene] of Object.entries(scenes)) {
    const markup = renderToStaticMarkup(scene);
    await writeFile(
      `artifacts/goggle-review/${name}.html`,
      `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Goggle · revisão ${name}</title><style>${css}\nbody{height:100vh;overflow:auto}.review-host{display:flex;min-height:100vh;width:100%}.review-host>.goggle-site{min-height:100vh}</style></head><body><div class="review-host">${markup}</div></body></html>`,
    );
    content.innerHTML = markup;
    await writeFile(
      `artifacts/goggle-review/embedded-${name}.html`,
      `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Goggle no navegador · revisão ${name}</title><style>${css}\nbody{height:100vh;overflow:hidden}.review-host{height:100vh;width:100%}.review-host>.browser-shell{width:100%}</style></head><body><div class="review-host">${browserDocument.body.innerHTML}</div></body></html>`,
    );
  }
  process.stdout.write(
    'Revisões visuais: artifacts/goggle-review/{home,authenticated,apps,login,search,images}.html\n',
  );
} finally {
  await vite.close();
}
