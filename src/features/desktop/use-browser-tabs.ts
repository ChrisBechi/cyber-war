import { useCallback, useEffect, useRef, useState } from 'react';
import { pageSchema } from '../../lib/api';
import { perform } from '../../lib/game-store';
import { canonicalAddress, isOnionAddress, newTab, startNavigation } from './browser-model';
import type { BrowserTab } from './browser-model';

export function useBrowserTabs(initialAddress?: string, navigationId?: number, torOnly = false) {
  const [tabs, setTabs] = useState<BrowserTab[]>(() => [newTab(1)]);
  const [activeId, setActiveId] = useState(1);
  const [history, setHistory] = useState<{ address: string; title: string }[]>([]);
  const current = useRef(tabs);
  const active = useRef(activeId);
  const nextId = useRef(2);
  const requestId = useRef(0);
  const mounted = useRef(true);
  const change = useCallback((update: (tabs: BrowserTab[]) => BrowserTab[]) => {
    current.current = update(current.current);
    setTabs(current.current);
  }, []);
  const select = useCallback((id: number) => {
    active.current = id;
    setActiveId(id);
  }, []);
  const navigate = useCallback(
    (address: string, tabId = active.current, index?: number) => {
      const normalized = canonicalAddress(address);
      if (normalized.length > 512) {
        change((tabs) =>
          tabs.map((tab) => (tab.id === tabId ? { ...tab, error: 'Endereço muito longo.' } : tab)),
        );
        return;
      }
      const token = ++requestId.current;
      change((tabs) =>
        tabs.map((tab) =>
          tab.id === tabId ? startNavigation(tab, normalized, token, index) : tab,
        ),
      );
      if (!normalized) {
        return;
      }
      if (torOnly && !isOnionAddress(normalized)) {
        change((tabs) =>
          tabs.map((tab) =>
            tab.id === tabId
              ? {
                  ...tab,
                  loading: false,
                  error: 'Tor Browser só acessa Onion Services v3 (.onion).',
                }
              : tab,
          ),
        );
        return;
      }
      void perform('browser_navigate', { address }, pageSchema)
        .then((page) => {
          if (
            !mounted.current ||
            !current.current.some((tab) => tab.id === tabId && tab.request === token)
          ) {
            return;
          }
          change((tabs) =>
            tabs.map((tab) => (tab.id === tabId ? { ...tab, page, loading: false } : tab)),
          );
          setHistory((old) =>
            [
              { address: normalized, title: page.title },
              ...old.filter((item) => item.address !== normalized),
            ].slice(0, 50),
          );
        })
        .catch((error: unknown) => {
          if (mounted.current) {
            change((tabs) =>
              tabs.map((tab) =>
                tab.id === tabId && tab.request === token
                  ? { ...tab, error: String(error), loading: false, draft: address }
                  : tab,
              ),
            );
          }
        });
    },
    [change, torOnly],
  );
  const add = useCallback(
    (address = '') => {
      if (current.current.length >= 16) {
        return;
      }
      const id = nextId.current++;
      change((tabs) => [...tabs, newTab(id)]);
      select(id);
      if (address) {
        navigate(address, id);
      }
    },
    [change, select, navigate],
  );
  const close = (id: number) => {
    const index = current.current.findIndex((tab) => tab.id === id);
    if (index < 0) {
      return;
    }
    const remaining = current.current.filter((tab) => tab.id !== id);
    if (!remaining.length) {
      const tab = newTab(nextId.current++);
      change(() => [tab]);
      select(tab.id);
    } else {
      change(() => remaining);
      if (active.current === id) {
        select(remaining[Math.min(index, remaining.length - 1)].id);
      }
    }
  };
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    if (initialAddress) {
      navigate(initialAddress);
    }
  }, [initialAddress, navigationId, navigate]);
  const tab = tabs.find((tab) => tab.id === activeId) ?? tabs[0];
  return {
    tabs,
    tab,
    history,
    change,
    select,
    add,
    close,
    navigate,
    clearHistory: () => setHistory([]),
  };
}
