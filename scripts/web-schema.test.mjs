import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildPack } from './web-content.mjs';
import { validateWebPack } from './web-schema.mjs';

test('the authored timeline is valid and deterministic', () => {
  const a = buildPack();
  assert.deepEqual(a, buildPack());
  assert.equal(validateWebPack(a).pack, 'web-core@3');
});

const invalid = [
  [
    'implausible cable price',
    (p) => {
      p.documents.find(
        (d) => d.detail.kind === 'product' && d.entities.includes('usb'),
      ).detail.priceCents = 400000;
    },
  ],
  [
    'implausible event phone price',
    (p) => {
      p.events.find((e) => e.offers.length).offers[0].priceCents = 10;
    },
  ],
  [
    'unknown profile in social graph',
    (p) => {
      p.documents.find((d) => d.detail.kind === 'socialProfile').detail.following = ['missing'];
    },
  ],
  [
    'listing minimum above asking price',
    (p) => {
      p.documents.find((d) => d.detail.kind === 'listing').detail.minimumCents = 9999999;
    },
  ],
  [
    'campaign with missing destination',
    (p) => {
      p.ads[0].documentId = 'missing';
    },
  ],
  [
    'campaign with reversed dates',
    (p) => {
      p.ads[0].endsAt = 1;
    },
  ],
  [
    'materializer with copied body',
    (p) => {
      p.documents.find((d) => d.materializer).blocks = [{ kind: 'paragraph', text: 'copied' }];
    },
  ],
  [
    'negative price',
    (p) => {
      p.events[1].offers[0].priceCents = -1;
    },
  ],
  [
    'unknown stock',
    (p) => {
      p.events[1].offers[0].stock = 'MAYBE';
    },
  ],
  [
    'offer on a non-product',
    (p) => {
      p.events[1].offers[0].documentId = 'web-wipedia-wiki-javascript';
    },
  ],
  [
    'two publication owners',
    (p) => {
      p.events[2].publish.push(p.events[1].publish[0]);
    },
  ],
  [
    'future link in an existing page',
    (p) => {
      p.documents
        .find((d) => d.id === 'web-wipedia-wiki-javascript')
        .links.push({ label: 'Future', url: 'https://www.nexora.com/empresa/campanha-x2' });
    },
  ],
  [
    'narrative spoiler without its flag',
    (p) => {
      p.documents.find((d) => d.id === p.events[0].publish[0]).requiredFlags = [];
    },
  ],
  [
    'update before publication',
    (p) => {
      p.documents[0].updatedAt = p.documents[0].publishedAt - 1;
    },
  ],
];
for (const [label, mutate] of invalid)
  test(`rejects ${label}`, () => {
    const pack = buildPack();
    mutate(pack);
    assert.throws(() => validateWebPack(pack));
  });
