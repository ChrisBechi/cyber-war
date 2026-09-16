import { z } from 'zod';

export const status = z.enum(['UNVERIFIED', 'PARTIAL', 'VERIFIED']);
export const kinds = z.enum([
  'NATIVE',
  'BUILTIN',
  'COMPAT_ENGINE',
  'CATALOG_ONLY',
  'ALIAS',
  'LAUNCHER',
  'FICTIONAL',
]);
export const capabilitySubsystem = {
  VFS_READ: 'VFS',
  VFS_WRITE: 'VFS',
  PROCESS_READ: 'PROCESS',
  PROCESS_WRITE: 'PROCESS',
  NETWORK_READ: 'NETWORK',
  NETWORK_CONNECT: 'NETWORK',
  NETWORK_LISTEN: 'NETWORK',
  DNS: 'DNS',
  HTTP: 'HTTP',
  VIRTUAL_WIFI: 'WIFI',
  PACKET_CAPTURE: 'PACKETS',
  PACKAGE_DB: 'PACKAGES',
  ARCHIVE: 'ARCHIVES',
  TTY: 'TTY',
  STDIN: 'SHELL',
  STDOUT: 'SHELL',
  STDERR: 'SHELL',
  SIGNALS: 'SIGNALS',
  INTERACTIVE: 'TTY',
  VIRTUAL_TARGETS: 'VIRTUAL_TARGETS',
  SESSIONS: 'SESSIONS',
  REMOTE_HOSTS: 'REMOTE_HOSTS',
  SERVICES: 'SERVICES',
  USERS: 'USERS',
  PERMISSIONS: 'PERMISSIONS',
  CLOCK: 'CLOCK',
};
export const capability = z.enum(Object.keys(capabilitySubsystem));
const shellRequirement = z.enum([
  'SHELL.PARSING',
  'SHELL.LISTS',
  'SHELL.EXPANSION',
  'SHELL.GLOBBING',
  'SHELL.PIPELINES',
  'SHELL.REDIRECTION',
  'SHELL.CONTEXT',
  'SHELL.JOBS',
]);
export const subsystemId = z.enum([...new Set(Object.values(capabilitySubsystem))]);
export const gateNames = [
  'DISCOVERY',
  'REFERENCE_PINNED',
  'PARSER',
  'SHORT_FLAGS',
  'LONG_FLAGS',
  'COMBINED_FLAGS',
  'POSITIONAL_ARGS',
  'HELP',
  'VERSION',
  'STDOUT',
  'STDERR',
  'EXIT_CODE',
  'STDIN',
  'PIPE',
  'REDIRECTION',
  'TTY',
  'SIGNALS',
  'SIDE_EFFECTS',
  'ERRORS',
  'PERMISSIONS',
  'STATE_CONSISTENCY',
  'CROSS_TOOL_CONSISTENCY',
  'HOST_ISOLATION',
  'ALIAS_RESOLUTION',
];
export const gateName = z.enum(gateNames);
export const applicability = z.enum(['REQUIRED', 'OPTIONAL', 'NOT_APPLICABLE']);
export const gateSpec = z
  .object({ applicability, reason: z.string().min(8), testIds: z.array(z.string()).default([]) })
  .strict();
const gates = z.partialRecord(gateName, gateSpec).default({});
const source = z
  .object({
    kind: z.enum([
      'OFFICIAL_DOCS',
      'MAN_PAGE',
      'UPSTREAM_HELP',
      'SOURCE_CODE',
      'REFERENCE_ENVIRONMENT',
    ]),
    url: z.string().min(1),
    notes: z.string().min(1),
  })
  .strict();
export const softwareSpec = z
  .object({
    displayName: z.string().min(1),
    packageName: z.string().min(1),
    classification: z.enum(['REAL_COMPAT', 'FICTIONAL_NATIVE']),
    upstream: z.string().nullable(),
    referenceVersion: z.string().min(1).nullable(),
    referenceDistribution: z.string().nullable(),
    referenceSources: z.array(source),
    priority: z.enum(['P0', 'P1', 'P2', 'P3']),
    wave: z.enum([
      'W0_COMPAT_INFRA',
      'W1_CORE_LINUX',
      'W2_SYSTEM',
      'W3_NETWORK_FOUNDATION',
      'W4_RECON',
      'W5_WIRELESS',
      'W6_SECURITY_FRAMEWORK',
      'W7_LONG_TAIL',
    ]),
    dependencies: z.array(z.string()),
    shellRequirements: z.array(shellRequirement).default([]),
    capabilities: z.array(capability),
    intentionalDeviations: z.array(z.string()),
    unsupportedFeatures: z.array(z.string()),
    declaredStatus: status.optional(),
    gates,
  })
  .strict();
export const nativeSpec = z
  .object({
    softwareId: z.string(),
    shellRequirements: z.array(shellRequirement).optional(),
    implementationKind: kinds,
    implementation: z.string(),
    parserKind: z.enum(['SHELL', 'PROGRAM_SPECIFIC', 'CATALOG_ADAPTER']),
    resultContract: z.enum(['STRUCTURED', 'LEGACY_STRING']),
    capabilities: z.array(capability),
    aliasOf: z.string().nullable(),
    declaredStatus: status.optional(),
    gates,
    requirement: z.enum(['REQUIRED', 'OPTIONAL', 'DEPRECATED']).default('REQUIRED'),
    referenceVersion: z.string().optional(),
  })
  .strict();
export const manifestSchema = z
  .object({
    schemaVersion: z.literal(2),
    software: z.record(z.string(), softwareSpec),
    native: z.record(z.string(), nativeSpec),
    collisions: z.record(
      z.string(),
      z
        .object({ winner: z.string(), sources: z.array(z.string()), reason: z.string().min(10) })
        .strict(),
    ),
  })
  .strict();
export const ttySchema = z
  .object({
    isTTY: z.boolean(),
    columns: z.number().int().min(1).max(1000),
    rows: z.number().int().min(1).max(1000),
    ansiSupport: z.boolean(),
    interactive: z.boolean(),
  })
  .strict();
export const baselineSchema = z
  .object({
    schemaVersion: z.literal(2),
    distribution: z.string(),
    snapshot: z.string().nullable(),
    architecture: z.string(),
    locale: z.string(),
    timezonePolicy: z.string(),
    shell: z.string(),
    shellVersion: z.string().nullable(),
    terminal: ttySchema,
    notes: z.string(),
  })
  .strict();
export const subsystemSchema = z
  .object({
    schemaVersion: z.literal(2),
    subsystems: z.array(
      z
        .object({
          id: subsystemId,
          state: z.enum(['MISSING', 'PARTIAL', 'READY']),
          reason: z.string().min(12),
          implementation: z.array(z.string()),
          evidenceIds: z.array(z.string()),
          requiredTests: z.array(z.string()),
          limitations: z.array(z.string()),
          capabilities: z
            .array(
              z
                .object({
                  id: z.string().regex(/^[A-Z]+\.[A-Z_]+$/),
                  state: z.enum(['MISSING', 'PARTIAL', 'READY']),
                  reason: z.string().min(12),
                  requiredTests: z.array(z.string()),
                  evidenceIds: z.array(z.string()),
                  limitations: z.array(z.string()),
                })
                .strict(),
            )
            .default([]),
        })
        .strict(),
    ),
  })
  .strict();
export const matcherSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('EXACT'), value: z.string() }).strict(),
  z
    .object({
      kind: z.literal('REGEX'),
      pattern: z.string().max(2000),
      flags: z.enum(['', 'i', 'm', 'im']).default(''),
    })
    .strict(),
  z.object({ kind: z.literal('STRUCTURED'), value: z.json() }).strict(),
  z
    .object({
      kind: z.literal('NORMALIZED_DYNAMIC'),
      value: z.string(),
      rules: z
        .array(
          z
            .object({
              type: z.enum(['PID', 'TIMESTAMP', 'LATENCY', 'TRANSFER_RATE', 'IDENTIFIER']),
              prefix: z.string().min(1),
              suffix: z.string().min(1),
              reason: z.string().min(15),
              token: z.string().min(1),
            })
            .strict(),
        )
        .min(1),
    })
    .strict(),
]);
export const assertionSchema = z
  .object({
    path: z.string().startsWith('/'),
    matcher: matcherSchema.optional(),
    absent: z.boolean().optional(),
    unchanged: z.boolean().optional(),
  })
  .strict()
  .refine(
    (v) => [!!v.matcher, !!v.absent, !!v.unchanged].filter(Boolean).length === 1,
    'Choose exactly one assertion',
  );
export const caseSchema = z
  .object({
    schemaVersion: z.literal(2),
    id: z.string().regex(/^[a-z0-9][a-z0-9/_.-]+$/),
    softwareId: z.string(),
    command: z.string().min(1),
    argv: z.array(z.string()),
    script: z.string().optional(),
    inputEvents: z
      .array(
        z.discriminatedUnion('kind', [
          z.object({ kind: z.literal('stdin'), data: z.string().max(65536) }).strict(),
          z.object({ kind: z.literal('eof') }).strict(),
          z.object({ kind: z.literal('cancel') }).strict(),
        ]),
      )
      .default([]),
    stdin: z.string().nullable(),
    env: z.record(z.string(), z.string()),
    cwd: z.string().startsWith('/'),
    tty: ttySchema,
    reference: z
      .object({
        softwareId: z.string(),
        version: z.string().min(1),
        kind: source.shape.kind,
        source: z.string().min(1),
      })
      .strict(),
    gates: z.array(gateName),
    flags: z.array(z.string()).default([]),
    errorCase: z.boolean().default(false),
    fixture: z
      .object({
        files: z.record(z.string(), z.string()).default({}),
        directories: z.array(z.string()).default([]),
        modes: z.record(z.string(), z.number().int().min(0).max(511)).default({}),
        setup: z.array(z.string()).default([]),
      })
      .strict(),
    expected: z
      .object({
        stdout: matcherSchema,
        stderr: matcherSchema,
        exitCode: z.number().int(),
        state: z.array(assertionSchema).default([]),
      })
      .strict(),
  })
  .strict();

export const overridesSchema = z
  .object({
    schemaVersion: z.literal(2),
    overrides: z.array(
      z
        .object({
          id: z.string().min(1),
          reason: z.string().min(20),
          reference: z.string().min(3),
          expires: z.iso.datetime(),
        })
        .strict(),
    ),
  })
  .strict();
export const debtSchema = z
  .object({
    schemaVersion: z.literal(2),
    software: z.array(
      z
        .object({
          id: z.string(),
          effectiveStatus: status,
          referenceVersion: z.string().nullable(),
        })
        .strict(),
    ),
    commands: z.array(
      z
        .object({
          id: z.string(),
          effectiveStatus: status,
          implementationKind: kinds,
          referenceVersion: z.string().nullable(),
          gates: z.record(z.string(), z.json()),
        })
        .strict(),
    ),
    subsystems: z.array(
      z
        .object({ id: subsystemId, effectiveState: z.enum(['MISSING', 'PARTIAL', 'READY']) })
        .strict(),
    ),
  })
  .strict();

export const referenceFixtureSchema = z
  .object({
    schemaVersion: z.literal(2),
    softwareId: z.string(),
    version: z.string().min(1),
    provenance: z.enum(['DECLARED_EXPECTATIONS', 'CAPTURED_REFERENCE']),
    environmentDigest: z.string().nullable(),
    caseIds: z.array(z.string()).min(1),
    notes: z.string().min(20),
  })
  .strict()
  .refine(
    (v) => v.provenance !== 'CAPTURED_REFERENCE' || !!v.environmentDigest,
    'Captured reference requires environment digest',
  );
