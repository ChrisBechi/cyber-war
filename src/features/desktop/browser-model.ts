import { z } from 'zod';
import type { BrowserPage } from '../../lib/api';

export const preferencesSchema = z.object({
  home: z.string().max(512),
  zoom: z.number().int().min(50).max(200),
  showBookmarks: z.boolean(),
  bookmarks: z
    .array(z.object({ title: z.string().min(1).max(100), address: z.string().max(512) }))
    .max(50),
});
export type BrowserPreferences = z.infer<typeof preferencesSchema>;
export const BLACKWIRE_ONION_ADDRESS =
  'pg6mmjiyjmcrsslvykfwnntlaru7p5svn6y2ymmju6nubxndf4pscryd.onion';
const onionAlphabet = /^[a-z2-7]+$/;
export const defaultPreferences: BrowserPreferences = {
  home: 'https://www.wipedia.org',
  zoom: 100,
  showBookmarks: true,
  bookmarks: [
    'https://www.wipedia.org',
    'https://www.archive.org',
    'https://www.b1.tech',
    'https://www.fakebook.com',
    'https://www.mercado.com.br',
    'https://www.meudominio.com.br',
  ].map((address) => ({
    title: address
      .replace(/^https?:\/\//, '')
      .replace(/^www\./, '')
      .split('.')[0],
    address,
  })),
};
export function readPreferences(raw?: string): BrowserPreferences {
  try {
    return preferencesSchema.parse(JSON.parse(raw ?? ''));
  } catch {
    return structuredClone(defaultPreferences);
  }
}
export type BrowserTab = {
  id: number;
  address: string;
  draft: string;
  history: string[];
  index: number;
  page: BrowserPage | null;
  error: string;
  loading: boolean;
  request: number;
  value: string;
};
export const newTab = (id: number): BrowserTab => ({
  id,
  address: '',
  draft: '',
  history: [''],
  index: 0,
  page: null,
  error: '',
  loading: false,
  request: 0,
  value: '',
});
export const addressKey = (address: string) =>
  address
    .trim()
    .replace(/^https?:\/\//i, '')
    .replace(/\/$/, '')
    .replace(/^www\./i, '')
    .toLowerCase();

export function canonicalAddress(address: string) {
  const value = address.trim();
  if (!value) {
    return value;
  }
  const withoutProtocol = value.replace(/^https?:\/\//i, '');
  const slash = withoutProtocol.indexOf('/');
  const rawHost = slash < 0 ? withoutProtocol : withoutProtocol.slice(0, slash);
  const path = slash < 0 ? '' : withoutProtocol.slice(slash).replace(/\/+$/, '');
  const host = rawHost
    .replace(/^www\./i, '')
    .replace(/\.$/, '')
    .toLowerCase();
  const canonicalHost = host.endsWith('.onion') ? host : `www.${host}`;
  const protocol = /^http:\/\//i.test(value) ? 'http' : 'https';
  return `${protocol}://${canonicalHost}${path}`;
}

export function isOnionAddress(address: string) {
  const host = address
    .trim()
    .replace(/^https?:\/\//i, '')
    .split('/')[0]
    .replace(/\.$/, '')
    .toLowerCase();
  const label = host.endsWith('.onion') ? host.slice(0, -'.onion'.length) : '';
  return label.length === 56 && onionAlphabet.test(label);
}
export function startNavigation(
  tab: BrowserTab,
  address: string,
  request: number,
  index?: number,
): BrowserTab {
  const history =
    index !== undefined
      ? tab.history
      : [...tab.history.slice(0, tab.index + 1), address].slice(-100);
  return {
    ...tab,
    address,
    draft: address,
    request,
    loading: !!address,
    error: '',
    page: null,
    value: '',
    history,
    index: index ?? history.length - 1,
  };
}
