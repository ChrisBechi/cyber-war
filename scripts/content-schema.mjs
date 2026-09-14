import { z } from 'zod';

const text = z.string().min(1);
const path = text.refine(
  (value) =>
    value.startsWith('/') &&
    !value.includes(':') &&
    !value.includes('\\') &&
    [...value].every((c) => c.charCodeAt(0) >= 32),
  'Invalid virtual path',
);
const variant = (kind, shape) => z.strictObject({ kind: z.literal(kind), ...shape });
const key = { key: text };
export const conditionSchema = z.discriminatedUnion('kind', [
  variant('flag', key),
  variant('technique', key),
  variant('inventory', key),
  variant('fileContains', { path, text: z.string() }),
  variant('money', { minimum: z.int().nonnegative() }),
  variant('reputation', { minimum: z.int().nonnegative() }),
  variant('decision', { ...key, value: text }),
  variant('hostService', { host: text, port: z.int().min(1).max(65535), running: z.boolean() }),
]);
export const effectSchema = z.discriminatedUnion('kind', [
  variant('flag', key),
  variant('inventory', key),
  variant('evidence', key),
  variant('file', { path, text: z.string() }),
  variant('message', { contact: text, text }),
  variant('decision', { ...key, value: text }),
  variant('reward', { money: z.int().nonnegative(), reputation: z.int().nonnegative() }),
  variant('connection', { online: z.boolean() }),
  variant('session', { number: z.int().min(0).max(3) }),
]);
const choice = z.strictObject({
  id: text,
  label: text,
  irreversible: z.boolean(),
  effects: z.array(effectSchema).min(1),
});
export const missionSchema = z.strictObject({
  id: text,
  title: text,
  description: text,
  contact: text,
  session: z.int().min(0).max(3),
  requirements: z.array(conditionSchema),
  startTriggers: z.array(conditionSchema),
  onStart: z.array(effectSchema),
  stages: z
    .array(
      z.strictObject({
        objective: text,
        hint: z.string(),
        conditions: z.array(conditionSchema).min(1),
        choices: z.array(choice),
        effects: z.array(effectSchema),
      }),
    )
    .min(1),
  outcomes: z.array(effectSchema).min(1),
});
export const forumSchema = z.array(
  z.strictObject({
    id: text,
    title: text,
    author: text,
    body: text,
    category: text,
    createdAt: z.iso.datetime(),
    tag: text.optional(),
    pinned: z.boolean().optional(),
    locked: z.boolean().optional(),
    requiredFlag: text.optional(),
    replies: z.array(z.strictObject({ author: text, text })),
  }),
);
const service = z.strictObject({
  port: z.int().min(1).max(65535),
  name: text,
  version: text,
  running: z.boolean(),
  body: z.string(),
});
const files = z.strictObject({
  nodes: z.record(z.string(), z.unknown()),
  clock: z.int().nonnegative(),
});
export const networkSchema = z.strictObject({
  connected: z.boolean(),
  gateway: text,
  subnet: text,
  dns: z.record(z.string(), text),
  hosts: z.record(
    z.string(),
    z.strictObject({
      address: text,
      hostname: text,
      online: z.boolean(),
      services: z.array(service),
      credentials: z.record(z.string(), text),
      firewall: z.array(z.int().min(1).max(65535)),
      patched: z.boolean(),
      files,
    }),
  ),
  wifi: z.array(
    z.strictObject({
      ssid: text,
      bssid: text,
      channel: z.int().positive(),
      signal: z.int(),
      encryption: text,
      clients: z.int().nonnegative(),
      access: z.boolean(),
    }),
  ),
});
