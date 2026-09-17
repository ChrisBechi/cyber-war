import { pipeline } from './cli/pipeline.mjs';
import { coreutilsReport } from './cli/coreutils.mjs';
import { digest, evidenceSources, json, write } from './cli/io.mjs';
import { waveDecision, EXECUTABLE_LIMIT } from './cli/coreutils-wave.mjs';

const args = process.argv.slice(2);
const processed = args.includes('--processed')
  ? args[args.indexOf('--processed') + 1].split(',')
  : [];
const report = pipeline();
const queue = coreutilsReport(report);
const evidence = json('artifacts/cli-evidence.json');
if (evidence.fingerprint !== digest(evidenceSources()))
  throw new Error('Refresh global evidence before deriving wave status');
const decisions = waveDecision(queue, processed);
const result = {
  schemaVersion: 1,
  limit: EXECUTABLE_LIMIT,
  fingerprint: evidence.fingerprint,
  processed: processed.map((name) => {
    const command = queue.commands.find((c) => c.command === name);
    if (!command) throw new Error(`Unknown processed executable ${name}`);
    return command;
  }),
  ...decisions,
};
write('artifacts/coreutils-wave.json', result);
console.log(JSON.stringify(result, null, 2));
