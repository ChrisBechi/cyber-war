import { z } from 'zod';
import { virtualUrl } from './search-schema.mjs';
import { validatePrice } from './web-economy.mjs';
const text = z.string().min(1).max(10000);
const stock = z.enum(['IN_STOCK', 'LOW_STOCK', 'OUT_OF_STOCK', 'DISCONTINUED', 'PREORDER']);
const price = z.int().min(1).max(1_000_000_000);
const link = z.strictObject({ label: text, url: virtualUrl });
const block = z.discriminatedUnion('kind', [
  z.strictObject({ kind: z.literal('paragraph'), text }),
  z.strictObject({ kind: z.literal('heading'), text }),
  z.strictObject({ kind: z.literal('quote'), text, attribution: text }),
  z.strictObject({ kind: z.literal('code'), text, language: text }),
  z.strictObject({ kind: z.literal('list'), items: z.array(text), ordered: z.boolean() }),
  z.strictObject({
    kind: z.literal('table'),
    headings: z.array(text),
    rows: z.array(z.array(text)),
  }),
  z.strictObject({ kind: z.literal('link'), label: text, url: virtualUrl }),
]);
const detail = z.discriminatedUnion('kind', [
  z.strictObject({
    kind: z.literal('socialProfile'),
    bio: text,
    location: text,
    role: text,
    employer: link,
    following: z.array(text),
  }),
  z.strictObject({ kind: z.literal('socialPost'), profileId: text, likes: z.int().nonnegative() }),
  z.strictObject({
    kind: z.literal('listing'),
    priceCents: price,
    minimumCents: price,
    condition: z.enum(['novo', 'usado']),
    location: text,
    seller: link,
    status: z.enum(['AVAILABLE', 'RESERVED', 'SOLD']),
  }),
  z.strictObject({ kind: z.literal('article'), readingMinutes: z.int().positive() }),
  z.strictObject({
    kind: z.literal('product'),
    priceCents: price,
    stock,
    seller: link,
    specifications: z.array(z.array(text)),
  }),
  z.strictObject({
    kind: z.literal('recipe'),
    minutes: z.int().positive(),
    servings: z.int().positive(),
    difficulty: text,
    ingredients: z.array(text).min(1),
    steps: z.array(text).min(1),
  }),
  z.strictObject({
    kind: z.literal('course'),
    minutes: z.int().positive(),
    level: text,
    teacher: link,
    lessons: z.array(link).min(1),
  }),
  z.strictObject({
    kind: z.literal('lesson'),
    course: link,
    position: z.int().positive(),
    next: link.nullable(),
  }),
  z.strictObject({
    kind: z.literal('thread'),
    community: text,
    votes: z.int().nonnegative(),
    locked: z.boolean(),
  }),
  z.strictObject({
    kind: z.literal('video'),
    seconds: z.int().positive(),
    views: z.int().nonnegative(),
    channel: link,
    chapters: z.array(text),
  }),
  z.strictObject({ kind: z.literal('profile'), bio: text, location: text }),
  z.strictObject({ kind: z.literal('reference'), facts: z.array(z.array(text)) }),
]);
export const webPackSchema = z.strictObject({
  ads: z
    .array(
      z.strictObject({
        id: text,
        advertiser: text,
        documentId: text,
        title: text,
        description: text,
        keywords: z.array(text).min(1),
        targeting: z
          .array(
            z.enum([
              'SEARCH',
              'EDITORIAL',
              'BLOG',
              'SOCIAL',
              'VIDEO',
              'COMMERCE',
              'EDUCATION',
              'FORUM',
              'CLASSIFIEDS',
              'CORPORATE',
              'RECIPE',
              'ENCYCLOPEDIA',
              'ARCHIVE',
            ]),
          )
          .optional(),
        quality: z.int().min(0).max(100).optional(),
        fraudRisk: z.int().min(0).max(100).optional(),
        budgetCents: price,
        clickCostCents: price,
        startsAt: z.int().nonnegative(),
        endsAt: z.int().positive(),
      }),
    )
    .default([]),
  id: text,
  version: z.int().positive(),
  seed: z.int().nonnegative(),
  brands: z.array(
    z.strictObject({
      id: text,
      name: text,
      domain: text,
      depth: z.enum(['FULL', 'STANDARD', 'LONG_TAIL']).optional(),
      identity: z
        .strictObject({
          typography: z.enum(['sans', 'serif', 'mono']),
          header: z.enum(['compact', 'masthead', 'band']),
          cards: z.enum(['grid', 'rows', 'ledger']),
          density: z.enum(['airy', 'dense']),
          surface: z.enum(['paper', 'plain', 'tinted', 'lined']),
        })
        .optional(),
      platform: z.enum([
        'EDITORIAL',
        'FORUM',
        'COMMERCE',
        'ENCYCLOPEDIA',
        'RECIPE',
        'EDUCATION',
        'CORPORATE',
        'VIDEO',
        'BLOG',
        'SOCIAL',
        'CLASSIFIEDS',
        'ARCHIVE',
      ]),
      layout: z.enum([
        'tech',
        'community',
        'shop',
        'reference',
        'kitchen',
        'learning',
        'corporate',
        'video',
        'personal',
        'newspaper',
        'social',
        'classifieds',
        'archive',
      ]),
      accent: z.string().regex(/^#[0-9a-f]{6}$/),
      mark: text,
      tagline: text,
      voice: text,
      navigation: z.array(link),
    }),
  ),
  domains: z.array(z.strictObject({ host: text, brandId: text, redirect: text.nullable() })),
  entities: z.array(
    z.strictObject({
      id: text,
      name: text,
      kind: text,
      aliases: z.array(text),
      description: text,
      relations: z.array(z.strictObject({ kind: text, target: text })),
    }),
  ),
  documents: z.array(
    z.strictObject({
      id: text,
      brandId: text,
      path: z.string().startsWith('/'),
      title: text,
      summary: text,
      category: text,
      author: text,
      publishedAt: z.int().nonnegative(),
      updatedAt: z.int().nonnegative().optional(),
      entities: z.array(text),
      keywords: z.array(text),
      blocks: z.array(block),
      detail,
      comments: z.array(
        z.strictObject({
          id: text,
          author: text,
          text,
          publishedAt: z.int(),
          rating: z.int().min(1).max(5).nullable(),
        }),
      ),
      links: z.array(link),
      visual: z.enum([
        'book',
        'code',
        'chip',
        'cake',
        'phone',
        'router',
        'film',
        'car',
        'paw',
        'pizza',
        'office',
        'chair',
        'radio',
        'house',
        'cup',
        'cable',
        'duck',
        'bowl',
      ]),
      requiredFlags: z.array(text),
      materializer: z.literal('product').optional(),
    }),
  ),
  events: z.array(
    z.strictObject({
      id: text,
      requiredFlag: text.nullable(),
      afterSeconds: z.int().nonnegative().default(0),
      publish: z.array(text),
      remove: z.array(text),
      offers: z
        .array(
          z.strictObject({
            documentId: text,
            priceCents: price.nullable(),
            stock: stock.nullable(),
          }),
        )
        .default([]),
    }),
  ),
});
export function validateWebPack(input) {
  const pack = webPackSchema.parse(input);
  const unique = (values, label) => {
    if (new Set(values).size !== values.length) throw new Error(`Duplicate ${label}`);
  };
  for (const field of ['brands', 'entities', 'documents', 'events'])
    unique(
      pack[field].map((x) => x.id),
      `${field} ID`,
    );
  unique(
    pack.domains.map((d) => d.host),
    'domain',
  );
  const brands = new Map(pack.brands.map((b) => [b.id, b]));
  const entities = new Set(pack.entities.map((e) => e.id));
  const docs = new Map(pack.documents.map((d) => [d.id, d]));
  unique(
    pack.ads.map((c) => c.id),
    'campaign ID',
  );
  for (const c of pack.ads) {
    if (!docs.has(c.documentId) || c.endsAt <= c.startsAt || c.clickCostCents > c.budgetCents)
      throw new Error(`Invalid ad campaign ${c.id}`);
  }
  const publishers = new Map();
  for (const event of pack.events) {
    unique(event.publish, `publication in ${event.id}`);
    unique(event.remove, `removal in ${event.id}`);
    unique(
      event.offers.map((o) => o.documentId),
      `offer in ${event.id}`,
    );
    for (const id of event.publish) {
      if (publishers.has(id)) throw new Error(`Multiple publication events for ${id}`);
      publishers.set(id, event);
      if (event.remove.includes(id)) throw new Error(`Event publishes and removes ${id}`);
    }
  }
  const key = (url) => {
    const parsed = new URL(url);
    return parsed.hostname.replace(/^www\./, '') + parsed.pathname.replace(/\/$/, '');
  };
  const addresses = pack.documents.map(
    (d) => `${brands.get(d.brandId)?.domain}${d.path.replace(/\/$/, '')}`,
  );
  unique(addresses, 'URL');
  const routes = new Map(addresses.map((address, i) => [address, pack.documents[i]]));
  const titles = new Set();
  const summaries = new Set();
  let links = 0;
  const checkLink = (l, source) => {
    links++;
    const target = routes.get(key(l.url));
    if (!target) throw new Error(`Broken URL ${l.url} from ${source?.id ?? 'brand'}`);
    if (source && target.requiredFlags.some((f) => !source.requiredFlags.includes(f)))
      throw new Error(`Spoiler link from ${source.id} to ${target.id}`);
    if (source && target.publishedAt > source.publishedAt)
      throw new Error(`Future link from ${source.id} to ${target.id}`);
    const targetEvent = publishers.get(target.id);
    const sourceEvent = source && publishers.get(source.id);
    if (
      targetEvent &&
      (!sourceEvent ||
        (targetEvent.requiredFlag && targetEvent.requiredFlag !== sourceEvent.requiredFlag) ||
        sourceEvent.afterSeconds < targetEvent.afterSeconds)
    )
      throw new Error(`Unpublished event link to ${target.id}`);
  };
  for (const b of pack.brands)
    for (const l of b.navigation) {
      checkLink(l);
      const category = new URL(l.url).searchParams.get('category');
      if (
        category &&
        !pack.documents.some(
          (d) => d.brandId === b.id && d.category === category && !d.requiredFlags.length,
        )
      )
        throw new Error(`Empty navigation category ${b.id}: ${category}`);
    }
  for (const domain of pack.domains) {
    if (!brands.has(domain.brandId)) throw new Error(`Missing brand ${domain.host}`);
    if (domain.redirect && !pack.domains.some((d) => d.host === domain.redirect && !d.redirect))
      throw new Error(`Invalid redirect ${domain.host}`);
  }
  for (const e of pack.entities)
    for (const r of e.relations)
      if (!entities.has(r.target)) throw new Error(`Missing entity relation ${r.target}`);
  for (const d of pack.documents) {
    if (d.detail.kind === 'product' || d.detail.kind === 'listing')
      validatePrice(d, d.detail.priceCents);
    if (d.materializer && (d.detail.kind !== 'product' || d.blocks.length))
      throw new Error(`Invalid materialization descriptor ${d.id}`);
    if (!brands.has(d.brandId)) throw new Error(`Missing brand ${d.id}`);
    if (d.updatedAt !== undefined && d.updatedAt < d.publishedAt)
      throw new Error(`Update predates document ${d.id}`);
    for (const e of d.entities) if (!entities.has(e)) throw new Error(`Missing entity ${e}`);
    if (d.path !== '/') {
      const title = `${d.brandId}:${d.title}`;
      if (titles.has(title)) throw new Error(`Duplicate title ${title}`);
      titles.add(title);
      const summary = `${d.brandId}:${d.summary}`;
      if (summaries.has(summary)) throw new Error(`Duplicate snippet ${d.id}`);
      summaries.add(summary);
    }
    for (const c of d.comments)
      if (c.publishedAt < d.publishedAt) throw new Error(`Comment predates document ${d.id}`);
    for (const l of [...d.links, ...d.blocks.filter((b) => b.kind === 'link')]) checkLink(l, d);
    if (d.detail.kind === 'socialProfile') {
      for (const id of d.detail.following) {
        const target = docs.get(id);
        if (id === d.id || target?.detail.kind !== 'socialProfile')
          throw new Error(`Invalid social connection ${d.id} -> ${id}`);
        checkLink({ url: `https://www.${brands.get(target.brandId).domain}${target.path}` }, d);
      }
    }
    if (d.detail.kind === 'socialPost') {
      const profile = docs.get(d.detail.profileId);
      if (profile?.detail.kind !== 'socialProfile')
        throw new Error(`Missing social profile ${d.id}`);
      checkLink({ url: `https://www.${brands.get(profile.brandId).domain}${profile.path}` }, d);
    }
    if (d.detail.kind === 'listing' && d.detail.minimumCents > d.detail.priceCents)
      throw new Error(`Invalid minimum price ${d.id}`);
    for (const name of ['seller', 'teacher', 'course', 'channel', 'next', 'employer'])
      if (d.detail[name]) checkLink(d.detail[name], d);
    for (const l of d.detail.lessons ?? []) checkLink(l, d);
  }
  for (const e of pack.events) {
    for (const id of [...e.publish, ...e.remove])
      if (!docs.has(id)) throw new Error(`Event document missing ${id}`);
    for (const id of e.publish)
      if (e.requiredFlag && !docs.get(id).requiredFlags.includes(e.requiredFlag))
        throw new Error(`Event leaks ${id}`);
    for (const offer of e.offers) {
      const doc = docs.get(offer.documentId);
      if (!doc || doc.detail.kind !== 'product')
        throw new Error(`Event offer is not a product: ${offer.documentId}`);
      if (offer.priceCents === null && offer.stock === null)
        throw new Error(`Empty offer in ${e.id}`);
      if (offer.priceCents !== null) validatePrice(doc, offer.priceCents);
      if (doc.requiredFlags.some((flag) => flag !== e.requiredFlag))
        throw new Error(`Event offer precedes access to ${doc.id}`);
    }
  }
  return {
    pack: `${pack.id}@${pack.version}`,
    brands: pack.brands.length,
    documents: pack.documents.length,
    entities: entities.size,
    checkedLinks: links,
    bytes: Buffer.byteLength(JSON.stringify(pack)),
    byBrand: Object.fromEntries(
      pack.brands.map((b) => [b.name, pack.documents.filter((d) => d.brandId === b.id).length]),
    ),
  };
}
