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
    names,
    includes: (c) => !names || (c.softwareId === 'coreutils' && names.includes(c.command)),
  };
}
