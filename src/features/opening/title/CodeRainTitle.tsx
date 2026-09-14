import { useEffect, useRef } from 'react';
import { TERMINAL_FONT } from '../../terminal/terminal-runtime';
import { CODE_ALPHABET, codeHash, codeTitleCells } from './code-title';
import cursorStyle from './title-cursor.json';

export function CodeRainTitle({ elapsed }: { elapsed: number }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const clock = useRef(elapsed);
  clock.current = elapsed;
  useEffect(() => {
    const element = canvas.current;
    const context = element?.getContext('2d');
    if (!element || !context) {
      return;
    }
    const width = window.innerWidth;
    const height = window.innerHeight;
    const dpr = window.devicePixelRatio;
    element.width = width * dpr;
    element.height = height * dpr;
    context.scale(dpr, dpr);
    const fontSize = Math.max(10, Math.round(width / 145));
    context.font = `600 ${fontSize}px ${TERMINAL_FONT}`;
    const cellWidth = context.measureText('0').width;
    const cellHeight = fontSize * 1.18;
    const columns = Math.ceil(width / cellWidth);
    const rows = Math.ceil(height / cellHeight);
    const targets = codeTitleCells(columns, rows, cellWidth / cellHeight);
    const mask = new Map(targets.map((cell) => [cell.column + cell.row * columns, cell]));
    let frame = 0;
    const draw = () => {
      const time = clock.current;
      const seconds = time / 1000;
      context.globalAlpha = 1;
      context.fillStyle = '#020502';
      context.fillRect(0, 0, width, height);
      const fade = Math.max(0, 1 - Math.max(0, time - 9100) / 3200);
      const enter = Math.min(1, time / 550);
      for (let column = 0; column < columns; column++) {
        const hash = codeHash(column, 3);
        const speed = 17 + (hash % 28);
        const length = rows * (0.65 + (hash % 28) / 100);
        const period = rows + 13;
        const head = seconds * speed + (hash % period);
        for (let row = 0; row < rows; row++) {
          const target = mask.get(column + row * columns);
          if (target && time >= target.freezeAt) {
            continue;
          }
          const distance = (((head - row) % period) + period) % period;
          const tail = Math.max(0, 1 - distance / length);
          context.globalAlpha = (0.055 + tail * tail * 0.73) * fade * enter;
          context.fillStyle = distance < 1.1 ? '#d6f9b1' : distance < 4 ? '#8ee33e' : '#469c2a';
          const char =
            CODE_ALPHABET[
              codeHash(column, row + Math.floor(seconds * (6 + (hash % 5)))) % CODE_ALPHABET.length
            ];
          context.fillText(char, column * cellWidth, (row + 1) * cellHeight - 2);
        }
      }
      for (const cell of targets) {
        if (time < cell.freezeAt) {
          continue;
        }
        const age = time - cell.freezeAt;
        context.globalAlpha =
          Math.min(1, age / 180) * (0.86 + (codeHash(cell.column, cell.row) % 14) / 100);
        context.fillStyle = age < 180 ? '#d4f9ad' : '#8bd837';
        context.fillText(cell.char, cell.column * cellWidth, (cell.row + 1) * cellHeight - 2);
      }
      context.globalAlpha = 1;
      if (
        time >= cursorStyle.revealAt &&
        Math.floor((time - cursorStyle.revealAt) / cursorStyle.blinkMs) % 2 === 0
      ) {
        context.fillStyle = cursorStyle.color;
        context.fillRect(
          width * cursorStyle.x,
          height * cursorStyle.y,
          width * cursorStyle.width,
          height * cursorStyle.height,
        );
      }
      element.dataset.titleCells = String(targets.length);
      element.dataset.frozenCells = String(targets.filter((cell) => time >= cell.freezeAt).length);
      if (time < 19500) {
        frame = requestAnimationFrame(draw);
      }
    };
    draw();
    return () => cancelAnimationFrame(frame);
  }, []);
  return (
    <canvas
      ref={canvas}
      className="cyber-war-reveal"
      aria-label="CYBER WAR formado por letras e números"
    />
  );
}
