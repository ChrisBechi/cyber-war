import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { AppIcon } from './AppIcon';
import type { IconName } from './AppIcon';
import type { WindowId } from '../../lib/window-store';
import { useDismissOutside } from '../../lib/use-dismiss-outside';
import './desktop-context.css';

export type ContextItem = null | {
  label: string;
  icon?: IconName | WindowId;
  disabled?: boolean;
  action?: () => void;
  children?: ContextItem[];
};
type Point = { x: number; y: number; leftEdge?: number };

function MenuPanel({
  items,
  point,
  label,
  close,
  back,
  focus = true,
}: {
  items: ContextItem[];
  point: Point;
  label: string;
  close: () => void;
  back?: () => void;
  focus?: boolean;
}) {
  const panel = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState(point);
  const [submenu, setSubmenu] = useState<{ index: number; point: Point; focus: boolean } | null>(
    null,
  );
  useLayoutEffect(() => {
    const element = panel.current;
    if (!element) {
      return;
    }
    const box = element.getBoundingClientRect();
    const x =
      point.x + box.width > window.innerWidth && point.leftEdge !== undefined
        ? point.leftEdge - box.width
        : point.x;
    setPosition({
      x: Math.max(4, Math.min(x, window.innerWidth - box.width - 4)),
      y: Math.max(4, Math.min(point.y, window.innerHeight - box.height - 4)),
    });
    if (focus) {
      element.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus();
    }
  }, [point, focus]);
  const expand = (index: number, button: HTMLElement, keyboard = false) => {
    if (!items[index]?.children) {
      setSubmenu(null);
      return;
    }
    const rect = button.getBoundingClientRect();
    setSubmenu({
      index,
      point: { x: rect.right + 2, y: rect.top, leftEdge: rect.left - 2 },
      focus: keyboard,
    });
  };
  return (
    <>
      <div
        ref={panel}
        className="desktop-context-menu"
        role="menu"
        aria-label={label}
        onScroll={() => setSubmenu(null)}
        style={{ left: position.x, top: position.y }}
        onContextMenu={(e) => e.preventDefault()}
        onKeyDown={(e) => {
          e.stopPropagation();
          const buttons = Array.from(
            panel.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? [],
          );
          const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
          if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(e.key)) {
            e.preventDefault();
            setSubmenu(null);
            const next =
              e.key === 'Home'
                ? 0
                : e.key === 'End'
                  ? buttons.length - 1
                  : (index + (e.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length;
            buttons[next]?.focus();
          } else if (e.key === 'Escape' || e.key === 'Tab') {
            e.preventDefault();
            close();
          } else if (e.key === 'ArrowLeft' && back) {
            e.preventDefault();
            back();
          } else if (e.key === 'ArrowRight') {
            e.preventDefault();
            const button = buttons[index];
            if (button) {
              expand(Number(button.dataset.index), button, true);
            }
          }
        }}
      >
        {items.map((item, index) =>
          item ? (
            <button
              type="button"
              key={item.label}
              role="menuitem"
              title={item.label}
              data-index={index}
              disabled={item.disabled}
              aria-haspopup={item.children ? 'menu' : undefined}
              aria-expanded={item.children ? submenu?.index === index : undefined}
              className={submenu?.index === index ? 'context-expanded' : undefined}
              onMouseEnter={(e) => expand(index, e.currentTarget)}
              onClick={(e) => {
                if (item.children) {
                  expand(index, e.currentTarget, true);
                } else {
                  close();
                  item.action?.();
                }
              }}
            >
              <span className="context-icon">
                {item.icon && <AppIcon name={item.icon} size={16} />}
              </span>
              <span>{item.label}</span>
              {item.children && (
                <span className="context-arrow" aria-hidden="true">
                  ▸
                </span>
              )}
            </button>
          ) : (
            <div key={`separator-${index}`} role="separator" />
          ),
        )}
      </div>
      {submenu && items[submenu.index]?.children && (
        <MenuPanel
          key={submenu.index}
          items={items[submenu.index]!.children!}
          point={submenu.point}
          label={items[submenu.index]!.label}
          focus={submenu.focus}
          close={close}
          back={() => {
            panel.current
              ?.querySelector<HTMLButtonElement>(`button[data-index="${submenu.index}"]`)
              ?.focus();
            setSubmenu(null);
          }}
        />
      )}
    </>
  );
}

export function DesktopContextMenu({
  items,
  x,
  y,
  onClose,
}: {
  items: ContextItem[];
  x: number;
  y: number;
  onClose: () => void;
}) {
  const container = useRef<HTMLDivElement>(null);
  const origin = useRef(document.activeElement);
  const [point] = useState({ x, y });
  useDismissOutside(true, (target) => !!container.current?.contains(target), onClose);
  useEffect(() => {
    window.addEventListener('resize', onClose);
    const previous = origin.current;
    return () => {
      window.removeEventListener('resize', onClose);
      if (previous instanceof HTMLElement && previous.isConnected) {
        previous.focus();
      }
    };
  }, [onClose]);
  return (
    <div ref={container}>
      <MenuPanel items={items} point={point} label="Menu da área de trabalho" close={onClose} />
    </div>
  );
}
