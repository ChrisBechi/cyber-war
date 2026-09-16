// Crop each upstream SVG to its visible symbol, keeping the same padding around it.
const icons = {
  network: { name: 'network-wired-symbolic', size: 16, viewBox: '-1 -1 17 17' },
  volume: { name: 'audio-volume-high-symbolic', size: 24, viewBox: '2 2.5 19 19' },
  muted: { name: 'audio-volume-muted-symbolic', size: 24, viewBox: '2 2 20 20' },
  notifications: { name: 'notification-symbolic', size: 24, viewBox: '3.5 4 17 17' },
  power: { name: 'battery-full-charged-symbolic', size: 24, viewBox: '4 4 16 16' },
  lock: { name: 'system-lock-screen-symbolic', size: 16, viewBox: '0 0 16 16' },
  logout: { name: 'system-log-out-symbolic', size: 16, viewBox: '-1 -0.75 17.5 17.5' },
};

export function PanelStatusIcon({ name }: { name: keyof typeof icons }) {
  const icon = icons[name];
  return (
    <svg
      className="panel-status-icon"
      width="20"
      height="20"
      viewBox={icon.viewBox}
      aria-hidden="true"
      focusable="false"
    >
      <image href={`/assets/kali/${icon.name}.svg`} width={icon.size} height={icon.size} />
    </svg>
  );
}
