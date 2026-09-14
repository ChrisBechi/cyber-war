import { useEffect, useState } from 'react';
import { audioManager } from '../../../lib/audio-manager';
import { useAdvance } from '../use-advance';
import { StudioLogo } from './StudioLogo';
import { HackerCodeRain } from './HackerCodeRain';

export function StudioIntro({ onFinish }: { onFinish: () => void }) {
  const [phase, setPhase] = useState('black');
  const [command, setCommand] = useState('');
  const { finish, skip } = useAdvance(onFinish, 1500);
  useEffect(() => {
    const tracks: string[] = [audioManager.music('studio-intro-ambience')];
    let typing = '';
    const cues: [number, () => void][] = [
      [
        500,
        () => {
          setPhase('code');
          typing = audioManager.play('keyboard-loop', 'sfx', true);
          tracks.push(typing);
        },
      ],
      [
        2000,
        () => {
          setPhase('reveal');
          tracks.push(audioManager.play('logo-reveal'));
        },
      ],
      [3500, () => setPhase('stable')],
      [4200, () => setPhase('clear-command')],
      [
        5900,
        () => {
          setPhase('clear');
          audioManager.stop(typing, 80);
          tracks.push(audioManager.play('terminal-clear'));
        },
      ],
      [
        6200,
        () => {
          setPhase('out');
          audioManager.stop(tracks[0], 700);
        },
      ],
      [7000, finish],
    ];
    const timers = cues.map(([time, cue]) => window.setTimeout(cue, time));
    const commandText = './cyber-war --start';
    [...commandText].forEach((_, index) =>
      timers.push(
        window.setTimeout(
          () => {
            setCommand(commandText.slice(0, index + 1));
            audioManager.play('ui-key');
          },
          4350 + index * 65,
        ),
      ),
    );
    return () => {
      timers.forEach((timer) => window.clearTimeout(timer));
      tracks.forEach((track) => audioManager.stop(track, 250));
    };
  }, [finish]);
  return (
    <section
      className={`boot-screen studio-intro phase-${phase}`}
      aria-label="Abertura Studio Bechi Games"
      onClick={skip}
    >
      <HackerCodeRain />
      <div className="studio-signature">
        <StudioLogo />
        <h1>STUDIO BECHI GAMES</h1>
        <span className="studio-rule" />
      </div>
      <div className="studio-clear" aria-hidden="true">
        <div className="studio-shell-prompt">┌──(root㉿lifeos)-[~]</div>
        <div>
          └─# {command}
          <span className="studio-shell-cursor">▌</span>
        </div>
        {phase === 'clear' && (
          <div className="studio-shell-result">[ OK ] Starting Cyber War session…</div>
        )}
      </div>
    </section>
  );
}
