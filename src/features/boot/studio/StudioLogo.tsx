import { studioLogo } from '../boot-assets';

export function StudioLogo() {
  return (
    <img
      className="studio-logo"
      src={studioLogo}
      width="160"
      height="160"
      alt="Studio Bechi Games"
      draggable={false}
    />
  );
}
