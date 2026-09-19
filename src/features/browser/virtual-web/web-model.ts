import { z } from 'zod';

const linkSchema = z.object({ label: z.string(), url: z.string() });
export const adSchema = z.object({
  id: z.string(),
  advertiser: z.string(),
  title: z.string(),
  description: z.string(),
  url: z.string(),
});
const table = z.array(z.array(z.string()));
export const blockSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('paragraph'), text: z.string() }),
  z.object({ kind: z.literal('heading'), text: z.string() }),
  z.object({ kind: z.literal('quote'), text: z.string(), attribution: z.string() }),
  z.object({ kind: z.literal('code'), text: z.string(), language: z.string() }),
  z.object({ kind: z.literal('list'), items: z.array(z.string()), ordered: z.boolean() }),
  z.object({ kind: z.literal('table'), headings: z.array(z.string()), rows: table }),
  z.object({ kind: z.literal('link'), label: z.string(), url: z.string() }),
]);
const detailSchema = z.discriminatedUnion('kind', [
  z.object({
    kind: z.literal('socialProfile'),
    bio: z.string(),
    location: z.string(),
    role: z.string(),
    employer: linkSchema,
    following: z.array(z.string()),
  }),
  z.object({ kind: z.literal('socialPost'), profileId: z.string(), likes: z.number() }),
  z.object({
    kind: z.literal('listing'),
    priceCents: z.number(),
    condition: z.string(),
    location: z.string(),
    seller: linkSchema,
    status: z.string(),
  }),
  z.object({ kind: z.literal('article'), readingMinutes: z.number() }),
  z.object({
    kind: z.literal('product'),
    priceCents: z.number(),
    stock: z.string(),
    seller: linkSchema,
    specifications: table,
  }),
  z.object({
    kind: z.literal('recipe'),
    minutes: z.number(),
    servings: z.number(),
    difficulty: z.string(),
    ingredients: z.array(z.string()),
    steps: z.array(z.string()),
  }),
  z.object({
    kind: z.literal('course'),
    minutes: z.number(),
    level: z.string(),
    teacher: linkSchema,
    lessons: z.array(linkSchema),
  }),
  z.object({
    kind: z.literal('lesson'),
    course: linkSchema,
    position: z.number(),
    next: linkSchema.nullable(),
  }),
  z.object({
    kind: z.literal('thread'),
    community: z.string(),
    votes: z.number(),
    locked: z.boolean(),
  }),
  z.object({
    kind: z.literal('video'),
    seconds: z.number(),
    views: z.number(),
    channel: linkSchema,
    chapters: z.array(z.string()),
  }),
  z.object({ kind: z.literal('profile'), bio: z.string(), location: z.string() }),
  z.object({ kind: z.literal('reference'), facts: table }),
]);
export const webDocumentSchema = z.object({
  id: z.string(),
  brandId: z.string(),
  path: z.string(),
  title: z.string(),
  summary: z.string(),
  category: z.string(),
  author: z.string(),
  publishedAt: z.number(),
  updatedAt: z.number().nullable().optional(),
  entities: z.array(z.string()),
  blocks: z.array(blockSchema),
  detail: detailSchema,
  visual: z.string(),
  links: z.array(linkSchema),
  comments: z.array(
    z.object({
      id: z.string(),
      author: z.string(),
      text: z.string(),
      publishedAt: z.number(),
      rating: z.number().nullable(),
    }),
  ),
});
export const cardSchema = z.object({
  listingStatus: z.string().nullable().optional(),
  rating: z.number().nullable().optional(),
  id: z.string(),
  url: z.string(),
  title: z.string(),
  summary: z.string(),
  category: z.string(),
  author: z.string(),
  visual: z.string(),
  priceCents: z.number().nullable(),
  stock: z.string().nullable().optional(),
});
export const webHistorySchema = z.array(
  z.object({ url: z.string(), title: z.string(), favicon: z.string(), timestamp: z.number() }),
);
export const webPageSchema = z.object({
  archiveUrl: z.string().nullable().optional(),
  filters: z.record(z.string(), z.string()).optional(),
  ads: z
    .array(
      z.object({
        id: z.string(),
        advertiser: z.string(),
        title: z.string(),
        description: z.string(),
        url: z.string(),
      }),
    )
    .optional(),
  capture: z.object({ timestamp: z.number(), originalUrl: z.string() }).nullable().optional(),
  community: z
    .object({
      suggested: z.array(cardSchema).optional(),
      connections: z.array(cardSchema).optional(),
      following: z.array(z.string()),
      followers: z.number(),
      feed: z.array(cardSchema),
      nickname: z.string(),
      posts: z.array(z.object({ id: z.string(), text: z.string(), timestamp: z.number() })),
      messages: z.array(
        z.object({
          id: z.string(),
          target: z.string(),
          text: z.string(),
          incoming: z.boolean(),
          timestamp: z.number(),
        }),
      ),
      notifications: z.array(
        z.object({ id: z.string(), title: z.string(), url: z.string(), read: z.boolean() }),
      ),
      listing: z.object({ status: z.string(), offerCents: z.number().nullable() }).nullable(),
    })
    .nullable()
    .optional(),
  brand: z.object({
    depth: z.string().nullable().optional(),
    identity: z
      .object({
        typography: z.string(),
        header: z.string(),
        cards: z.string(),
        density: z.string(),
        surface: z.string(),
      })
      .nullable()
      .optional(),
    id: z.string(),
    name: z.string(),
    domain: z.string(),
    platform: z.string(),
    layout: z.string(),
    tagline: z.string(),
    accent: z.string(),
    mark: z.string(),
    navigation: z.array(linkSchema),
    voice: z.string(),
  }),
  canonicalUrl: z.string(),
  status: z.number(),
  document: webDocumentSchema.nullable(),
  cards: z.array(cardSchema),
  related: z.array(cardSchema),
  categories: z.array(z.string()),
  total: z.number(),
  offset: z.number(),
  query: z.string(),
  category: z.string(),
  liked: z.boolean(),
  completed: z.boolean(),
  cartCount: z.number(),
  cart: z.array(
    z.object({ card: cardSchema, quantity: z.number(), available: z.boolean().optional() }),
  ),
  offerHistory: z
    .array(z.object({ timestamp: z.number(), priceCents: z.number(), stock: z.string() }))
    .default([]),
  history: webHistorySchema,
});
export type WebPage = z.infer<typeof webPageSchema>;
export type WebDocument = z.infer<typeof webDocumentSchema>;
export type WebCard = z.infer<typeof cardSchema>;
export type WebLink = z.infer<typeof linkSchema>;
export const money = (cents: number) =>
  (cents / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
export const date = (seconds: number) =>
  new Date(seconds * 1000).toLocaleDateString('pt-BR', {
    timeZone: 'UTC',
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  });
export const dateTime = (seconds: number) =>
  new Date(seconds * 1000).toLocaleString('pt-BR', {
    timeZone: 'UTC',
    dateStyle: 'medium',
    timeStyle: 'short',
  });
export function stockLabel(stock: string | null | undefined) {
  return (
    (
      {
        IN_STOCK: 'Em estoque',
        LOW_STOCK: 'Últimas unidades',
        OUT_OF_STOCK: 'Indisponível',
        DISCONTINUED: 'Fora de linha',
        PREORDER: 'Pré-venda',
      } as Record<string, string>
    )[stock ?? ''] ?? 'Indisponível'
  );
}
