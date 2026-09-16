import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { developmentTransport } from './development-transport';

export const nodeSchema = z.object({
  id: z.string(),
  parentId: z.string().nullable(),
  name: z.string(),
  kind: z.enum(['file', 'directory', 'symlink']),
  content: z.string(),
  blob: z
    .object({ hash: z.string(), size: z.number().int().nonnegative(), mime: z.string() })
    .optional(),
  owner: z.string(),
  group: z.string(),
  mode: z.number(),
  modifiedAt: z.number(),
  metadata: z.record(z.string(), z.string()),
});
export type VfsNode = z.infer<typeof nodeSchema>;
export const subdomainSchema = z.object({
  label: z.string(),
  address: z.string(),
  createdAtSeconds: z.number().int().nonnegative(),
});
export const domainRecordSchema = z.object({
  address: z.string(),
  registeredName: z.string(),
  suffix: z.string(),
  category: z.string(),
  country: z.string().nullable(),
  owner: z.string(),
  ownerKind: z.string(),
  organization: z.string(),
  businessType: z.string(),
  registeredAtSeconds: z.number().int().nonnegative(),
  // Permanent NPC registrations use a Rust u64 sentinel in existing saves.
  expiresAtSeconds: z
    .number()
    .nonnegative()
    .refine(Number.isInteger)
    .transform((value) => Math.min(value, Number.MAX_SAFE_INTEGER)),
  autoRenew: z.boolean(),
  primary: z.boolean(),
  redirectTo: z.string().nullable(),
  listedPrice: z.number().int().nullable(),
  subdomains: z.record(z.string(), subdomainSchema),
});
export const domainMarketOfferSchema = z.object({
  id: z.string(),
  address: z.string(),
  buyer: z.string(),
  amount: z.number().int(),
  status: z.string(),
  counterPrice: z.number().int().nullable(),
});
export const domainStateSchema = z.object({
  registrations: z.record(z.string(), domainRecordSchema),
  offers: z.array(domainMarketOfferSchema),
  onionServices: z
    .record(
      z.string(),
      z.object({
        address: z.string(),
        serviceName: z.string(),
        description: z.string(),
        owner: z.string(),
        availabilityMode: z.string(),
        lastSeenSeconds: z.number().int().nonnegative(),
      }),
    )
    .default({}),
});
export const domainOfferSchema = z.object({
  address: z.string(),
  registeredName: z.string(),
  suffix: z.string(),
  category: z.string(),
  country: z.string().nullable(),
  description: z.string(),
  price: z.number().int(),
  renewalPrice: z.number().int(),
  available: z.boolean(),
  restricted: z.boolean(),
  restrictionReason: z.string().nullable(),
  premium: z.boolean(),
  recommendation: z.string(),
  recommended: z.boolean(),
  status: z.string(),
  owner: z.string().nullable(),
  registeredAtYear: z.number().int().nullable(),
  expiresAtYear: z.number().int().nullable(),
});
export const domainSearchSchema = z.object({
  query: z.string(),
  normalizedName: z.string(),
  results: z.array(domainOfferSchema),
  suggestions: z.array(z.string()),
});
export const domainWhoisSchema = z.object({
  address: z.string(),
  ip: z.string().nullable(),
  status: z.string(),
  owner: z.string().nullable(),
  registeredAtYear: z.number().int().nullable(),
  expiresAtYear: z.number().int().nullable(),
  suffix: z.string(),
  category: z.string(),
  country: z.string().nullable(),
  subdomains: z.array(z.string()),
  redirectTo: z.string().nullable(),
  network: z.string().nullable().default(null),
  kind: z.string().nullable().default(null),
  usesTraditionalDns: z.boolean().nullable().default(null),
  addressVersion: z.number().int().nullable().default(null),
  registrable: z.boolean().nullable().default(null),
  purchasable: z.boolean().nullable().default(null),
  tradable: z.boolean().nullable().default(null),
});
export type DomainRecord = z.infer<typeof domainRecordSchema>;
export type DomainMarketOffer = z.infer<typeof domainMarketOfferSchema>;
export type DomainOffer = z.infer<typeof domainOfferSchema>;
export type DomainSearchResult = z.infer<typeof domainSearchSchema>;
export type DomainWhois = z.infer<typeof domainWhoisSchema>;
export const onionServiceInfoSchema = z.object({
  address: z.string(),
  serviceName: z.string(),
  description: z.string(),
  owner: z.string(),
  network: z.literal('tor'),
  kind: z.literal('onion_service'),
  type: z.literal('special_use'),
  extension: z.literal('.onion'),
  addressVersion: z.literal(3),
  addressLength: z.literal(56),
  usesTraditionalDns: z.literal(false),
  registrable: z.literal(false),
  purchasable: z.literal(false),
  tradable: z.literal(false),
  generatedFromCryptographicIdentity: z.literal(true),
  allowedCharacters: z.literal('abcdefghijklmnopqrstuvwxyz234567'),
  online: z.boolean(),
  availabilityMode: z.enum(['always', 'intermittent', 'offline']),
  availability: z.string(),
  lastSeenSeconds: z.number().int().nonnegative(),
  nextChangeSeconds: z.number().int().nonnegative().nullable(),
});
export type OnionServiceInfo = z.infer<typeof onionServiceInfoSchema>;
export const worldSchema = z.object({
  packages: z
    .object({
      managed: z.array(z.string()),
      installed: z.record(z.string(), z.object({ status: z.string() })),
    })
    .optional(),
  nickname: z.string(),
  hostname: z.string(),
  session: z.number(),
  money: z.number(),
  reputation: z.number(),
  playtimeSeconds: z.number(),
  vfs: z.object({ nodes: z.record(z.string(), nodeSchema) }),
  network: z.object({
    connected: z.boolean(),
    gateway: z.string(),
    hosts: z.record(
      z.string(),
      z.object({
        hostname: z.string(),
        online: z.boolean(),
        services: z.array(z.object({ port: z.number(), name: z.string(), running: z.boolean() })),
      }),
    ),
  }),
  terminal: z.object({ cwd: z.string(), user: z.string(), host: z.string().nullable() }),
  messages: z.array(
    z.object({
      id: z.string(),
      contact: z.string(),
      text: z.string(),
      read: z.boolean(),
      attachments: z.array(z.string()).optional(),
    }),
  ),
  contacts: z.array(z.string()),
  inventory: z.array(z.string()),
  techniques: z.array(z.string()),
  flags: z.array(z.string()),
  memoryValue: z.number(),
  memoryCandidates: z.array(z.number()),
  processes: z.array(
    z.object({ pid: z.number(), name: z.string(), user: z.string(), running: z.boolean() }),
  ),
  domains: domainStateSchema.default({ registrations: {}, offers: [], onionServices: {} }),
  settings: z.record(z.string(), z.string()),
});
export type World = z.infer<typeof worldSchema>;
export const missionSchema = z.object({
  id: z.string(),
  title: z.string(),
  description: z.string(),
  contact: z.string(),
  status: z.enum(['available', 'active', 'completed']),
  objective: z.string(),
  hint: z.string(),
  choices: z.array(z.object({ id: z.string(), label: z.string() })),
});
export type Mission = z.infer<typeof missionSchema>;
export const slotSchema = z.object({
  slotIndex: z.number(),
  label: z.string(),
  occupied: z.boolean(),
  currentMission: z.string().nullable(),
  playtimeSeconds: z.number(),
  updatedAt: z.string().nullable(),
  session: z.number().int().nonnegative().nullable().default(null),
});
export type SaveSlot = z.infer<typeof slotSchema>;
export const checkpointSchema = z.object({
  id: z.string(),
  checkpointType: z.string(),
  missionId: z.string().nullable(),
  label: z.string(),
  createdAt: z.string(),
});
export type Checkpoint = z.infer<typeof checkpointSchema>;
export const nanoOptionsSchema = z.object({
  smartHome: z.boolean(),
  backup: z.boolean(),
  backupDir: z.string().nullable(),
  boldText: z.boolean(),
  tabsToSpaces: z.boolean(),
  newBuffer: z.boolean(),
  locking: z.boolean(),
  historyLog: z.boolean(),
  ignoreRcfiles: z.boolean(),
  guideStripe: z.number().nullable(),
  rawSequences: z.boolean(),
  noNewlines: z.boolean(),
  trimBlanks: z.boolean(),
  noConvert: z.boolean(),
  bookStyle: z.boolean(),
  positionLog: z.boolean(),
  quoteString: z.string().nullable(),
  restricted: z.boolean(),
  softwrap: z.boolean(),
  tabSize: z.number(),
  quickBlank: z.boolean(),
  wordBounds: z.boolean(),
  wordChars: z.string().nullable(),
  syntax: z.string().nullable(),
  zap: z.boolean(),
  atBlanks: z.boolean(),
  breakLongLines: z.boolean(),
  constantShow: z.boolean(),
  rebindDelete: z.boolean(),
  emptyLine: z.boolean(),
  rcfile: z.string().nullable(),
  showCursor: z.boolean(),
  autoIndent: z.boolean(),
  jumpyScrolling: z.boolean(),
  cutFromCursor: z.boolean(),
  lineNumbers: z.boolean(),
  mouse: z.boolean(),
  noRead: z.boolean(),
  operatingDir: z.string().nullable(),
  preserve: z.boolean(),
  indicator: z.boolean(),
  fill: z.number().nullable(),
  speller: z.string().nullable(),
  unix: z.boolean(),
  view: z.boolean(),
  noWrap: z.boolean(),
  noHelp: z.boolean(),
  afterEnds: z.boolean(),
  magic: z.boolean(),
  colonParsing: z.boolean(),
  stateFlags: z.boolean(),
  minibar: z.boolean(),
  zero: z.boolean(),
  soloSideScroll: z.boolean(),
  modernBindings: z.boolean(),
});
export const nanoLaunchSchema = z.object({
  path: z.string(),
  displayName: z.string(),
  content: z.string(),
  expectedContent: z.string().nullable(),
  options: nanoOptionsSchema,
  startingLine: z.number().int().nonnegative(),
  startingColumn: z.number().int().nonnegative(),
});
export type NanoLaunch = z.infer<typeof nanoLaunchSchema>;
export const commandSchema = z.object({
  shellIncomplete: z.boolean().optional(),
  ordered: z.array(z.tuple([z.union([z.literal(1), z.literal(2)]), z.string()])).optional(),
  stdout: z.string(),
  stderr: z.string(),
  cwd: z.string(),
  user: z.string(),
  host: z.string(),
  exitCode: z.number(),
  interactive: nanoLaunchSchema.nullable().optional(),
  launchApp: z.string().nullable().optional(),
  archivePrompt: z.object({ message: z.string(), secret: z.boolean() }).nullable().optional(),
  archiveJob: z.number().nullable().optional(),
});
export const pageSchema = z.object({
  title: z.string(),
  body: z.string(),
  action: z.string().nullable(),
});
export type BrowserPage = z.infer<typeof pageSchema>;
export const desktopRuntime = isTauri();

export async function request<T>(
  command: string,
  args: Record<string, unknown>,
  schema: z.ZodType<T>,
): Promise<T> {
  const scenario = developmentTransport();
  if (scenario) {
    return schema.parse(await scenario(command, args));
  }
  if (!isTauri()) {
    throw new Error(
      'Abra o aplicativo desktop com pnpm dev. A prévia web não executa o núcleo Rust.',
    );
  }
  return schema.parse(await invoke<unknown>(command, args));
}
export const emptySchema = z.null();
