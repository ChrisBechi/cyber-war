import { useLayoutEffect, useRef, useState } from 'react';
import type { CSSProperties } from 'react';
import { bootLogRows } from './boot-log';

export function HackerCodeRain() {
  const viewport = useRef<HTMLDivElement>(null);
  const measure = useRef<HTMLSpanElement>(null);
  const [layout, setLayout] = useState({ first: 31, second: 27, columns: 160 });
  useLayoutEffect(() => {
    const element = viewport.current;
    if (!element) {
      return;
    }
    const resize = () => {
      const style = getComputedStyle(element);
      const lineHeight = Number.parseFloat(style.lineHeight) || 26.4;
      const charWidth = (measure.current?.getBoundingClientRect().width ?? 0) / 20 || 9.6;
      const height = element.clientHeight || window.innerHeight - 44;
      const width = element.clientWidth || window.innerWidth - 60;
      const next = {
        first: Math.max(8, Math.floor(height / lineHeight)),
        second: Math.max(5, Math.floor((height - 104) / lineHeight)),
        columns: Math.floor(width / charWidth),
      };
      setLayout((previous) =>
        previous.first === next.first &&
        previous.second === next.second &&
        previous.columns === next.columns
          ? previous
          : next,
      );
    };
    resize();
    window.addEventListener('resize', resize);
    return () => window.removeEventListener('resize', resize);
  }, []);
  const total = layout.first + layout.second;
  const lines = bootLogRows(total, layout.columns);
  const followAt = (layout.first / (total - 1)) * 3100;
  return (
    <div ref={viewport} className="hacker-code" aria-hidden="true">
      <span ref={measure} className="boot-log-measure">
        00000000000000000000
      </span>
      <div
        className="boot-log-track"
        style={
          {
            '--follow-delay': `${followAt}ms`,
            '--follow-duration': `${3300 - followAt}ms`,
          } as CSSProperties
        }
      >
        {[layout.first, layout.second].map((count, page) => (
          <div
            className="boot-log-page"
            data-boot-page={page + 1}
            key={page}
            style={{ '--log-rows': count } as CSSProperties}
          >
            {lines
              .slice(page ? layout.first : 0, page ? total : layout.first)
              .map((text, index) => {
                const position = (page ? layout.first : 0) + index;
                const timestamp = text.match(/^\[.*?\]/)?.[0] ?? '';
                return (
                  <div
                    className="boot-log-line"
                    key={position}
                    style={
                      { '--log-delay': `${(position * 3100) / (total - 1)}ms` } as CSSProperties
                    }
                  >
                    <span>{timestamp}</span>
                    {text.slice(timestamp.length)}
                  </div>
                );
              })}
          </div>
        ))}
      </div>
    </div>
  );
}
