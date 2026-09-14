import { useEffect, useRef, useState } from 'react';
import { Desktop } from '../desktop/Desktop';
import { OpeningDirector } from './OpeningDirector';
import type { OpeningFrame } from './OpeningDirector';
import { CodeRainTitle } from './title/CodeRainTitle';
import './opening.css';

export function OpeningScenarioMode() {
  const surface = useRef<HTMLDivElement>(null);
  const director = useRef<OpeningDirector | null>(null);
  const [ready, setReady] = useState(false);
  const [frame, setFrame] = useState<OpeningFrame>({
    time: 0,
    scene: 'opening/desktop',
    title: false,
    done: false,
  });
  const [running, setRunning] = useState(false);
  const capture = new URLSearchParams(window.location.search).has('capture');
  useEffect(() => {
    if (!surface.current) {
      return;
    }
    const instance = new OpeningDirector(surface.current, setFrame);
    director.current = instance;
    setReady(true);
    return () => {
      instance.dispose();
      director.current = null;
    };
  }, []);
  const start = () => {
    if (!running) {
      setRunning(true);
      director.current?.start();
    }
  };
  return (
    <div
      ref={surface}
      className="opening-scenario"
      data-opening-state={frame.scene}
      data-opening-done={frame.done}
      data-opening-ready={ready}
    >
      {ready && <Desktop onMenu={() => undefined} presentation />}
      <svg
        className="scenario-cursor"
        width="21"
        height="27"
        viewBox="0 0 21 27"
        aria-hidden="true"
      >
        <path d="M2 1v22l5-6 4 9 4-2-4-8h8Z" fill="#eee" stroke="#151515" strokeWidth="1.5" />
      </svg>
      <div className="scenario-black" />
      {frame.title && <CodeRainTitle elapsed={(frame.time - 85000) * 3.9} />}
      {!running && (
        <button className="scenario-start" onClick={start}>
          Gravar OpeningScenarioMode
        </button>
      )}
      {!capture && running && (
        <output className="scenario-debug">
          {frame.scene} · {(frame.time / 1000).toFixed(1)}s
        </output>
      )}
      {frame.done && (
        <script type="application/json" id="opening-capture-manifest">
          {JSON.stringify({
            duration: 90000,
            events: director.current?.executed,
            audio: director.current?.audio,
            failures: director.current?.failures,
          })}
        </script>
      )}
    </div>
  );
}
