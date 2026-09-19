import { performance } from 'node:perf_hooks';
import {
  manifestSchema,
  baselineSchema,
  subsystemSchema,
  gateNames,
  capabilitySubsystem,
} from './schema.mjs';
import { read, json, files } from './io.mjs';

export const id = (group, name) => `command.${group}.${encodeURIComponent(name)}`;
const sort = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
function requirements(capabilities, shell = [], vfs = [], runtime = {}) {
  const subsystems = capabilities.map((c) => capabilitySubsystem[c]);
  return [
    ...new Set(
      subsystems.flatMap((id) =>
        id === 'SHELL' && shell.length
          ? shell
          : id === 'VFS' && vfs.length
            ? vfs
            : (runtime[id] ?? [id]),
      ),
    ),
  ].sort(sort);
}
export function defaults() {
  return Object.fromEntries(
    gateNames.map((g) => [
      g,
      {
        applicability: g === 'ALIAS_RESOLUTION' ? 'NOT_APPLICABLE' : 'REQUIRED',
        reason:
          g === 'ALIAS_RESOLUTION'
            ? 'Only applies to declared aliases.'
            : 'Unreviewed gate remains required until a documented applicability decision.',
        testIds: [],
      },
    ]),
  );
}
export function discover(
  runtime,
  configInput,
  catalog,
  baselineInput,
  subsystemInput,
  missions = [],
) {
  const start = performance.now();
  const config = manifestSchema.parse(configInput),
    baseline = baselineSchema.parse(baselineInput),
    subsystems = subsystemSchema.parse(subsystemInput).subsystems;
  const errors = [];
  const software = new Map();
  const claims = new Map();
  const launchers = [];
  const packageVersions = new Map();
  const bindings = new Map();
  for (const p of runtime.packages) {
    if (!packageVersions.has(p.name)) packageVersions.set(p.name, new Set());
    packageVersions.get(p.name).add(p.version);
    for (const f of p.files)
      if (f.binding) {
        const name = f.path.split('/').at(-1);
        if (!bindings.has(name)) bindings.set(name, []);
        bindings
          .get(name)
          .push({ packageName: p.name, version: p.version, path: f.path, binding: f.binding });
      }
    for (const f of p.files)
      if (f.path.endsWith('.desktop'))
        launchers.push({
          id: `package:${p.name}:${f.path}`,
          kind: 'GUI',
          packageName: p.name,
          source: 'src-tauri/src/packages/repository.rs',
          path: f.path,
        });
  }
  function group(name, entry) {
    if (software.has(name)) return software.get(name);
    const spec = config.software[name] ?? {
      displayName: name,
      packageName: name,
      classification: entry?.fictional ? 'FICTIONAL_NATIVE' : 'REAL_COMPAT',
      upstream: entry?.source ?? null,
      referenceVersion: null,
      referenceDistribution: null,
      referenceSources: [],
      priority: 'P3',
      wave: 'W7_LONG_TAIL',
      dependencies: [],
      capabilities: ['STDOUT', 'STDERR'],
      intentionalDeviations: [],
      unsupportedFeatures: ['Native CLI contract not audited; catalog adapters only.'],
      gates: {},
    };
    const row = {
      id: name,
      ...spec,
      requirements: requirements(spec.capabilities, spec.shellRequirements, spec.vfsRequirements),
      executables: [],
      aliases: [],
      launchers: [],
      packageVersions: [...(packageVersions.get(spec.packageName) ?? [])].sort(sort),
      usedByMissions: [],
      verificationSummary: {},
    };
    software.set(name, row);
    return row;
  }
  const add = (name, claim) => {
    if (!claims.has(name)) claims.set(name, []);
    if (!claims.get(name).some((r) => r.source === claim.source)) claims.get(name).push(claim);
  };
  for (const name of [...runtime.native, ...runtime.shellOnly]) {
    const spec = config.native[name];
    if (!spec) {
      errors.push(`Registered native command without metadata: ${name}`);
      continue;
    }
    group(spec.softwareId);
    add(name, {
      ...spec,
      source: 'native',
      implementationKind: spec.aliasOf ? 'ALIAS' : spec.implementationKind,
    });
  }
  for (const name of Object.keys(config.native))
    if (!runtime.native.includes(name) && !runtime.shellOnly.includes(name))
      errors.push(`Orphan command metadata: ${name}`);
  for (const entry of catalog.entries) {
    group(entry.package, entry);
    launchers.push({
      id: `catalog:${entry.id}`,
      kind: 'CATALOG',
      softwareId: entry.package,
      packageName: entry.package,
      source: 'content/software/kali-default.json',
    });
    for (const name of new Set([...entry.commands, entry.id, entry.name])) {
      const executable = entry.commands.includes(name);
      add(name, {
        source: `catalog:${entry.id}`,
        softwareId: entry.package,
        implementationKind: executable ? 'CATALOG_ONLY' : 'LAUNCHER',
        implementation: 'src-tauri/src/software.rs::command',
        parserKind: 'CATALOG_ADAPTER',
        resultContract: 'LEGACY_STRING',
        aliasOf: null,
        capabilities: group(entry.package).capabilities,
        gates: {},
        requirement: executable ? 'REQUIRED' : 'OPTIONAL',
      });
    }
  }
  for (const [name, list] of bindings) {
    if (claims.has(name)) continue;
    const binding = list[0];
    const fictional = ['netscan', 'wireless', 'iot'].includes(binding.binding);
    group(binding.packageName, { fictional });
    add(name, {
      source: 'package',
      softwareId: binding.packageName,
      implementationKind: fictional ? 'FICTIONAL' : 'CATALOG_ONLY',
      implementation: 'src-tauri/src/packages/executables.rs',
      parserKind: 'PROGRAM_SPECIFIC',
      resultContract: 'STRUCTURED',
      aliasOf: null,
      capabilities: fictional ? ['NETWORK_READ', 'STDOUT'] : ['STDOUT'],
      gates: {},
      requirement: 'REQUIRED',
    });
  }
  const commands = [];
  for (const [name, options] of claims) {
    const sources = options.map((o) => o.source).sort(sort);
    let selected = options[0];
    if (options.length > 1) {
      const rule = config.collisions[name];
      if (
        !rule ||
        JSON.stringify([...rule.sources].sort(sort)) !== JSON.stringify(sources) ||
        !options.some((o) => o.source === rule.winner)
      ) {
        errors.push(`Unresolved duplicate command ${name}: ${sources.join(', ')}`);
        continue;
      }
      selected = options.find((o) => o.source === rule.winner);
    }
    const family = group(selected.softwareId);
    if (selected.referenceVersion && selected.referenceVersion !== family.referenceVersion)
      errors.push(`Version mismatch for ${name}`);
    const gates = { ...defaults(), ...family.gates, ...selected.gates };
    if (selected.aliasOf)
      gates.ALIAS_RESOLUTION = {
        applicability: 'REQUIRED',
        reason: 'Alias resolution and result/argv forwarding require proof.',
        testIds: [],
        ...selected.gates.ALIAS_RESOLUTION,
      };
    if (family.classification === 'FICTIONAL_NATIVE')
      gates.REFERENCE_PINNED = {
        applicability: 'NOT_APPLICABLE',
        reason: 'Fictional game software has no upstream reference.',
        testIds: [],
      };
    for (const g of ['DISCOVERY', 'HOST_ISOLATION'])
      if (gates[g].applicability !== 'REQUIRED') errors.push(`${name}: ${g} cannot be waived`);
    if (
      family.classification === 'REAL_COMPAT' &&
      gates.REFERENCE_PINNED.applicability !== 'REQUIRED'
    )
      errors.push(`${name}: reference pin cannot be waived`);
    const row = {
      id: id(family.id, name),
      command: name,
      ...selected,
      sources,
      classification: family.classification,
      referenceVersion: family.referenceVersion,
      executablePath: bindings.get(name)?.[0]?.path ?? null,
      packageBindings: bindings.get(name) ?? [],
      packageName: family.packageName,
      availability: bindings.has(name)
        ? runtime.baselinePackages.some((p) => bindings.get(name).some((b) => b.packageName === p))
          ? 'BASE_SYSTEM'
          : 'INSTALLABLE'
        : family.classification === 'FICTIONAL_NATIVE'
          ? 'FICTIONAL'
          : name === 'sector-ix'
            ? 'MISSION_PROVIDED'
            : 'BASE_SYSTEM',
      gates,
      requirements: requirements(
        selected.capabilities,
        selected.shellRequirements ?? family.shellRequirements,
        selected.vfsRequirements ?? family.vfsRequirements,
        selected.runtimeRequirements,
      ),
      man: { command: name, referenceSources: family.referenceSources },
      usedByMissions: missions
        .filter((m) =>
          new RegExp(
            `(?:^|[^a-zA-Z0-9_-])${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}(?:$|[^a-zA-Z0-9_-])`,
          ).test(m.text),
        )
        .map((m) => m.id),
    };
    commands.push(row);
    family[
      row.implementationKind === 'ALIAS'
        ? 'aliases'
        : row.implementationKind === 'LAUNCHER'
          ? 'launchers'
          : 'executables'
    ].push(row.id);
    family.usedByMissions = [...new Set([...family.usedByMissions, ...row.usedByMissions])].sort(
      sort,
    );
  }
  for (const name of Object.keys(config.collisions))
    if ((claims.get(name)?.length ?? 0) < 2) errors.push(`Stale collision policy: ${name}`);
  const byName = new Map(commands.map((c) => [c.command, c]));
  for (const command of commands)
    if (command.aliasOf && !byName.has(command.aliasOf))
      errors.push(`Missing alias target ${command.aliasOf}`);
    else if (command.aliasOf && byName.get(command.aliasOf).softwareId !== command.softwareId)
      errors.push(`Cross-software alias ${command.command}`);
  for (const command of commands) {
    const seen = new Set();
    let current = command;
    while (current?.aliasOf) {
      if (seen.has(current.command)) {
        errors.push(`Alias cycle at ${command.command}`);
        break;
      }
      seen.add(current.command);
      current = byName.get(current.aliasOf);
    }
  }
  for (const name of Object.keys(config.software))
    if (!software.has(name)) errors.push(`Orphan software metadata ${name}`);
  const visit = (s, path = []) => {
    if (path.includes(s.id)) {
      errors.push(`Software dependency cycle: ${[...path, s.id].join(' -> ')}`);
      return;
    }
    for (const dep of s.dependencies) {
      if (!software.has(dep)) errors.push(`Missing software dependency ${dep}`);
      else visit(software.get(dep), [...path, s.id]);
    }
  };
  for (const s of software.values()) visit(s);
  const expected = new Set(runtime.names);
  if (commands.length !== expected.size || commands.some((c) => !expected.has(c.command)))
    errors.push('Runtime names and discovered inventory differ');
  const subsystemIds = new Set(subsystems.map((s) => s.id));
  if (subsystemIds.size !== subsystems.length) errors.push('Duplicate subsystem ID');
  for (const subsystem of subsystems) {
    for (const capability of subsystem.capabilities) {
      if (subsystemIds.has(capability.id))
        errors.push(`Duplicate subsystem capability ${capability.id}`);
      if (!capability.id.startsWith(`${subsystem.id}.`))
        errors.push(`Capability parent mismatch ${capability.id}`);
      subsystemIds.add(capability.id);
    }
  }
  for (const c of commands)
    for (const r of c.requirements) if (!subsystemIds.has(r)) errors.push(`Missing subsystem ${r}`);
  if (errors.length) throw new Error(errors.join('\n'));
  const softwareRows = [...software.values()].sort((a, b) => sort(a.id, b.id));
  const uniqueLaunchers = [...new Map(launchers.map((l) => [l.id, l])).values()].sort((a, b) =>
    sort(a.id, b.id),
  );
  return {
    schemaVersion: 2,
    baseline,
    software: softwareRows,
    commands: commands.sort((a, b) => sort(a.command, b.command)),
    subsystems,
    launchers: uniqueLaunchers,
    dynamicRegistration: {
      bindings: runtime.dynamicBindings,
      policy:
        'Custom virtual .deb executable names are save-specific; global inventory covers built-in providers. No host execution binding exists.',
    },
    performance: { discoveryMs: performance.now() - start },
  };
}
export function loadDiscovery(runtime) {
  const missions = files('content/missions')
    .filter((p) => p.endsWith('.json'))
    .flatMap((p) => json(p).map((m) => ({ id: m.id, text: JSON.stringify(m) })));
  const result = discover(
    runtime,
    json('content/cli-compatibility/manifest.json'),
    json('content/software/kali-default.json'),
    json('content/cli-compatibility/baseline.json'),
    json('content/cli-compatibility/subsystems.json'),
    missions,
  );
  const builtin = read('src/lib/window-store.ts').match(
    /export type BuiltinAppId\s*=([\s\S]*?);/,
  )[1];
  for (const match of builtin.matchAll(/'([^']+)'/g))
    result.launchers.push({
      id: `gui:${match[1]}`,
      kind: 'GUI',
      source: 'src/lib/window-store.ts',
      softwareId: null,
    });
  return result;
}
