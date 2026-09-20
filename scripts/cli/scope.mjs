import { json } from './io.mjs';
export function scope(args = process.argv.slice(2)) {
  const value = (name) => {
    const i = args.indexOf(name);
    if (i < 0) return null;
    if (!args[i + 1] || args[i + 1].startsWith('--')) throw new Error(`${name} requires a value`);
    return args[i + 1];
  };
  const command = value('--command'),
    wave = value('--wave');
  const focused = args.includes('--focused');
  if (focused && !command) throw new Error('--focused requires --command');
  if (command && wave) throw new Error('Choose --command or --wave');
  const specs = json('content/cli-compatibility/coreutils.json').commands;
  if (command && !specs[command]) throw new Error(`Unknown Coreutils command ${command}`);
  if (wave && wave !== 'foundation') throw new Error(`Unsupported wave ${wave}`);
  const names = command
    ? [command]
    : wave
      ? Object.keys(specs).filter((n) => specs[n].area === wave)
      : null;
  return {
    focused,
    names,
    includes: (c) => !names || (c.softwareId === 'coreutils' && names.includes(c.command)),
  };
}

// GNU-derived runtime readiness also requires the provider's whole command
// contracts. Capturing only its few subsystem rows cannot establish that.
export function differentialProviders(names, native, subsystems, cases) {
  const requirements = new Set(
    (names ?? []).flatMap((n) => Object.values(native[n]?.runtimeRequirements ?? {}).flat()),
  );
  const ids = new Set(
    subsystems.flatMap((s) =>
      (s.capabilities ?? [])
        .filter((c) => requirements.has(c.id) && c.readiness === 'GNU_DIFFERENTIAL')
        .flatMap((c) => c.requiredTests),
    ),
  );
  return new Set(cases.filter((c) => ids.has(c.id)).map((c) => c.command));
}
