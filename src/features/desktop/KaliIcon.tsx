export function KaliIcon({
  name,
  size = 22,
  symbolic = false,
}: {
  name: string;
  size?: number;
  symbolic?: boolean;
}) {
  return (
    <img
      className={`app-icon kali-icon ${symbolic ? 'symbolic' : ''}`}
      src={`/assets/kali/${name}.svg`}
      width={size}
      height={size}
      alt=""
      draggable={false}
    />
  );
}
