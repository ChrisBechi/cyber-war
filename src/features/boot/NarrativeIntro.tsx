import { useAdvance } from './use-advance';

export function NarrativeIntro({ onFinish }: { onFinish: () => void }) {
  const { skip } = useAdvance(onFinish, 500, ['Enter', ' ']);
  return (
    <button className="boot-screen narrative-intro" onClick={skip} aria-label="Entrar na história">
      <span className="front-kicker">01 / CURIOSIDADE</span>
      <span className="narrative-text">
        Você começou querendo descobrir
        <br />
        como as coisas funcionavam.
      </span>
      <span className="narrative-caption">17 ANOS. UM COMPUTADOR. UM NICKNAME.</span>
      <span className="press-start-prompt">ENTRAR NA HISTÓRIA →</span>
    </button>
  );
}
