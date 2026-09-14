import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { apps, useWindows, windowApp } from '../../lib/window-store';
import type { WindowState } from '../../lib/window-store';
import { AppIcon } from './AppIcon';

export function AppWindow({ model, children }: { model: WindowState; children: ReactNode }) {
  const app = apps[windowApp(model.id)];
  const { focus, close, update, workspace } = useWindows();
  const container = useRef<HTMLElement>(null);
  const [interacting, setInteracting] = useState(false);
  const drag = useRef<{ x: number; y: number; left: number; top: number } | null>(null);
  const shown = !model.minimized && model.workspace === workspace;
  useEffect(() => {
    if (!model.fullscreen) {
      return;
    }
    const exit = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        update(model.id, { fullscreen: false });
      }
    };
    window.addEventListener('keydown', exit);
    return () => window.removeEventListener('keydown', exit);
  }, [model.id, model.fullscreen, update]);
  const stopInteraction = () => {
    drag.current = null;
    setInteracting(false);
  };
  useLayoutEffect(() => {
    const element = container.current;
    if (!element || !model.minimized) {
      return;
    }
    const target = document.querySelector<HTMLElement>(`[data-window-task="${model.id}"]`);
    const layer = element.parentElement?.getBoundingClientRect();
    const button = target?.getBoundingClientRect();
    if (layer && button) {
      const centerX = layer.left + element.offsetLeft + element.offsetWidth / 2;
      const centerY = layer.top + element.offsetTop + element.offsetHeight / 2;
      element.style.setProperty('--minimize-x', `${button.left + button.width / 2 - centerX}px`);
      element.style.setProperty('--minimize-y', `${button.top + button.height / 2 - centerY}px`);
    }
  }, [model.id, model.minimized, model.maximized, model.workspace, workspace]);
  return (
    <section
      ref={container}
      data-window-id={model.id}
      data-window-app={windowApp(model.id)}
      role="dialog"
      aria-label={app.title}
      aria-hidden={!shown}
      inert={!shown}
      className={`window ${model.fullscreen ? 'is-fullscreen' : ''} ${model.maximized ? 'maximized' : ''} ${model.minimized ? 'is-minimized' : ''} ${interacting ? 'is-interacting' : ''}`}
      style={{
        left: model.maximized ? 0 : model.x,
        top: model.maximized ? 0 : model.y,
        width: model.maximized ? '100%' : model.width,
        height: model.maximized ? '100%' : model.height,
        zIndex: model.z,
        display: model.workspace === workspace ? 'flex' : 'none',
      }}
      onPointerDown={() => focus(model.id)}
    >
      <header
        className="window__titlebar"
        onDoubleClick={(event) => {
          if (!(event.target as HTMLElement).closest('button')) {
            update(model.id, { maximized: !model.maximized });
          }
        }}
        onPointerDown={(event) => {
          if ((event.target as HTMLElement).closest('button') || model.maximized) {
            return;
          }
          event.currentTarget.setPointerCapture(event.pointerId);
          setInteracting(true);
          drag.current = { x: event.clientX, y: event.clientY, left: model.x, top: model.y };
        }}
        onPointerMove={(event) => {
          if (drag.current) {
            update(model.id, {
              x: Math.max(
                0,
                Math.min(
                  window.innerWidth - 160,
                  drag.current.left + event.clientX - drag.current.x,
                ),
              ),
              y: Math.max(
                0,
                Math.min(
                  window.innerHeight - 120,
                  drag.current.top + event.clientY - drag.current.y,
                ),
              ),
            });
          }
        }}
        onPointerUp={stopInteraction}
        onPointerCancel={stopInteraction}
        onLostPointerCapture={stopInteraction}
      >
        <span>
          {windowApp(model.id) === 'terminal' ? (
            <svg
              className="app-icon"
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
              aria-hidden="true"
            >
              <path d="m3 4 4 4-4 4m6 0h4" />
            </svg>
          ) : (
            <AppIcon name={model.id} />
          )}{' '}
          {app.title}
          {model.asRoot ? ' (root)' : ''}
          {model.path ? ` — ${model.path.split('/').pop() ?? ''}` : ''}
        </span>
        <div>
          <button
            aria-label={`Minimizar ${app.title}`}
            onClick={() => update(model.id, { minimized: true })}
          >
            <svg viewBox="0 0 12 12" aria-hidden="true">
              <path d="M2.5 6.5h7" />
            </svg>
          </button>
          <button
            aria-label={`${model.maximized ? 'Restaurar' : 'Maximizar'} ${app.title}`}
            title={model.maximized ? 'Restaurar tamanho' : 'Maximizar'}
            onClick={() => update(model.id, { maximized: !model.maximized })}
          >
            <svg viewBox="0 0 12 12" aria-hidden="true">
              {model.maximized ? (
                <>
                  <path d="M4.5 4.5v-2h5v5h-2" />
                  <rect x="2.5" y="4.5" width="5" height="5" rx=".5" />
                </>
              ) : (
                <rect x="2.5" y="2.5" width="7" height="7" rx=".5" />
              )}
            </svg>
          </button>
          <button aria-label={`Fechar ${app.title}`} onClick={() => close(model.id)}>
            <svg viewBox="0 0 12 12" aria-hidden="true">
              <path d="m2.5 2.5 7 7m0-7-7 7" />
            </svg>
          </button>
        </div>
      </header>
      <div className="window__content">{children}</div>
      {!model.maximized && (
        <button
          className="resize-handle"
          aria-label="Redimensionar janela"
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            setInteracting(true);
            drag.current = { x: e.clientX, y: e.clientY, left: model.width, top: model.height };
          }}
          onPointerMove={(e) => {
            if (drag.current) {
              update(model.id, {
                width: Math.max(
                  windowApp(model.id) === 'processes' ? 290 : 430,
                  Math.min(
                    window.innerWidth - model.x,
                    drag.current.left + e.clientX - drag.current.x,
                  ),
                ),
                height: Math.max(
                  280,
                  Math.min(
                    window.innerHeight - model.y - 80,
                    drag.current.top + e.clientY - drag.current.y,
                  ),
                ),
              });
            }
          }}
          onPointerUp={stopInteraction}
          onPointerCancel={stopInteraction}
          onLostPointerCapture={stopInteraction}
        >
          ◢
        </button>
      )}
    </section>
  );
}
