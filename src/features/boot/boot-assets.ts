const videos = import.meta.glob<string>('/public/assets/video/cyber-war-opening.{webm,mp4}', {
  eager: true,
  query: '?url',
  import: 'default',
});
export const openingSources = Object.entries(videos)
  .sort(([a], [b]) => Number(b.endsWith('.webm')) - Number(a.endsWith('.webm')))
  .map(([, url]) => url);
export const studioLogo = '/assets/branding/studio-bechi-logo.svg';
export const gameLogo = '/assets/branding/cyber-war-logo.png';
export const pressStartLogo = '/assets/branding/cyber-war-logo.png';
export async function preloadBranding(): Promise<void> {
  await Promise.all(
    [studioLogo, gameLogo, pressStartLogo].map((url) => {
      const image = new Image();
      image.src = url;
      return image.decode().catch(() => undefined);
    }),
  );
}
