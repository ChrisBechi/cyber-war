import { describe, expect, it } from 'vitest';
import { codeTitleCells } from './code-title';

describe('title composed of code', () => {
  it('keeps stable alphanumeric cells inside wide and tall viewports', () => {
    for (const [columns, rows] of [
      [242, 70],
      [290, 83],
      [172, 64],
    ]) {
      const cells = codeTitleCells(columns, rows);
      expect(cells).toEqual(codeTitleCells(columns, rows));
      expect(cells.length).toBeGreaterThan(350);
      expect(new Set(cells.map((cell) => `${cell.column}:${cell.row}`)).size).toBe(cells.length);
      expect(
        cells.every(
          (cell) =>
            /^[a-zA-Z0-9]$/.test(cell.char) &&
            cell.column >= 0 &&
            cell.column < columns &&
            cell.row >= 0 &&
            cell.row < rows,
        ),
      ).toBe(true);
    }
  });
  it('separates all eight letters and preserves three bars with open counters in E', () => {
    const cells = codeTitleCells(290, 83);
    const occupied = [...new Set(cells.map((cell) => cell.column))].sort((a, b) => a - b);
    const groups: number[][] = [];
    for (const column of occupied) {
      if (!groups.length || column - groups.at(-1)!.at(-1)! > 1) {
        groups.push([]);
      }
      groups.at(-1)!.push(column);
    }
    expect(groups).toHaveLength(8);
    const e = cells.filter((cell) => groups[3].includes(cell.column));
    const top = Math.min(...e.map((cell) => cell.row));
    const bottom = Math.max(...e.map((cell) => cell.row));
    const left = groups[3][0];
    const right = groups[3].at(-1)!;
    const widths = Array.from(
      { length: bottom - top + 1 },
      (_, i) => e.filter((cell) => cell.row === top + i).length,
    );
    expect(widths[0]).toBeGreaterThan((right - left) * 0.7);
    expect(widths[Math.floor(widths.length / 2)]).toBeGreaterThan((right - left) * 0.55);
    expect(widths.at(-1)).toBeGreaterThan((right - left) * 0.7);
    expect(widths[Math.floor(widths.length / 4)]).toBeLessThan((right - left) * 0.35);
    expect(widths[Math.floor((widths.length * 3) / 4)]).toBeLessThan((right - left) * 0.35);
  });
});
