import { useCallback, useState } from 'react';
import type { AppSettings } from '../../lib/app-settings';
import { MainMenu } from '../menu/MainMenu';
import { FadeTransition } from './FadeTransition';
import { initialBootStage, nextBootStage } from './boot.types';
import { StudioIntro } from './studio/StudioIntro';
import { OpeningCinematic } from '../opening/OpeningCinematic';
import { PressStartScreen } from './press-start/PressStartScreen';

export function BootFlow({
  settings,
  returnToMenu = false,
  onPlay,
}: {
  settings: AppSettings;
  returnToMenu?: boolean;
  onPlay: (newGame: boolean, needsLogin?: boolean) => void;
}) {
  const [stage, setStage] = useState(() =>
    returnToMenu ? ('main_menu' as const) : initialBootStage(settings),
  );
  const advance = useCallback(
    () => setStage((current) => nextBootStage(current, settings.skipTrailer)),
    [settings.skipTrailer],
  );
  return (
    <div className="boot-flow" data-boot-stage={stage}>
      <FadeTransition
        stage={stage}
        render={(shown) => {
          switch (shown) {
            case 'studio_intro':
              return <StudioIntro onFinish={advance} />;
            case 'opening_cinematic':
              return <OpeningCinematic onFinish={advance} />;
            case 'press_start':
              return <PressStartScreen onStart={advance} />;
            case 'main_menu':
              return <MainMenu onPlay={onPlay} />;
          }
        }}
      />
    </div>
  );
}
