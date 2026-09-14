import { StrictMode } from 'react';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { StudioIntro } from './studio/StudioIntro';
import { OpeningCinematic } from '../opening/OpeningCinematic';
import { PressStartScreen } from './press-start/PressStartScreen';
import { BootFlow } from './BootFlow';
import { defaultAppSettings } from '../../lib/app-settings';

const attachAudio = vi.hoisted(() => vi.fn(() => vi.fn()));

vi.mock('../../lib/audio-manager', () => ({
  audioManager: {
    music: vi.fn(() => 'music'),
    play: vi.fn(() => 'cue'),
    stop: vi.fn(),
    unlock: vi.fn(),
    attach: attachAudio,
  },
}));
vi.mock('../menu/MainMenu', () => ({ MainMenu: () => <div>MENU PRINCIPAL</div> }));
beforeEach(() => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'performance'] });
  vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue();
  vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(() => undefined);
});
afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});
const advance = (ms: number) => act(() => vi.advanceTimersByTimeAsync(ms));

describe('StudioIntro', () => {
  it('completes the timeline automatically once under StrictMode', async () => {
    const finish = vi.fn();
    render(
      <StrictMode>
        <StudioIntro onFinish={finish} />
      </StrictMode>,
    );
    expect(screen.getByLabelText('Abertura Studio Bechi Games')).toHaveClass('phase-black');
    await advance(5900);
    expect(screen.getByLabelText('Abertura Studio Bechi Games')).toHaveClass('phase-clear');
    expect(screen.getByText(/cyber-war --start/)).toBeInTheDocument();
    await advance(1100);
    expect(finish).toHaveBeenCalledTimes(1);
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(finish).toHaveBeenCalledTimes(1);
  });
  it('blocks early skips and never advances twice after a valid skip', async () => {
    const finish = vi.fn();
    render(<StudioIntro onFinish={finish} />);
    fireEvent.click(screen.getByLabelText('Abertura Studio Bechi Games'));
    await advance(1499);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(finish).not.toHaveBeenCalled();
    await advance(1);
    fireEvent.keyDown(window, { key: 'Escape' });
    fireEvent.click(screen.getByLabelText('Abertura Studio Bechi Games'));
    await advance(7000);
    expect(finish).toHaveBeenCalledTimes(1);
  });
});
describe('OpeningCinematic', () => {
  it('keeps music and effects independent and enforces the 90-second playback boundary', () => {
    const finish = vi.fn();
    const { container, unmount } = render(
      <OpeningCinematic onFinish={finish} sources={['/opening.mp4']} />,
    );
    const video = container.querySelector('video')!;
    const effects = container.querySelector('audio')!;
    expect(attachAudio).toHaveBeenCalledWith(video, 'music');
    expect(attachAudio).toHaveBeenCalledWith(effects, 'sfx');
    video.currentTime = 89.9;
    fireEvent.timeUpdate(video);
    expect(finish).not.toHaveBeenCalled();
    video.currentTime = 90;
    fireEvent.timeUpdate(video);
    fireEvent.ended(video);
    expect(finish).toHaveBeenCalledTimes(1);
    const detaches = attachAudio.mock.results
      .slice(-2)
      .flatMap((result) => (result.type === 'return' ? [result.value] : []));
    unmount();
    detaches.forEach((detach) => expect(detach).toHaveBeenCalledTimes(1));
  });
  it('does not mistake an application paused in the background for a failed asset', async () => {
    const finish = vi.fn();
    render(<OpeningCinematic onFinish={finish} sources={['/opening.mp4']} />);
    const hidden = vi.spyOn(document, 'hidden', 'get').mockReturnValue(true);
    fireEvent(document, new Event('visibilitychange'));
    await advance(30000);
    expect(finish).not.toHaveBeenCalled();
    hidden.mockReturnValue(false);
    fireEvent(document, new Event('visibilitychange'));
    await advance(12000);
    expect(finish).toHaveBeenCalledTimes(1);
  });
  it('advances on ended once', () => {
    const finish = vi.fn();
    const { container } = render(
      <OpeningCinematic onFinish={finish} sources={['/trailer.webm']} />,
    );
    const video = container.querySelector('video')!;
    fireEvent.ended(video);
    fireEvent.ended(video);
    expect(finish).toHaveBeenCalledTimes(1);
  });
  it('allows skips only after one second', async () => {
    const finish = vi.fn();
    render(<OpeningCinematic onFinish={finish} sources={['/trailer.webm']} />);
    fireEvent.keyDown(window, { key: ' ' });
    expect(finish).not.toHaveBeenCalled();
    await advance(1000);
    fireEvent.keyDown(window, { key: ' ' });
    fireEvent.click(screen.getByLabelText('Abertura CYBER WAR'));
    expect(finish).toHaveBeenCalledTimes(1);
  });
  it('falls back from WebM to MP4 and advances if both assets fail', () => {
    const finish = vi.fn();
    const { container } = render(
      <OpeningCinematic onFinish={finish} sources={['/trailer.webm', '/trailer.mp4']} />,
    );
    fireEvent.error(container.querySelector('video')!);
    expect(container.querySelector('video')).toHaveAttribute('src', '/trailer.mp4');
    fireEvent.error(container.querySelector('video')!);
    expect(finish).toHaveBeenCalledTimes(1);
  });
  it('does not stall when the asset is absent at build time', () => {
    const finish = vi.fn();
    render(
      <StrictMode>
        <OpeningCinematic onFinish={finish} sources={[]} />
      </StrictMode>,
    );
    expect(finish).toHaveBeenCalledTimes(1);
  });
});
describe('PressStartScreen', () => {
  it.each(['click', 'Enter', ' '])('accepts %s once', (input) => {
    const start = vi.fn();
    render(<PressStartScreen onStart={start} />);
    expect(screen.getByAltText('CYBER WAR_')).toBeInTheDocument();
    if (input === 'click') {
      fireEvent.click(screen.getByRole('button', { name: 'Clique para iniciar' }));
    } else {
      fireEvent.keyDown(window, { key: input });
    }
    fireEvent.keyDown(window, { key: 'Enter' });
    fireEvent.click(screen.getByRole('button'));
    expect(start).toHaveBeenCalledTimes(1);
  });
});
describe('BootFlow', () => {
  it('always waits for press start when both opening sequences are skipped', async () => {
    render(
      <BootFlow
        settings={{ ...defaultAppSettings, skipStudioIntro: true, skipTrailer: true }}
        onPlay={vi.fn()}
      />,
    );
    expect(screen.getByRole('button', { name: 'Clique para iniciar' })).toBeInTheDocument();
    expect(screen.queryByText('MENU PRINCIPAL')).not.toBeInTheDocument();
    fireEvent.keyDown(window, { key: 'Enter' });
    await advance(500);
    expect(screen.getByText('MENU PRINCIPAL')).toBeInTheDocument();
  });
  it('returns from gameplay directly to the menu', () => {
    render(<BootFlow settings={defaultAppSettings} returnToMenu onPlay={vi.fn()} />);
    expect(screen.getByText('MENU PRINCIPAL')).toBeInTheDocument();
    expect(screen.queryByLabelText('Abertura Studio Bechi Games')).not.toBeInTheDocument();
  });
});
