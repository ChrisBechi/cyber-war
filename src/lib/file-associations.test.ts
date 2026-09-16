import { describe, expect, it } from 'vitest';
import { associationForPath, mediaSourceForPath } from './file-associations';

describe('virtual file associations', () => {
  it('opens DEB by extension or detected MIME even after renaming', () => {
    expect(associationForPath('/package.deb').application).toBe('package-installer');
    expect(
      associationForPath('/renamed.bin', {
        metadata: {},
        blob: { hash: 'a'.repeat(64), size: 30, mime: 'application/vnd.debian.binary-package' },
      }).application,
    ).toBe('package-installer');
  });
  it('opens shell scripts in the virtual terminal', () => {
    expect(associationForPath('/home/kali/tools/scan.sh').application).toBe('terminal');
    expect(associationForPath('/home/kali/tools/scan.sh').mime).toBe('application/x-shellscript');
  });

  it('selects media applications conditionally without claiming codec support', () => {
    expect(associationForPath('/home/kali/Music/track.flac').application).toBe('media-player');
    expect(associationForPath('/home/kali/Videos/clip.mkv').kind).toBe('video');
    expect(associationForPath('/home/kali/Pictures/evidence.webp').application).toBe(
      'image-viewer',
    );
    expect(
      associationForPath('/renamed.mp3', {
        metadata: {},
        blob: { hash: 'a'.repeat(64), size: 3, mime: 'image/png' },
      }).application,
    ).toBe('image-viewer');
    expect(
      associationForPath('/renamed.sh', {
        metadata: {},
        blob: { hash: 'a'.repeat(64), size: 3, mime: 'application/octet-stream' },
      }).support,
    ).toBe('unsupported');
  });
  it('distinguishes unsupported formats and does not execute Zsh as Bash', () => {
    expect(associationForPath('/document.pdf').support).toBe('unsupported');
    expect(associationForPath('/archive.zip').application).toBe('archive-viewer');
    expect(associationForPath('/archive.rar').support).toBe('unsupported');
    expect(associationForPath('/script.zsh').application).toBe('editor');
    expect(associationForPath('/clip.mkv').support).toBe('preview-conditional');
  });
  it('rejects external URLs hidden in virtual media metadata', () => {
    for (const mediaSource of [
      'https://example.com/track.mp3',
      '//example.com/clip.webm',
      'file:///etc/passwd',
      '/assets/../secret.png',
    ]) {
      expect(mediaSourceForPath('/x.mp3', { metadata: { mediaSource }, content: '' })).toBeNull();
    }
  });

  it('maps seeded virtual media to local project assets', () => {
    expect(mediaSourceForPath('/home/kali/Videos/cyber-war-opening.webm')).toBe(
      '/assets/video/cyber-war-opening.webm',
    );
  });
});
