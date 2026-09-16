// DEV ONLY contract; no subprocess or network implementation exists here.
import { z } from 'zod';
import { ttySchema } from './schema.mjs';
export const referenceRequest = z
  .object({
    softwareId: z.string(),
    version: z.string().min(1),
    command: z.string(),
    argv: z.array(z.string()),
    stdin: z.string().nullable(),
    env: z.record(z.string(), z.string()),
    cwd: z.string(),
    tty: ttySchema,
    fixtureId: z.string(),
    environmentDigest: z.string().min(1),
  })
  .strict();
export class ReferenceEnvironment {
  async execute(request) {
    referenceRequest.parse(request);
    throw new Error('Reference environment not configured. No host fallback is permitted.');
  }
}
