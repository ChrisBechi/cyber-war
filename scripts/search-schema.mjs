import { z } from 'zod';

const id = z.string().min(1).max(100);
export const virtualUrl = z
  .string()
  .max(512)
  .refine((value) => {
    try {
      const url = new URL(value);
      return (
        ['http:', 'https:'].includes(url.protocol) &&
        !url.username &&
        !url.password &&
        !url.port &&
        /^[a-z0-9.-]+$/i.test(url.hostname) &&
        !/[\\\s]/.test(value)
      );
    } catch {
      return false;
    }
  }, 'Invalid virtual identifier');
export const searchDocumentSchema = z
  .object({
    id,
    url: virtualUrl,
    title: z.string().min(1).max(300),
    description: z.string().max(2000),
    content: z.string().max(100000),
    keywords: z.array(z.string()),
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
    publishedAt: z.string().nullable().optional(),
    popularity: z.int().min(0).max(100),
    authority: z.int().min(0).max(100),
    missionTags: z.array(z.string()).default([]),
    visibility: z.enum(['PUBLIC', 'HIDDEN']),
    requiredFlags: z.array(z.string()).default([]),
    requiredMissions: z.array(z.string()).default([]),
    suggestions: z.array(z.string().min(1).max(200)).default([]),
    imageId: z.string().nullable().optional(),
  })
  .refine(
    (doc) => new URL(doc.url).hostname.replace(/^www\./, '') === doc.domain.replace(/^www\./, ''),
    'Document domain mismatch',
  );
export const searchEffectSchema = z.discriminatedUnion('kind', [
  z.strictObject({ kind: z.literal('ADD_SEARCH_DOCUMENT'), document: searchDocumentSchema }),
  z.strictObject({ kind: z.literal('REMOVE_SEARCH_DOCUMENT'), id }),
  z.strictObject({
    kind: z.literal('CHANGE_SEARCH_VISIBILITY'),
    id,
    visibility: z.enum(['PUBLIC', 'HIDDEN']),
  }),
  z.strictObject({
    kind: z.literal('ADD_SEARCH_SUGGESTION'),
    id,
    suggestion: z.object({
      text: z.string().min(1).max(200),
      requiredFlags: z.array(z.string()).default([]),
    }),
  }),
  z.strictObject({
    kind: z.literal('CHANGE_SEARCH_RANKING'),
    id,
    boost: z.int().min(-100000).max(100000),
  }),
]);
