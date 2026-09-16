import { audioManager } from '../../../lib/audio-manager';
import { useAdvance } from '../use-advance';

export function PressStartScreen({ onStart }: { onStart: () => void }) {
  const { skip } = useAdvance(
    () => {
      audioManager.play('menu-select');
      onStart();
    },
    0,
    ['Enter', ' '],
  );
  return (
    <button className="boot-screen press-start" onClick={skip} aria-label="Clique para iniciar">
      <span className="press-start-title">
        CYBER WAR
        <span className="game-title-cursor" aria-hidden="true">
          _
        </span>
      </span>
      <span className="press-start-prompt">CLIQUE PARA INICIAR</span>
      <span className="press-start-credit">STUDIO BECHI GAMES</span>
    </button>
  );
}
