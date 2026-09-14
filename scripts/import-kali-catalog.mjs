// Run after downloading the pinned official references into artifacts/kali-reference.
// Reads metadata and SVG only; desktop Exec fields are never evaluated or imported.
import {
  readFileSync,
  readdirSync,
  writeFileSync,
  mkdirSync,
  copyFileSync,
  existsSync,
} from 'node:fs';
import { join } from 'node:path';
import yaml from 'js-yaml';

const reference = 'artifacts/kali-reference';
const menu = join(reference, 'menu');
const output = 'content/software';
const assets = 'public/assets/kali';
mkdirSync(output, { recursive: true });
mkdirSync(assets, { recursive: true });
const text = (file) => readFileSync(file, 'utf8').replace(/^\uFEFF/, '');
const decode = (value) =>
  value
    .replace(/&amp;/g, '&')
    .replace(/&#39;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/<[^>]*>/g, '')
    .trim();
const ini = (file) =>
  Object.fromEntries(
    text(file)
      .split(/\r?\n/)
      .filter((line) => /^[\w-]+=/.test(line))
      .map((line) => [line.slice(0, line.indexOf('=')), line.slice(line.indexOf('=') + 1)]),
  );
const slug = (value) =>
  value
    .toLowerCase()
    .replace(/&/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
const metaHtml = text(join(reference, 'metapackages.html'));
const metapackages = new Map();
for (const match of metaHtml.matchAll(/<h3 id=([^ >]+)[^>]*>([\s\S]*?)(?=<h3 |$)/g)) {
  const dependencies =
    match[2].match(/<details[^>]*id=dependencies[^>]*>([\s\S]*?)<\/details>/)?.[1] ?? '';
  metapackages.set(
    match[1],
    [...dependencies.matchAll(/<li>(.*?)<\/li>/g)].map((item) => decode(item[1]).split(' | ')[0]),
  );
}
const roots = ['kali-linux-default', 'kali-desktop-xfce'];
const packages = new Set();
function includePackage(name) {
  if (packages.has(name)) {
    return;
  }
  packages.add(name);
  for (const dependency of metapackages.get(name) ?? []) {
    includePackage(dependency);
  }
}
roots.forEach(includePackage);
if (packages.size < 200 || packages.has('kali-linux-everything')) {
  throw new Error('Invalid default package selection');
}

const projects = new Map();
for (const match of text(join(reference, 'all-tools.html')).matchAll(
  /<a href=https:\/\/www\.kali\.org\/tools\/([a-z0-9.+-]+)\/(?:#([^ >]+))?[^>]*>([\s\S]*?)<\/a>/g,
)) {
  if (['all-tools', 'kali-meta', 'top-100'].includes(match[1])) {
    continue;
  }
  const project = projects.get(match[1]) ?? { packages: new Set([match[1]]), commands: new Set() };
  const title = match[0].match(/title="([^"]+) (package|command)"/);
  if (title?.[2] === 'package') {
    project.packages.add(decode(title[1]));
  }
  if (title?.[2] === 'command') {
    project.commands.add(decode(title[1]));
  }
  for (const command of match[0].matchAll(/title="Includes (.*?) command"/g)) {
    project.commands.add(decode(command[1]));
  }
  projects.set(match[1], project);
}
const projectFor = (pkg) => [...projects].find(([, project]) => project.packages.has(pkg));
const usedIcons = new Set();
function icon(name, fallback = 'kali-menu') {
  for (const candidate of [name, fallback]) {
    for (const directory of [join(menu, 'icons'), join(reference, 'theme')]) {
      const path = join(directory, `${candidate}.svg`);
      if (existsSync(path)) {
        const svg = text(path);
        if (
          !svg.includes('<svg') ||
          /<script|<foreignObject|(?:href|src)=["'](?:https?:|\/\/)|\son\w+\s*=/i.test(svg)
        ) {
          continue;
        }
        copyFileSync(path, join(assets, `${candidate}.svg`));
        usedIcons.add(candidate);
        return candidate;
      }
    }
  }
  throw new Error(`No icon for ${name}`);
}
const categories = [];
function categoryTree(nodes, parent = null) {
  for (const node of nodes) {
    if (typeof node === 'string') {
      continue;
    }
    const sourceId = `kali-${slug(node.Category)}`;
    // Kali reuses Pass-the-Hash in more than one top-level category.
    const id = categories.some((category) => category.sourceId === sourceId)
      ? `${sourceId}--${parent}`
      : sourceId;
    const directory = join(menu, 'desktop-directories', `${sourceId}.directory`);
    const metadata = existsSync(directory) ? ini(directory) : {};
    categories.push({
      id,
      sourceId,
      name: node.Name.replace(/^•\s*/, ''),
      parent,
      icon: icon(metadata.Icon ?? sourceId, 'applications-other'),
    });
    categoryTree(node.Include ?? [], id);
  }
}
categoryTree(yaml.load(text(join(menu, 'categories.yaml'))));
function operation(cats) {
  if (cats.some((c) => /wireless|wifi|802-11/.test(c))) {
    return 'wifi';
  }
  if (cats.some((c) => /forensic|reverse-engineering|steganography|cryptography|file/.test(c))) {
    return 'file';
  }
  if (cats.some((c) => /web/.test(c))) {
    return 'web';
  }
  if (
    cats.some((c) =>
      /reconnaissance|discovery|initial-access|credential|lateral|network|exploitation/.test(c),
    )
  ) {
    return 'host';
  }
  return 'system';
}
const entries = new Map();
for (const file of readdirSync(join(menu, 'desktop-files')).filter((name) =>
  name.endsWith('.desktop'),
)) {
  const item = ini(join(menu, 'desktop-files', file));
  const pkg = item['X-Kali-Package'];
  if (!packages.has(pkg) || !item.Name) {
    continue;
  }
  const id = file.slice(0, -8);
  const project = projectFor(pkg);
  const cats = (item.Categories ?? '')
    .split(';')
    .flatMap((id) =>
      categories.filter((category) => category.sourceId === id).map((category) => category.id),
    );
  if (!cats.length) {
    cats.push('kali-usual-applications');
  }
  const resource =
    id === 'kali-www'
      ? 'https://www.kali.org/'
      : id === 'kali-tools'
        ? 'https://www.kali.org/tools/'
        : id === 'kali-docs'
          ? 'https://www.kali.org/docs/'
          : id === 'kali-bugs'
            ? 'https://bugs.kali.org/'
            : id === 'offsec-training'
              ? 'https://www.offsec.com/'
              : id === 'exploit-database'
                ? 'https://www.exploit-db.com/'
                : id === 'vulnhub'
                  ? 'https://www.vulnhub.com/'
                  : null;
  entries.set(id, {
    id,
    name: item.Name,
    description: item.Comment ?? '',
    package: pkg,
    categories: cats,
    icon: icon(item.Icon),
    commands: [...(project?.[1].commands ?? [])],
    source: project
      ? `https://www.kali.org/tools/${project[0]}/`
      : 'https://www.kali.org/tools/kali-meta/',
    builtin: id === 'root-terminal' ? 'root-terminal' : null,
    resource,
    operation: operation(cats),
  });
}
// Include default CLI packages that do not provide a desktop launcher.
for (const pkg of [...packages].sort()) {
  if (metapackages.has(pkg) || [...entries.values()].some((entry) => entry.package === pkg)) {
    continue;
  }
  const project = projectFor(pkg);
  if (!project) {
    continue;
  }
  const related = [...entries.values()].find(
    (entry) => entry.source === `https://www.kali.org/tools/${project[0]}/`,
  );
  const cats = related?.categories ?? ['kali-services-and-other-tools'];
  entries.set(pkg, {
    id: pkg,
    name: pkg,
    description: `Ferramenta do conjunto padrão do Kali · ${pkg}`,
    package: pkg,
    categories: cats,
    icon: icon(`kali-${pkg}`),
    commands: [...project[1].commands],
    source: `https://www.kali.org/tools/${project[0]}/`,
    builtin: null,
    resource: null,
    operation: operation(cats),
  });
}
for (const [id, name, pkg, builtin, glyph] of [
  ['terminal', 'Terminal Emulator', 'qterminal', 'terminal', 'utilities-terminal'],
  ['file-manager', 'File Manager', 'thunar', 'files', 'system-file-manager'],
  ['text-editor', 'Text Editor', 'mousepad', 'editor', 'accessories-text-editor'],
  ['web-browser', 'Web Browser', 'firefox-esr', 'browser', 'web-browser'],
  ['task-manager', 'Task Manager', 'xfce4-taskmanager', 'processes', 'applications-system'],
  ['settings-manager', 'Settings Manager', 'xfce4-settings', 'settings', 'preferences-system'],
  ['calculator', 'Calculator', 'mate-calc', 'calculator', 'accessories-calculator'],
  ['document-viewer', 'Document Viewer', 'atril', 'files', 'applications-accessories'],
  ['archive-manager', 'Archive Manager', 'engrampa', 'files', 'folder'],
  ['image-viewer', 'Image Viewer', 'ristretto', 'files', 'applications-accessories'],
]) {
  if (!packages.has(pkg) && !['xfce4-settings', 'xfce4-taskmanager'].includes(pkg)) {
    continue;
  }
  for (const [oldId, entry] of entries) {
    if (entry.package === pkg) {
      entries.delete(oldId);
    }
  }
  entries.set(id, {
    id,
    name,
    package: pkg,
    builtin,
    icon: icon(glyph),
    description: name,
    categories: ['kali-usual-applications'],
    commands: [],
    source: 'https://www.kali.org/tools/kali-meta/',
    operation: 'system',
    resource: null,
  });
}
const favorites = [
  'terminal',
  'root-terminal',
  'file-manager',
  'text-editor',
  'web-browser',
  'kali-www',
  'kali-tools',
  'kali-docs',
  'kali-bugs',
  'offsec-training',
  'exploit-database',
  'vulnhub',
];
for (const id of favorites) {
  if (!entries.has(id)) {
    throw new Error(`Missing favorite ${id}`);
  }
}
for (const name of [
  'kali-panel-menu',
  'user-desktop',
  'firefox-esr',
  'starred',
  'folder-favorites',
  'network-wired-symbolic',
  'audio-volume-high-symbolic',
  'audio-volume-muted-symbolic',
  'notification-symbolic',
  'battery-full-charged-symbolic',
  'system-lock-screen-symbolic',
  'system-log-out-symbolic',
  'document-open-recent',
  'applications-other',
  'preferences-system',
  'system-log-out',
  'system-lock-screen',
  'system-shutdown',
]) {
  icon(name);
}
const catalog = {
  source: 'https://www.kali.org/tools/kali-meta/',
  menuSource: 'https://gitlab.com/kalilinux/packages/kali-menu',
  menuCommit: text(join(reference, 'menu-commit.txt')).trim(),
  themeCommit: text(join(reference, 'theme-commit.txt')).trim(),
  edition: 'Kali default + Xfce',
  roots,
  packages: [...packages].sort(),
  favorites,
  categories,
  entries: [...entries.values()].sort((a, b) => a.name.localeCompare(b.name, 'en')),
};
writeFileSync(join(output, 'kali-default.json'), JSON.stringify(catalog, null, 2) + '\n');
copyFileSync(join(menu, 'debian/copyright'), join(assets, 'kali-menu-copyright.txt'));
copyFileSync(join(reference, 'theme-copyright.txt'), join(assets, 'kali-themes-copyright.txt'));
console.info(
  JSON.stringify({
    edition: catalog.edition,
    packages: packages.size,
    entries: entries.size,
    categories: categories.length,
    icons: usedIcons.size,
  }),
);
