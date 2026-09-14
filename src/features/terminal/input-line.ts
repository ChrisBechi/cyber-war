const segmenter = new Intl.Segmenter('pt-BR', { granularity: 'grapheme' });
const graphemes = (text: string): string[] => [...segmenter.segment(text)].map((s) => s.segment);

export class InputLine {
  private cells: string[] = [];
  private position = 0;
  private history: string[] = [];
  private historyIndex = 0;
  private draft = '';
  revision = 0;

  get text(): string {
    return this.cells.join('');
  }
  get cursor(): number {
    return [...this.cells.slice(0, this.position).join('')].length;
  }

  set(text: string, cursor = [...text].length): void {
    this.cells = graphemes(text);
    this.position = graphemes([...text].slice(0, cursor).join('')).length;
    this.revision += 1;
  }

  insert(text: string): void {
    const left = this.cells.slice(0, this.position).join('') + text;
    const next = left + this.cells.slice(this.position).join('');
    if (new TextEncoder().encode(next).length <= 8192) {
      this.set(next, [...left].length);
    }
  }

  edit(key: string): boolean {
    switch (key) {
      case '\x1b[D':
      case '\x1bOD':
        this.position = Math.max(0, this.position - 1);
        break;
      case '\x1b[C':
      case '\x1bOC':
        this.position = Math.min(this.cells.length, this.position + 1);
        break;
      case '\x01':
      case '\x1b[H':
      case '\x1bOH':
      case '\x1b[1~':
        this.position = 0;
        break;
      case '\x05':
      case '\x1b[F':
      case '\x1bOF':
      case '\x1b[4~':
        this.position = this.cells.length;
        break;
      case '\x7f':
      case '\b':
        if (this.position > 0) {
          this.cells.splice(--this.position, 1);
        }
        break;
      case '\x1b[3~':
        this.cells.splice(this.position, 1);
        break;
      case '\x15':
        this.cells.splice(0, this.position);
        this.position = 0;
        break;
      case '\x0b':
        this.cells.splice(this.position);
        break;
      case '\x1b[A':
      case '\x1bOA':
        if (this.historyIndex === this.history.length) {
          this.draft = this.text;
        }
        this.historyIndex = Math.max(0, this.historyIndex - 1);
        this.set(this.history[this.historyIndex] ?? this.draft);
        break;
      case '\x1b[B':
      case '\x1bOB':
        this.historyIndex = Math.min(this.history.length, this.historyIndex + 1);
        this.set(this.history[this.historyIndex] ?? this.draft);
        break;
      default:
        return false;
    }
    this.revision += 1;
    return true;
  }

  submit(): string {
    const text = this.text;
    if (text.trim()) {
      this.history.push(text);
    }
    this.historyIndex = this.history.length;
    this.draft = '';
    this.set('');
    return text;
  }

  /** Horizontal viewport keeps the cursor editable even in a narrow terminal. */
  viewport(width: number): { text: string; cursorColumn: number } {
    const capacity = Math.max(1, width);
    let start = this.position;
    let before = 0;
    while (start > 0 && before + cellWidth(this.cells[start - 1]) <= capacity) {
      before += cellWidth(this.cells[--start]);
    }
    let end = start;
    let used = 0;
    while (end < this.cells.length && used + cellWidth(this.cells[end]) <= capacity) {
      used += cellWidth(this.cells[end++]);
    }
    return { text: this.cells.slice(start, end).join(''), cursorColumn: before };
  }
}

function cellWidth(grapheme: string): number {
  if (/^\p{Mark}+$/u.test(grapheme) || grapheme === '\u200d') {
    return 0;
  }
  const code = grapheme.codePointAt(0) ?? 0;
  if (
    /\p{Emoji_Presentation}|\uFE0F/u.test(grapheme) ||
    (code >= 0x1100 &&
      (code <= 0x115f ||
        code === 0x2329 ||
        code === 0x232a ||
        (code >= 0x2e80 && code <= 0xa4cf) ||
        (code >= 0xac00 && code <= 0xd7a3) ||
        (code >= 0xf900 && code <= 0xfaff) ||
        (code >= 0xfe10 && code <= 0xfe6f) ||
        (code >= 0xff01 && code <= 0xff60) ||
        (code >= 0xffe0 && code <= 0xffe6) ||
        code >= 0x20000))
  ) {
    return 2;
  }
  return 1;
}
