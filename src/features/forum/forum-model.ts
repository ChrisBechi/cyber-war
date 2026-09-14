import { z } from 'zod';
import community from '../../../content/forums/community.json';
import seedThreads from '../../../content/forums/threads.json';

const quoteSchema = z.object({ author: z.string(), text: z.string() });
export const forumPostSchema = z.object({
  id: z.string(),
  author: z.string(),
  text: z.string(),
  createdAt: z.string(),
  quote: quoteSchema.optional(),
});
const threadSchema = z.object({
  id: z.string(),
  title: z.string(),
  author: z.string(),
  body: z.string(),
  category: z.string(),
  createdAt: z.string(),
  tag: z.string().optional(),
  pinned: z.boolean().optional(),
  locked: z.boolean().optional(),
  requiredFlag: z.string().nullable().optional(),
  replies: z.array(quoteSchema),
});
export const forumStateSchema = z.object({
  threads: z.array(threadSchema).default([]),
  replies: z.record(z.string(), z.array(forumPostSchema)).default({}),
  followed: z.array(z.string()).default([]),
  readCounts: z.record(z.string(), z.number()).default({}),
  reports: z
    .array(z.object({ threadId: z.string(), postId: z.string(), reason: z.string() }))
    .default([]),
});
export type ForumState = z.infer<typeof forumStateSchema>;
export type ForumPost = z.infer<typeof forumPostSchema>;
export type ForumThread = z.infer<typeof threadSchema> & { posts: ForumPost[]; updatedAt: string };
export type ForumMember = (typeof community.members)[number];
export type ForumCategory = (typeof community.categories)[number];
export type ForumAction =
  | { kind: 'createThread'; category: string; title: string; body: string }
  | { kind: 'reply'; threadId: string; body: string; quoteId: string | null }
  | { kind: 'toggleFollow' | 'markRead'; threadId: string }
  | { kind: 'markAllRead' }
  | { kind: 'report'; threadId: string; postId: string; reason: string };
export type ForumDispatch = (action: ForumAction) => Promise<string | null>;
export const forumCategories = community.categories;
export const forumGroups = community.groups;
export const emptyForumState = (): ForumState => forumStateSchema.parse({});

export function boardThreads(state: ForumState, flags: string[]): ForumThread[] {
  return [...seedThreads, ...state.threads]
    .filter((thread) => !thread.requiredFlag || flags.includes(thread.requiredFlag))
    .map((thread) => {
      const posts: ForumPost[] = [
        {
          id: `${thread.id}:op`,
          author: thread.author,
          text: thread.body,
          createdAt: thread.createdAt,
        },
        ...thread.replies.map((reply, index) => ({
          ...reply,
          id: `${thread.id}:reply:${index}`,
          createdAt: new Date(Date.parse(thread.createdAt) + (index + 1) * 60000).toISOString(),
        })),
        ...(state.replies[thread.id] ?? []),
      ];
      return { ...thread, posts, updatedAt: posts.at(-1)?.createdAt ?? thread.createdAt };
    })
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));
}

export function boardMembers(nickname: string, reputation: number): ForumMember[] {
  return [
    ...community.members.filter((member) => member.name !== nickname),
    {
      name: nickname,
      rank: 'Membro',
      color: 'teal',
      joined: '2026-09-14',
      reputation,
      signature: 'Cada descoberta começa com uma pergunta.',
      bio: 'Membro da comunidade Terminal Board.',
      avatar: 'ghost',
    },
  ];
}

export function searchThreads(
  threads: ForumThread[],
  query: string,
  author?: string,
): ForumThread[] {
  const normalize = (value: string) =>
    value
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, '')
      .toLowerCase();
  const words = normalize(query).trim().split(/\s+/).filter(Boolean);
  return threads.filter((thread) => {
    if (author && !thread.posts.some((post) => post.author === author)) {
      return false;
    }
    const haystack = normalize(
      `${thread.title} ${thread.posts.map((post) => `${post.author} ${post.text}`).join(' ')}`,
    );
    return words.every((word) => haystack.includes(word));
  });
}

export function forumDate(value: string, full = false): string {
  return new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit',
    month: full ? 'long' : 'short',
    year: full ? 'numeric' : undefined,
    hour: '2-digit',
    minute: '2-digit',
    timeZone: 'America/Sao_Paulo',
  }).format(new Date(value));
}

export function memberStats(name: string, threads: ForumThread[]) {
  return {
    threads: threads.filter((thread) => thread.author === name).length,
    posts: threads.reduce(
      (count, thread) => count + thread.posts.filter((post) => post.author === name).length,
      0,
    ),
  };
}
