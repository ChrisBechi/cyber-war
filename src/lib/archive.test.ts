import { expect, it } from 'vitest';
import { archiveChildren } from './archive';
import { domainRecordSchema } from './api';

it('accepts the permanent NPC domain sentinel returned by a real native world', () => {
  expect(domainRecordSchema.shape.expiresAtSeconds.parse(9_223_372_036_854_776_000)).toBe(
    Number.MAX_SAFE_INTEGER,
  );
  expect(() => domainRecordSchema.shape.expiresAtSeconds.parse(-1)).toThrow();
  expect(() => domainRecordSchema.shape.expiresAtSeconds.parse(1.5)).toThrow();
});
it('an empty archive index has no synthetic content', () => {
  expect(archiveChildren([], '')).toEqual([]);
});
