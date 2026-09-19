// Explicit development harness: real Browser, Rust IPC and isolated SQLite saves.
import { useCallback, useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { Browser } from '../../desktop/Browser';
import { emptySchema, pageSchema, request, worldSchema } from '../../../lib/api';
import { perform, useGame } from '../../../lib/game-store';
import { useWebClock } from './use-web-clock';
import { installListenerMeter } from './qa-listener-meter';

const routes = [
  'goggle.com/search?q=roteador',
  'linkup.com',
  'linkup.com/pessoas/nara-campos',
  'linkup.com/publicacoes/nara-campos-caderno',
  'feiralivre.com',
  'feiralivre.com/anuncios/radio-aurora',
  'shopnow.com/produto/pato-de-borracha',
  'cozinhafacil.com/receitas/pizza-caseira',
  'educamais.com/cursos/javascript',
  'memoria.web',
  'goggle.com/search?q=pato%20de%20boraxa',
  'techbyte.com',
];
const runId = new Date().toISOString();
const listeners = installListenerMeter();
const options = new URLSearchParams(location.hash.split('?')[1] ?? '');
const pause = (ms: number) => new Promise<void>((resolve) => window.setTimeout(resolve, ms));
const percentiles = (values: number[]) => {
  const sorted = [...values].sort((a, b) => a - b);
  return {
    count: values.length,
    p50: sorted[Math.floor(sorted.length * 0.5)] ?? 0,
    p95: sorted[Math.floor(sorted.length * 0.95)] ?? 0,
    max: sorted.at(-1) ?? 0,
  };
};

export function WebSoakQA() {
  const [ready, setReady] = useState(false);
  const [running, setRunning] = useState(false);
  const [step, setStep] = useState(0);
  const [mount, setMount] = useState(0);
  const [shown, setShown] = useState(true);
  const [status, setStatus] = useState('Preparando cinco slots de QA…');
  const [minutes, setMinutes] = useState(
    Math.max(1, Math.min(180, Number(options.get('minutes')) || 120)),
  );
  const autoStarted = useRef(false);
  const root = useRef<HTMLDivElement>(null);
  const cancelled = useRef(false);
  useWebClock(ready);
  useEffect(() => {
    let active = true;
    void request('web_qa_prepare', {}, z.unknown())
      .then(async () => {
        await useGame.getState().refresh();
        if (active) {
          setReady(true);
          setStatus('Pronto. Banco isolado; nenhuma chamada simulada.');
        }
      })
      .catch((error: unknown) => {
        if (active) {
          setStatus(String(error));
        }
      });
    return () => {
      active = false;
      cancelled.current = true;
    };
  }, []);
  const run = useCallback(async () => {
    if (running || !ready) {
      return;
    }
    cancelled.current = false;
    setRunning(true);
    const started = performance.now();
    const latencies: number[] = [],
      frameTimes: number[] = [];
    const errors: string[] = [];
    let iterations = 0,
      slot = 1,
      actions = 0;
    let previousFrame = performance.now(),
      frames = 0,
      frameRequest = 0;
    const frame = (now: number) => {
      if (frameTimes.length < 7200) {
        frameTimes.push(now - previousFrame);
      }
      previousFrame = now;
      frames++;
      frameRequest = requestAnimationFrame(frame);
    };
    frameRequest = requestAnimationFrame(frame);
    const sample = async (state: string) => {
      const memory = performance as Performance & {
        memory?: { usedJSHeapSize: number; totalJSHeapSize: number };
      };
      return request(
        'web_qa_report',
        {
          sample: {
            runId,
            state,
            elapsedSeconds: (performance.now() - started) / 1000,
            plannedMinutes: minutes,
            iterations,
            slot,
            actions,
            errors,
            latencyMs: percentiles(latencies),
            frameMs: percentiles(frameTimes),
            frames,
            globalListeners: listeners(),
            domNodes: document.getElementsByTagName('*').length,
            browserWidth:
              root.current?.querySelector('.browser-shell')?.getBoundingClientRect().width ?? 0,
            heap: memory.memory
              ? { used: memory.memory.usedJSHeapSize, total: memory.memory.totalJSHeapSize }
              : null,
            visibility: document.visibilityState,
            userAgent: navigator.userAgent,
          },
        },
        z.unknown(),
      );
    };
    try {
      await sample('started');
      while (!cancelled.current && performance.now() - started < minutes * 60_000) {
        const at = performance.now();
        const next = iterations;
        setStep(next);
        await pause(250);
        while (
          (useGame.getState().busy || root.current?.querySelector('.browser-loading')) &&
          performance.now() - at < 10_000
        ) {
          await pause(100);
        }
        await pause(150);
        const problem =
          useGame.getState().error || root.current?.querySelector('[role="alert"]')?.textContent;
        if (problem) {
          throw new Error(problem);
        }
        if (root.current?.querySelector('.browser-loading')) {
          throw new Error('Navigation exceeded 10 seconds');
        }
        if (root.current?.querySelector('.web-not-found')) {
          throw new Error(`Unexpected missing page: ${routes[next % routes.length]}`);
        }
        const browserWidth =
          root.current?.querySelector('.browser-shell')?.getBoundingClientRect().width ?? 0;
        if (browserWidth < 1000) {
          throw new Error(`QA browser viewport is too narrow: ${browserWidth}`);
        }
        if (!root.current?.querySelector('.virtual-web, .goggle-site, .goggle-page')) {
          // Goggle has its own layout; a populated browser page must still be visible.
          if (!root.current?.querySelector('h1, [role="search"]')) {
            throw new Error('Rendered page is empty');
          }
        }
        latencies.push(performance.now() - at);
        iterations++;
        if (iterations % 12 === 0) {
          const address = 'linkup.com';
          await perform(
            'web_interact',
            {
              id: 'web-linkup-home',
              action: 'post',
              text: `QA ${slot} ciclo ${iterations}`,
              address,
            },
            pageSchema,
          );
          actions++;
          await sample('running');
          latencies.length = 0;
          frameTimes.length = 0;
          setShown(false);
          await pause(200);
          await perform('end_session', {}, emptySchema);
          slot = (slot % 5) + 1;
          useGame.setState({ slot });
          await perform('load_slot', { slotIndex: slot, manual: false }, worldSchema);
          await perform('session_start', {}, worldSchema);
          await sample('unmounted');
          setMount((old) => old + 1);
          setShown(true);
        }
        setStatus(
          `Em execução · ${iterations} navegações · slot ${slot} · ${Math.floor((performance.now() - started) / 60_000)}/${minutes} min`,
        );
        await pause(Math.max(0, 5000 - (performance.now() - at)));
      }
      await perform('end_session', {}, emptySchema);
      await sample(cancelled.current ? 'stopped' : 'completed');
      setStatus(
        cancelled.current ? 'Interrompido. Relatório salvo.' : 'Concluído. Relatório salvo.',
      );
    } catch (error) {
      errors.push(String(error));
      await sample('failed').catch(() => undefined);
      setStatus(`Falha: ${String(error)}`);
    } finally {
      cancelAnimationFrame(frameRequest);
      setRunning(false);
    }
  }, [ready, running, minutes]);
  useEffect(() => {
    if (ready && !autoStarted.current && options.get('auto') === '1') {
      autoStarted.current = true;
      void run();
    }
  }, [ready, run]);
  return (
    <div
      style={{
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
        background: '#fff',
        color: '#20252b',
      }}
    >
      <header
        style={{ padding: 12, display: 'flex', gap: 16, alignItems: 'center', flexWrap: 'wrap' }}
      >
        <strong>Internet virtual · QA nativo</strong>
        <span role="status">{status}</span>
        <label>
          Minutos{' '}
          <input
            type="number"
            min={1}
            max={180}
            value={minutes}
            disabled={running}
            onChange={(e) => setMinutes(Math.max(1, Math.min(180, Number(e.target.value))))}
          />
        </label>
        <button disabled={!ready || running} onClick={() => void run()}>
          Iniciar ensaio
        </button>
        <button
          disabled={!running}
          onClick={() => {
            cancelled.current = true;
          }}
        >
          Parar ensaio
        </button>
      </header>
      <div ref={root} style={{ flex: 1, minHeight: 0 }}>
        {ready && shown && (
          <Browser key={mount} initialAddress={routes[step % routes.length]} navigationId={step} />
        )}
      </div>
    </div>
  );
}
