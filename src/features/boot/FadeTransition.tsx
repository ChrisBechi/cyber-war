import { useEffect, useState } from 'react';
import type { ReactNode } from 'react';

export function FadeTransition<T extends string>({
  stage,
  render,
  duration = 500,
}: {
  stage: T;
  render: (stage: T) => ReactNode;
  duration?: number;
}) {
  const [shown, setShown] = useState(stage);
  const leaving = shown !== stage;
  useEffect(() => {
    if (!leaving) {
      return;
    }
    const timer = window.setTimeout(() => setShown(stage), duration);
    return () => window.clearTimeout(timer);
  }, [stage, leaving, duration]);
  return (
    <div
      className={`fade-transition ${leaving ? 'is-leaving' : ''}`}
      style={{ transitionDuration: `${duration}ms` }}
      inert={leaving}
    >
      <div key={shown} className="fade-transition-content">
        {render(shown)}
      </div>
    </div>
  );
}
