export function ArchiveIcon({ format, size = 40 }: { format: string; size?: number }) {
  const type =
    format.includes('ZIP') && !format.includes('BZIP') && !format.includes('GZIP')
      ? 'ZIP'
      : format.includes('BZIP')
        ? 'BZ2'
        : format.includes('GZIP')
          ? 'GZ'
          : format.includes('XZ')
            ? 'XZ'
            : 'TAR';
  const colors: Record<string, string> = {
    ZIP: '#d3aa50',
    TAR: '#7cadd0',
    GZ: '#78bb98',
    BZ2: '#b090cc',
    XZ: '#d78c76',
  };
  return (
    <svg width={size} height={size} viewBox="0 0 40 40" role="img" aria-label={`Archive ${type}`}>
      <rect x="6" y="3" width="28" height="34" rx="4" fill={colors[type]} />
      <path d="M18 4h4v4h-4v4h4v4h-4v4h4v4h-4" fill="none" stroke="#263644" strokeWidth="2" />
      <rect x="8" y="25" width="24" height="10" rx="2" fill="#263644" />
      <text x="20" y="32.5" textAnchor="middle" fill="white" fontSize="8" fontFamily="sans-serif">
        {type}
      </text>
    </svg>
  );
}
