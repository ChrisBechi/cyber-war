export function GoggleLogo({ small = false }: { small?: boolean }) {
  return (
    <span
      role="img"
      aria-label="Goggle"
      className={`goggle-logo ${small ? 'goggle-logo-small' : ''}`}
    >
      {Array.from('Goggle').map((letter, i) => (
        <span aria-hidden="true" key={i}>
          {letter}
        </span>
      ))}
    </span>
  );
}
