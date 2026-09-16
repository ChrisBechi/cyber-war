import { useEffect, useRef, useState } from 'react';
import { archiveSize, cancelArchiveJob, waitArchiveJob } from '../../lib/archive';
import { confirmPackage, inspectPackage, planPackage } from '../../lib/packages';
import type { PackageInspection } from '../../lib/packages';
import { useGame } from '../../lib/game-store';
import { AppIcon } from '../desktop/AppIcon';
import './packages.css';

export function PackageInstaller({ initialPath }: { initialPath?: string }) {
  const revision = useGame((state) => state.revision);
  const [info, setInfo] = useState<PackageInspection | null>(null);
  const [plan, setPlan] = useState<{ id: string; summary: string } | null>(null);
  const [error, setError] = useState('');
  const [log, setLog] = useState('');
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);
  const pending = useRef<string | null>(null);
  const job = useRef<number | null>(null);
  const generation = useRef(0);
  useEffect(() => {
    let current = true;
    if (initialPath) {
      void inspectPackage(initialPath)
        .then((value) => {
          if (current) {
            setInfo(value);
            setError('');
          }
        })
        .catch((reason: unknown) => {
          if (current) {
            setInfo(null);
            setError(String(reason));
          }
        });
    }
    return () => {
      current = false;
    };
  }, [initialPath, revision]);
  useEffect(() => {
    setPlan(null);
    setLog('');
    setProgress(null);
    setBusy(false);
    return () => {
      generation.current += 1;
      if (pending.current) {
        void confirmPackage(pending.current, false).catch((reason: unknown) =>
          useGame.setState({ error: String(reason) }),
        );
      }
      if (job.current) {
        void cancelArchiveJob(job.current).catch((reason: unknown) =>
          useGame.setState({ error: String(reason) }),
        );
      }
      pending.current = null;
      job.current = null;
    };
  }, [initialPath]);
  const prepare = async (operation: 'install' | 'remove' | 'purge') => {
    if (!initialPath) {
      return;
    }
    setBusy(true);
    setError('');
    const current = generation.current;
    try {
      const value = await planPackage(initialPath, operation);
      if (current !== generation.current) {
        await confirmPackage(value.id, false);
        return;
      }
      pending.current = value.id;
      setPlan(value);
    } catch (reason: unknown) {
      if (current === generation.current) {
        setError(String(reason));
      }
    } finally {
      if (current === generation.current) {
        setBusy(false);
      }
    }
  };
  const confirm = async (accept: boolean) => {
    if (!plan) {
      return;
    }
    setBusy(true);
    setError('');
    const current = generation.current;
    try {
      const result = await confirmPackage(plan.id, accept);
      if (current !== generation.current) {
        if (result.job) {
          await cancelArchiveJob(result.job);
        }
        return;
      }
      pending.current = null;
      setPlan(null);
      setLog(result.stdout + result.stderr);
      if (result.job) {
        job.current = result.job;
        const done = await waitArchiveJob(result.job, (value) => {
          if (current === generation.current) {
            setProgress(value.progress);
          }
        });
        if (current !== generation.current) {
          return;
        }
        setLog(done.stdout + done.stderr);
        if (done.exitCode) {
          setError(done.stderr || 'Operação cancelada.');
        }
      } else if (result.exitCode && accept) {
        setError(result.stderr);
      }
    } catch (reason: unknown) {
      if (current === generation.current) {
        setError(String(reason));
      }
    } finally {
      if (current === generation.current) {
        job.current = null;
        setProgress(null);
        setBusy(false);
      }
    }
  };
  return (
    <section className="package-installer">
      <header>
        <AppIcon name="package-installer" size={48} />
        <div>
          <h2>Instalador de pacotes</h2>
          <p>{initialPath ?? 'Abra um arquivo .deb no gerenciador de arquivos.'}</p>
        </div>
      </header>
      {error && (
        <p className="package-error" role="alert">
          {error}
        </p>
      )}
      {info && (
        <>
          <h3>
            {info.name} <small>{info.version}</small>
          </h3>
          <p>{info.description}</p>
          <dl>
            <dt>Arquitetura</dt>
            <dd>{info.architecture}</dd>
            <dt>Estado</dt>
            <dd>{info.status}</dd>
            <dt>Tamanho instalado</dt>
            <dd>{archiveSize(info.installedSize)}</dd>
            <dt>Download</dt>
            <dd>{archiveSize(info.downloadSize)}</dd>
            <dt>Origem</dt>
            <dd>{info.origin}</dd>
            <dt>Dependências</dt>
            <dd>{info.dependencies.join(', ') || 'Nenhuma'}</dd>
          </dl>
          {!plan && (
            <div className="package-actions">
              <button disabled={busy} onClick={() => void prepare('install')}>
                Instalar
              </button>
              <button
                disabled={busy || info.status === 'not-installed'}
                onClick={() => void prepare('remove')}
              >
                Remover
              </button>
              <button
                disabled={busy || info.status === 'not-installed'}
                onClick={() => void prepare('purge')}
              >
                Remover configurações
              </button>
            </div>
          )}
        </>
      )}
      {plan && (
        <section className="package-plan" aria-label="Confirmar alteração de pacotes">
          <h3>Revisar alterações</h3>
          <pre>{plan.summary}</pre>
          <p>A operação será executada como administrador do computador virtual.</p>
          <div className="package-actions">
            <button disabled={busy} onClick={() => void confirm(true)}>
              Confirmar
            </button>
            <button disabled={busy} onClick={() => void confirm(false)}>
              Cancelar
            </button>
          </div>
        </section>
      )}
      {progress !== null && (
        <div role="status">
          <progress max={100} value={progress} /> {progress}%{' '}
          <button
            onClick={() => {
              if (job.current) {
                void cancelArchiveJob(job.current).catch((reason: unknown) =>
                  setError(String(reason)),
                );
              }
            }}
          >
            Cancelar operação
          </button>
        </div>
      )}
      {log && (
        <pre className="package-log" aria-label="Resultado da operação">
          {log}
        </pre>
      )}
    </section>
  );
}
