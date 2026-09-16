const videos = import.meta.glob<string>('/public/assets/video/cyber-war-opening.{webm,mp4}', {
  eager: true,
  query: '?url',
  import: 'default',
});
export const openingSources = Object.entries(videos)
  .sort(([a], [b]) => Number(b.endsWith('.webm')) - Number(a.endsWith('.webm')))
  .map(([, url]) => url);
export const studioLogo = '/assets/branding/studio-bechi-logo.svg';
export async function preloadBranding(): Promise<void> {
  await Promise.all(
    [studioLogo].map((url) => {
      const image = new Image();
      image.src = url;
      return image.decode().catch(() => undefined);
    }),
  );
}
