import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { MediaPlayer } from './MediaPlayer';
import { ImageViewer } from './ImageViewer';
import { audioManager } from '../../lib/audio-manager';
import type * as ApiModule from '../../lib/api';

const api = vi.hoisted(() => ({ request: vi.fn() }));
vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof ApiModule>()),
  request: api.request,
}));
const createUrl = vi.fn(() => 'blob:virtual-media');
const revokeUrl = vi.fn();
const played = vi.fn();
const node = (path: string, mime = 'video/webm') => ({
  id: path,
  name: path.split('/').pop(),
  kind: 'file',
  parentId: '/home/kali',
  content: '',
  metadata: { mime },
  blob: { hash: 'a'.repeat(64), size: 3, mime },
  owner: 'kali',
  group: 'kali',
  mode: 420,
  modifiedAt: 1,
});

beforeEach(() => {
  vi.stubGlobal(
    'URL',
    Object.assign(URL, { createObjectURL: createUrl, revokeObjectURL: revokeUrl }),
  );
  vi.spyOn(HTMLMediaElement.prototype, 'play').mockImplementation(function (
    this: HTMLMediaElement,
  ) {
    fireEvent.play(this);
    played();
    return Promise.resolve();
  });
  vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(function (
    this: HTMLMediaElement,
  ) {
    fireEvent.pause(this);
  });
  api.request.mockImplementation((command: string, args: { path: string }) => {
    if (command === 'vfs_stat') {
      return Promise.resolve(
        node(args.path, args.path.endsWith('.png') ? 'image/png' : 'video/webm'),
      );
    }
    if (command === 'vfs_read_bytes') {
      return Promise.resolve({
        base64: 'AP+A',
        size: 3,
        mime: args.path.endsWith('.png') ? 'image/png' : 'video/webm',
      });
    }
    if (command === 'vfs_read' && args.path.endsWith('.srt')) {
      return Promise.resolve('1\n00:00:01,000 --> 00:00:02,000\nHello\n');
    }
    return Promise.reject(new Error('missing'));
  });
  audioManager.setVolumes({ master: 80, music: 50, voice: 100, sfx: 100 });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
});

it('loads actual binary bytes, seeks, mixes volume and controls playback without autoplay', async () => {
  const { container, unmount } = render(<MediaPlayer initialPath="/home/kali/movie.webm" />);
  await waitFor(() => expect(container.querySelector('video')).not.toBeNull());
  const video = container.querySelector('video')!;
  expect(played).not.toHaveBeenCalled();
  Object.defineProperty(video, 'duration', { configurable: true, value: 120 });
  fireEvent.loadedMetadata(video);
  fireEvent.click(screen.getByLabelText('Reproduzir'));
  expect(played).toHaveBeenCalledOnce();
  fireEvent.change(screen.getByLabelText('Barra de reprodução'), { target: { value: '45' } });
  fireEvent.click(screen.getByLabelText('Voltar 10 segundos'));
  expect(video.currentTime).toBe(35);
  fireEvent.click(screen.getByLabelText('Avançar 10 segundos'));
  expect(video.currentTime).toBe(45);
  fireEvent.change(screen.getByLabelText('Volume'), { target: { value: '0.5' } });
  expect(video.volume).toBeCloseTo(0.2);
  act(() => audioManager.setVolumes({ master: 0, music: 50, voice: 100, sfx: 100 }));
  expect(video.volume).toBe(0);
  fireEvent.click(screen.getByLabelText('Silenciar'));
  expect(video.muted).toBe(true);
  fireEvent.click(screen.getByLabelText('Repetir'));
  expect(video.loop).toBe(true);
  fireEvent.change(screen.getByLabelText('Velocidade de reprodução'), { target: { value: '1.5' } });
  expect(video.playbackRate).toBe(1.5);
  unmount();
  expect(revokeUrl).toHaveBeenCalledWith('blob:virtual-media');
});

it('converts SRT and applies caption mode after track load', async () => {
  const { container } = render(<MediaPlayer initialPath="/home/kali/movie.webm" />);
  await screen.findByLabelText('Alternar legendas');
  const video = container.querySelector('video')!;
  const track = { mode: 'disabled' };
  Object.defineProperty(video, 'textTracks', { value: [track] });
  const element = container.querySelector('track')!;
  expect(decodeURIComponent(element.src)).toContain('00:00:01.000 --> 00:00:02.000');
  fireEvent.load(element);
  expect(track.mode).toBe('showing');
  fireEvent.click(screen.getByLabelText('Alternar legendas'));
  expect(track.mode).toBe('hidden');
});

it('reports decoder errors and handles fullscreen rejection', async () => {
  const { container } = render(<MediaPlayer initialPath="/home/kali/movie.webm" />);
  await waitFor(() => expect(container.querySelector('video')).not.toBeNull());
  const player = container.querySelector('.media-player')!;
  Object.defineProperty(player, 'requestFullscreen', {
    value: vi.fn().mockRejectedValue(new Error('denied')),
  });
  fireEvent.click(screen.getByLabelText('Tela cheia'));
  await waitFor(() =>
    expect(screen.getByLabelText('Tela cheia')).toHaveAttribute('aria-pressed', 'false'),
  );
  fireEvent.error(container.querySelector('video')!);
  expect(screen.getByText(/codec não suportado/)).toBeInTheDocument();
});

it('fits images after zoom, rotates, handles keyboard and releases sources on replacement', async () => {
  const { container, rerender } = render(<ImageViewer initialPath="/home/kali/image.png" />);
  const img = await screen.findByRole('img');
  fireEvent.click(screen.getByLabelText('Aumentar zoom'));
  expect(img.style.transform).toContain('scale(1.25)');
  fireEvent.click(screen.getByText('Ajustar'));
  expect(img.style.transform).toContain('scale(1)');
  fireEvent.keyDown(container.querySelector('.image-viewer')!, { key: 'r' });
  expect(img.style.transform).toContain('rotate(90deg)');
  rerender(<ImageViewer initialPath="/home/kali/next.png" />);
  await waitFor(() => expect(revokeUrl).toHaveBeenCalled());
  expect(api.request).toHaveBeenCalledWith(
    'vfs_read_bytes',
    { path: '/home/kali/next.png' },
    expect.anything(),
  );
});

it('never creates a source when read permission fails even for a bundled asset', async () => {
  api.request.mockImplementation((command: string) => {
    if (command === 'vfs_stat') {
      return Promise.resolve({
        ...node('/home/kali/image.png', 'image/png'),
        blob: undefined,
        metadata: { mediaSource: '/assets/kali-waves.png' },
      });
    }
    return Promise.reject(new Error('permission denied'));
  });
  const { container } = render(<ImageViewer initialPath="/home/kali/image.png" />);
  await screen.findByText(/permission denied/);
  expect(container.querySelector('img')).toBeNull();
  expect(createUrl).not.toHaveBeenCalled();
});

it('does not publish URLs from a stale asynchronous read', async () => {
  let resolve!: (value: unknown) => void;
  api.request.mockImplementation((command: string) => {
    if (command === 'vfs_stat') {
      return Promise.resolve(node('/home/kali/image.png', 'image/png'));
    }
    return new Promise((done) => {
      resolve = done;
    });
  });
  const { unmount } = render(<ImageViewer initialPath="/home/kali/image.png" />);
  await waitFor(() => expect(resolve).toBeDefined());
  unmount();
  await act(() => {
    resolve({ base64: 'AP+A', size: 3, mime: 'image/png' });
    return Promise.resolve();
  });
  expect(createUrl).not.toHaveBeenCalled();
});
