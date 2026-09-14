import { audioManager } from '../../../lib/audio-manager';
import { pressStartLogo } from '../boot-assets';
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
      <img src={pressStartLogo} className="game-logo" alt="CYBER WAR_" />
      <span className="press-start-prompt">CLIQUE PARA INICIAR</span>
      <span className="press-start-credit">STUDIO BECHI GAMES</span>
    </button>
  );
}
