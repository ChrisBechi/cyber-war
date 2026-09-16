import { useCallback, useEffect, useRef, useState } from 'react';
import { pageSchema } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { canonicalAddress, isOnionAddress, newTab, startNavigation } from './browser-model';
import type { BrowserTab } from './browser-model';
import type { BrowserRequest } from './browser-inspection';
import { loadBrowserFile, type BrowserLocalFile } from './browser-local-file';

export function useBrowserTabs(initialAddress?: string, navigationId?: number, torOnly = false) {
  const [tabs, setTabs] = useState<BrowserTab[]>(() => [newTab(1)]);
  const [activeId, setActiveId] = useState(1);
  const [history, setHistory] = useState<{ address: string; title: string }[]>([]);
  const [requests, setRequests] = useState<BrowserRequest[]>([]);
  const current = useRef(tabs);
  const active = useRef(activeId);
  const nextId = useRef(2);
  const requestId = useRef(0);
  const inspectionId = useRef(0);
  const mounted = useRef(true);
  const beginRequest = useCallback((tabId: number, address: string, operation: string) => {
    const id = ++inspectionId.current;
    const started = performance.now();
    setRequests((items) =>
      [
        ...items,
        {
          id,
          tabId,
          address,
          operation,
          startedAt: Date.now(),
          status: 'pending' as const,
          durationMs: null,
          error: '',
        },
      ].slice(-100),
    );
    return (status: BrowserRequest['status'], error = '') => {
      if (!mounted.current) {
        return;
      }
      setRequests((items) =>
        items.map((item) =>
          item.id === id
            ? {
                ...item,
                status,
                error,
                durationMs: Math.max(0, Math.round(performance.now() - started)),
              }
            : item,
        ),
      );
    };
  }, []);
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
      const isFile = /^file:/i.test(normalized);
      const finishRequest = beginRequest(
        tabId,
        normalized,
        isFile ? 'Arquivo local' : 'Navigation',
      );
      if (torOnly && !isFile && !isOnionAddress(normalized)) {
        finishRequest('blocked', 'Tor Browser só acessa Onion Services v3 (.onion).');
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
      const loading = isFile
        ? loadBrowserFile(normalized)
        : perform('browser_navigate', { address }, pageSchema).then((page) => ({
            page,
            localFile: undefined as BrowserLocalFile | undefined,
          }));
      void loading
        .then(({ page, localFile }) => {
          finishRequest('success');
          if (
            !mounted.current ||
            !current.current.some((tab) => tab.id === tabId && tab.request === token)
          ) {
            return;
          }
          change((tabs) =>
            tabs.map((tab) =>
              tab.id === tabId ? { ...tab, page, localFile, loading: false } : tab,
            ),
          );
          setHistory((old) =>
            [
              { address: normalized, title: page.title },
              ...old.filter((item) => item.address !== normalized),
            ].slice(0, 50),
          );
        })
        .catch((error: unknown) => {
          finishRequest('error', String(error).replace(/^Error:\s*/, ''));
          // Navigation failures belong to their tab, not the desktop-wide toast.
          if (useGame.getState().error === String(error)) {
            useGame.getState().clearError();
          }
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
    [change, torOnly, beginRequest],
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
    requests,
    beginRequest,
    clearRequests: () =>
      setRequests((items) => items.filter((item) => item.tabId !== active.current)),
    change,
    select,
    add,
    close,
    navigate,
    clearHistory: () => setHistory([]),
  };
}
