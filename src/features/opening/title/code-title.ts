type Point = readonly [number, number];
// Uniform strokes and open counters, sampled as code cells rather than painted lettering.
const letters: Record<string, readonly (readonly Point[])[]> = {
  C: [
    [
      [7, 0],
      [1.4, 0],
      [0, 1.4],
      [0, 8.6],
      [1.4, 10],
      [7, 10],
    ],
  ],
  Y: [
    [
      [0, 0],
      [3.5, 5],
      [7, 0],
    ],
    [
      [3.5, 5],
      [3.5, 10],
    ],
  ],
  B: [
    [
      [0, 10],
      [0, 0],
      [5.5, 0],
      [7, 1.5],
      [7, 3.5],
      [5.5, 5],
      [0, 5],
    ],
    [
      [5.5, 5],
      [7, 6.5],
      [7, 8.5],
      [5.5, 10],
      [0, 10],
    ],
  ],
  E: [
    [
      [7, 0],
      [0, 0],
      [0, 10],
      [7, 10],
    ],
    [
      [0, 5],
      [5.7, 5],
    ],
  ],
  R: [
    [
      [0, 10],
      [0, 0],
      [5.5, 0],
      [7, 1.5],
      [7, 3.5],
      [5.5, 5],
      [0, 5],
    ],
    [
      [3.5, 5],
      [7, 10],
    ],
  ],
  W: [
    [
      [0, 0],
      [1.2, 10],
      [3.5, 5.5],
      [5.8, 10],
      [7, 0],
    ],
  ],
  A: [
    [
      [0, 10],
      [2.3, 0],
      [4.7, 0],
      [7, 10],
    ],
    [
      [1.1, 6.3],
      [5.9, 6.3],
    ],
  ],
};
export const CODE_ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyzABCDEF';
export const codeHash = (column: number, row: number) => {
  let value = Math.imul(column + 1, 374761393) ^ Math.imul(row + 1, 668265263);
  value = Math.imul(value ^ (value >>> 13), 1274126177);
  return (value ^ (value >>> 16)) >>> 0;
};
export type CodeCell = { column: number; row: number; char: string; freezeAt: number };
function distance(x: number, y: number, a: Point, b: Point) {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const along = Math.max(0, Math.min(1, ((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy)));
  return Math.hypot(x - a[0] - along * dx, y - a[1] - along * dy);
}
export function codeTitleCells(columns: number, rows: number, cellAspect = 0.53): CodeCell[] {
  const word = 'CYBER WAR';
  const advances = [0, 9.5, 19, 28.5, 38, 47.5, 53, 62.5, 72];
  const width = columns * cellAspect;
  const unit = Math.min((width * 0.87) / 79, (rows * 0.26) / 11.2);
  const left = (width - 79 * unit) / 2;
  const top = (rows - 10 * unit) / 2;
  const cells: CodeCell[] = [];
  for (let row = 0; row < rows; row++) {
    for (let column = 0; column < columns; column++) {
      const x = ((column + 0.5) * cellAspect - left) / unit;
      const y = (row + 0.5 - top) / unit;
      const lit = [...word].some((letter, i) =>
        letters[letter]?.some((path) =>
          path
            .slice(1)
            .some((end, segment) => distance(x - advances[i], y, path[segment], end) <= 0.62),
        ),
      );
      if (lit) {
        const hash = codeHash(column, row);
        cells.push({
          column,
          row,
          char: CODE_ALPHABET[hash % CODE_ALPHABET.length],
          freezeAt: 4900 + (hash % 3300) + Math.max(0, y) * 65,
        });
      }
    }
  }
  return cells;
}
