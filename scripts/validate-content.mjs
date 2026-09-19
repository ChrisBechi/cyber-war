import { readFile, readdir } from 'node:fs/promises';
import { validateWebPack } from './web-schema.mjs';
import { z } from 'zod';
import { forumSchema, missionSchema, networkSchema } from './content-schema.mjs';
import { searchDocumentSchema } from './search-schema.mjs';

const json = async (path) => JSON.parse(await readFile(path, 'utf8'));
const webReport = validateWebPack(await json('content/web/web-core.json'));
process.stdout.write(
  `Virtual Web valid: ${webReport.brands} brands, ${webReport.documents} documents, ${webReport.checkedLinks} links.\n`,
);
const missions = [];
for (const session of await readdir('content/missions')) {
  for (const file of await readdir(`content/missions/${session}`)) {
    if (file.endsWith('.json')) {
      missions.push(
        ...z.array(missionSchema).parse(await json(`content/missions/${session}/${file}`)),
      );
    }
  }
}
const unique = (ids, label) => {
  if (new Set(ids).size !== ids.length) {
    throw new Error(`Duplicate ${label}`);
  }
};
const searchDocuments = z
  .array(searchDocumentSchema)
  .parse(await json('content/search/documents.json'));
unique(
  searchDocuments.map((document) => document.id),
  'search document',
);
unique(
  searchDocuments.map((document) => document.url),
  'search URL',
);
for (const document of searchDocuments) {
  for (const mission of [...document.requiredMissions, ...document.missionTags]) {
    if (!missions.some((item) => item.id === mission)) {
      throw new Error(`Missing mission gate: ${mission}`);
    }
  }
  if (document.imageId) {
    const svg = await readFile(`public/assets/goggle/${document.imageId}.svg`, 'utf8');
    if (/<script|<foreignObject|(?:href|src)=["'](?:https?:|\/\/)|\son\w+\s*=/i.test(svg)) {
      throw new Error(`Unsafe search image: ${document.imageId}`);
    }
  }
}
unique(
  missions.map((m) => m.id),
  'mission',
);
const network = networkSchema.parse(await json('content/networks/initial.json'));
for (const [address, host] of Object.entries(network.hosts)) {
  if (host.address !== address) {
    throw new Error(`Host address mismatch: ${address}`);
  }
  unique(
    host.services.map((s) => s.port),
    'service port',
  );
}
for (const address of Object.values(network.dns)) {
  if (!network.hosts[address]) {
    throw new Error(`DNS points to missing host: ${address}`);
  }
}
for (const mission of missions) {
  for (const stage of mission.stages) {
    unique(
      stage.choices.map((c) => c.id),
      'choice',
    );
  }
  for (const condition of [
    ...mission.requirements,
    ...mission.startTriggers,
    ...mission.stages.flatMap((s) => s.conditions),
  ]) {
    if (
      condition.kind === 'hostService' &&
      !network.hosts[condition.host]?.services.some((s) => s.port === condition.port)
    ) {
      throw new Error(`Missing service in ${mission.id}`);
    }
  }
}
const threads = forumSchema.parse(await json('content/forums/threads.json'));
const community = z
  .object({
    groups: z.array(z.object({ id: z.string(), name: z.string() })),
    categories: z.array(z.object({ id: z.string(), group: z.string(), readOnly: z.boolean() })),
    members: z.array(z.object({ name: z.string() })),
  })
  .parse(await json('content/forums/community.json'));
unique(
  threads.map((thread) => thread.id),
  'forum thread',
);
unique(
  community.categories.map((category) => category.id),
  'forum category',
);
unique(
  community.members.map((member) => member.name),
  'forum member',
);
for (const category of community.categories) {
  if (!community.groups.some((group) => group.id === category.group)) {
    throw new Error(`Missing forum group: ${category.group}`);
  }
}
for (const thread of threads) {
  if (!community.categories.some((category) => category.id === thread.category)) {
    throw new Error(`Missing forum category: ${thread.category}`);
  }
  for (const author of [thread.author, ...thread.replies.map((reply) => reply.author)]) {
    if (!community.members.some((member) => member.name === author)) {
      throw new Error(`Missing forum member: ${author}`);
    }
  }
}
const safeId = z.string().regex(/^[a-zA-Z0-9_.+-]+$/);
const software = z
  .object({
    roots: z.array(z.string()),
    packages: z.array(z.string()),
    favorites: z.array(safeId),
    categories: z.array(z.object({ id: safeId, parent: safeId.nullable(), icon: safeId })),
    entries: z.array(
      z.object({
        id: safeId,
        name: z.string().min(1),
        package: z.string(),
        icon: safeId,
        categories: z.array(safeId).min(1),
        source: z.url(),
        operation: z.enum(['host', 'web', 'file', 'wifi', 'system']),
        commands: z.array(z.string()),
      }),
    ),
  })
  .parse(await json('content/software/kali-default.json'));
if (software.roots.join(',') !== 'kali-linux-default,kali-desktop-xfce') {
  throw new Error('Only the default Kali/Xfce edition is allowed');
}
unique(
  software.entries.map((entry) => entry.id),
  'software entry',
);
unique(
  software.categories.map((category) => category.id),
  'software category',
);
const softwareIds = new Set(software.entries.map((entry) => entry.id));
const categoryMap = new Map(software.categories.map((category) => [category.id, category]));
for (const favorite of software.favorites) {
  if (!softwareIds.has(favorite)) {
    throw new Error(`Missing favorite ${favorite}`);
  }
}
for (const category of software.categories) {
  let parent = category.parent;
  const visited = new Set([category.id]);
  while (parent) {
    if (visited.has(parent) || !categoryMap.has(parent)) {
      throw new Error(`Invalid category tree at ${category.id}`);
    }
    visited.add(parent);
    parent = categoryMap.get(parent).parent;
  }
}
for (const entry of software.entries) {
  if (
    !software.packages.includes(entry.package) &&
    !['xfce4-settings', 'xfce4-taskmanager'].includes(entry.package)
  ) {
    throw new Error(`Optional package ${entry.package} in default catalog`);
  }
  for (const category of entry.categories) {
    if (!categoryMap.has(category)) {
      throw new Error(`Unknown category ${category}`);
    }
  }
}
for (const name of await readdir('public/assets/kali')) {
  if (!name.endsWith('.svg')) {
    continue;
  }
  const svg = await readFile(`public/assets/kali/${name}`, 'utf8');
  if (
    !svg.includes('<svg') ||
    /<script|<foreignObject|(?:href|src)=["'](?:https?:|\/\/)|\son\w+\s*=/i.test(svg)
  ) {
    throw new Error(`Invalid or externally linked icon ${name}`);
  }
}
for (const icon of new Set(
  [...software.entries, ...software.categories].map((entry) => entry.icon),
)) {
  await readFile(`public/assets/kali/${icon}.svg`);
}
unique(
  threads.map((t) => t.id),
  'forum thread',
);
process.stdout.write(
  `Content valid: ${missions.length} missions, ${threads.length} threads, ${Object.keys(network.hosts).length} hosts, ${software.entries.length} default Kali entries.\n`,
);
