import { z } from 'zod';
import { adSchema } from '../../virtual-web/web-model';

export const GOGGLE_HOME = 'https://www.goggle.com';
export interface DefaultSearchProvider {
  id: string;
  home: string;
  searchUrl: (query: string) => string;
}
export const goggleProvider: DefaultSearchProvider = {
  id: 'goggle',
  home: GOGGLE_HOME,
  searchUrl: (query) => `${GOGGLE_HOME}/search?q=${encodeURIComponent(query)}`,
};
export function goggleRoute(address: string) {
  try {
    const url = new URL(/^https?:\/\//i.test(address) ? address : `https://${address}`);
    if (
      !['goggle.com', 'www.goggle.com'].includes(url.hostname) ||
      url.username ||
      url.password ||
      url.port
    ) {
      return null;
    }
    return {
      path: url.pathname,
      query: url.searchParams.get('q') ?? '',
      image: url.searchParams.get('image'),
      offset: Math.max(0, Number(url.searchParams.get('offset')) || 0),
      mode: ['videos', 'news', 'shopping'].includes(url.searchParams.get('type') ?? '')
        ? (url.searchParams.get('type') as 'videos' | 'news' | 'shopping')
        : ('all' as const),
    };
  } catch {
    return null;
  }
}
export const searchDocumentSchema = z.object({
  id: z.string(),
  url: z.string(),
  title: z.string(),
  description: z.string(),
  domain: z.string(),
  type: z.enum([
    'WEB_PAGE',
    'NEWS',
    'FORUM',
    'PROFILE',
    'COMPANY',
    'IMAGE',
    'MARKET',
    'BLOG',
    'DOCUMENT',
  ]),
  imageId: z.string().nullable(),
  offer: z
    .object({ priceCents: z.number(), stock: z.string(), timestamp: z.number() })
    .nullable()
    .optional(),
});
export const imageRecordSchema = z.object({
  id: z.string(),
  virtualUrl: z.string(),
  asset: z.string(),
  documentIds: z.array(z.string()),
  entities: z.array(z.string()),
});
export const searchResponseSchema = z.object({
  ads: z.array(adSchema).optional(),
  query: z.string(),
  documents: z.array(searchDocumentSchema),
  total: z.number(),
  offset: z.number(),
  images: z.array(imageRecordSchema),
  correction: z.string().nullable().optional(),
  intent: z.string().optional(),
});
export type SearchResponse = z.infer<typeof searchResponseSchema>;
export type SearchDocument = z.infer<typeof searchDocumentSchema>;
export const sessionSchema = z.object({
  account: z
    .object({
      id: z.string(),
      email: z.string(),
      displayName: z.string(),
      avatar: z.string().nullable(),
      apps: z.array(z.string()),
    })
    .nullable(),
  historyEnabled: z.boolean(),
  history: z.array(z.string()),
});
export type GoggleSession = z.infer<typeof sessionSchema>;
export const assets: Record<string, string> = {
  'orion-campus': '/assets/goggle/orion-campus.svg',
  'archive-records': '/assets/goggle/archive-records.svg',
};
export const apps = [
  { id: 'mail', name: 'Goggle Mail', status: 'COMING_SOON', icon: 'mail' },
  { id: 'images', name: 'Goggle Images', status: 'IMPLEMENTED', icon: 'image' },
  { id: 'drive', name: 'Goggle Drive', status: 'COMING_SOON', icon: 'drive' },
  { id: 'maps', name: 'Goggle Maps', status: 'COMING_SOON', icon: 'pin' },
  { id: 'news', name: 'Goggle News', status: 'COMING_SOON', icon: 'news' },
  { id: 'calendar', name: 'Goggle Calendar', status: 'COMING_SOON', icon: 'calendar' },
  { id: 'photos', name: 'Goggle Photos', status: 'COMING_SOON', icon: 'image' },
  { id: 'meet', name: 'Goggle Meet', status: 'COMING_SOON', icon: 'video' },
  { id: 'account', name: 'Goggle Account', status: 'IMPLEMENTED', icon: 'person' },
] as const;
export const appAddress = (id: string) =>
  `${GOGGLE_HOME}/${id === 'images' || id === 'account' ? id : `apps/${id}`}`;
