import type { Terminal } from '@xterm/xterm';
import type { NanoLaunch } from '../../lib/api';

type PromptMode =
  'normal' | 'confirm' | 'write' | 'search' | 'replace' | 'replace-confirm' | 'read' | 'help';
type Position = { line: number; column: number };
type HistoryState = { lines: string[][]; finalNewline: boolean; cursor: Position };

export type NanoActions = {
  write: (path: string, content: string, expectedContent: string | null) => Promise<void>;
  read: (path: string) => Promise<string>;
  close: () => Promise<void>;
  exited: () => void;
};

const KEY = {
  backspace: '\x7f',
  cancel: '\x03',
  enter: '\r',
  escape: '\x1b',
  exit: '\x18',
  writeOut: '\x0f',
  help: '\x07',
  cut: '\x0b',
  paste: '\x15',
  mark: '\x1e',
  search: '\x17',
  replace: '\x1c',
  read: '\x12',
  cursorPos: '\x03',
  goToLine: '\x1f',
  undo: '\x1a',
  redo: '\x19',
  spell: '\x14',
};

function splitContent(content: string): { lines: string[][]; finalNewline: boolean } {
  const normalized = content.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  const finalNewline = normalized.endsWith('\n');
  const values = normalized.split('\n');
  if (finalNewline) {
    values.pop();
  }
  if (values.length === 0) {
    values.push('');
  }
  return { lines: values.map((line) => Array.from(line)), finalNewline };
}

function safeText(data: string): string {
  return Array.from(data)
    .filter((char) => char >= ' ' && char !== '\x7f' && char !== '\x1b')
    .join('');
}

export class NanoEditor {
  private readonly terminal: Terminal;
  private readonly actions: NanoActions;
  private path: string;
  private readonly options: NanoLaunch['options'];
  private lines: string[][];
  private displayName: string;
  private finalNewline: boolean;
  private cursor: Position;
  private scrollLine = 0;
  private scrollColumn = 0;
  private mark: Position | null = null;
  private cutBuffer = '';
  private consecutiveCut = false;
  private exitAfterWrite = false;
  private caseSensitive = false;
  private backwards = false;
  private replacement = '';
  private replaceQueue: Position[] = [];
  private replaced = 0;
  private history: HistoryState[] = [];
  private redoHistory: HistoryState[] = [];
  private expectedContent: string | null;
  private promptMode: PromptMode = 'normal';
  private promptText = '';
  private promptCursor = 0;
  private statusMessage = '';
  private pending = false;
  private searchTerm = '';
  private dirty = false;

  constructor(terminal: Terminal, launch: NanoLaunch, actions: NanoActions) {
    this.terminal = terminal;
    this.actions = actions;
    this.path = launch.path;
    this.displayName = launch.displayName;
    this.options = launch.options;
    const content = splitContent(launch.content);
    this.lines = content.lines;
    this.finalNewline = content.finalNewline;
    this.expectedContent = launch.expectedContent;
    this.cursor = {
      line: Math.min(Math.max(launch.startingLine - 1, 0), this.lines.length - 1),
      column: Math.max(launch.startingColumn - 1, 0),
    };
    this.clampCursor();
    this.render();
  }

  handle(data: string): void {
    if (this.pending) {
      return;
    }
    if (data !== KEY.cut) {
      this.consecutiveCut = false;
    }
    if (this.promptMode !== 'normal') {
      this.handlePrompt(data);
      return;
    }
    if (!this.handleSpecial(data)) {
      if (data.startsWith(KEY.escape)) {
        return;
      }
      const text = Array.from(data)
        .filter(
          (char) =>
            char === '\r' ||
            char === '\n' ||
            char === '\t' ||
            (char >= ' ' && char !== '\x7f' && char !== '\x1b'),
        )
        .join('');
      if (text) {
        if (this.options.view) {
          this.statusMessage = 'File is read-only';
          this.render();
          return;
        }
        this.record();
        this.insertTextAtCursor(text);
        this.dirty = true;
        this.statusMessage = '';
      }
      this.render();
    }
  }

  dispose(): void {
    if (!this.pending) {
      void this.actions.close();
    }
  }

  resize(): void {
    this.render();
  }

  private handleSpecial(data: string): boolean {
    const movement: Record<string, () => void> = {
      '\x1b[A': () => this.moveVertical(-1),
      '\x1b[B': () => this.moveVertical(1),
      '\x1b[C': () => this.moveHorizontal(1),
      '\x1b[D': () => this.moveHorizontal(-1),
      '\x1b[H': () => this.moveHome(),
      '\x1b[F': () => this.moveEnd(),
      '\x1b[1~': () => this.moveHome(),
      '\x1b[4~': () => this.moveEnd(),
      '\x1b[5~': () => this.moveVertical(-this.bodyRows()),
      '\x1b[6~': () => this.moveVertical(this.bodyRows()),
      '\x1b[3~': () => this.deleteForward(),
    };
    if (movement[data]) {
      movement[data]();
      this.statusMessage = '';
      this.render();
      return true;
    }
    if (data === KEY.backspace || data === '\x08') {
      this.backspace();
      this.render();
      return true;
    }
    if (data === KEY.enter) {
      this.newLine();
      this.render();
      return true;
    }
    switch (data) {
      case '\x1bu':
        this.undo();
        this.render();
        return true;
      case '\x1be':
        this.redo();
        this.render();
        return true;
      case '\x1b6':
        this.copyText();
        this.render();
        return true;
      case '\x1ba':
        this.mark = this.mark ? null : { ...this.cursor };
        this.statusMessage = this.mark ? 'Mark Set' : 'Mark Unset';
        this.render();
        return true;
      case '\x1bw':
        if (this.searchTerm) {
          this.findNext(this.searchTerm);
        }
        this.render();
        return true;
      case KEY.exit:
        void this.requestExit();
        return true;
      case KEY.writeOut:
        this.beginWrite();
        return true;
      case KEY.help:
        this.promptMode = 'help';
        this.render();
        return true;
      case KEY.cut:
        this.cutLine();
        this.render();
        return true;
      case KEY.paste:
        this.paste();
        this.render();
        return true;
      case KEY.mark:
        this.mark = this.mark ? null : { ...this.cursor };
        this.statusMessage = this.mark ? 'Mark Set' : 'Mark Unset';
        this.render();
        return true;
      case KEY.search:
        this.promptMode = 'search';
        this.promptText = this.searchTerm;
        this.promptCursor = this.promptText.length;
        this.statusMessage = '';
        this.render();
        return true;
      case KEY.replace:
        if (this.options.view) {
          this.statusMessage = 'File is read-only';
          this.render();
          return true;
        }
        this.promptMode = 'replace';
        this.promptText = '';
        this.promptCursor = 0;
        this.render();
        return true;
      case KEY.read:
        this.promptMode = 'read';
        this.promptText = '';
        this.promptCursor = 0;
        this.render();
        return true;
      case KEY.goToLine:
        this.promptMode = 'search';
        this.statusMessage = 'Go To Line, Column';
        this.promptText = '';
        this.promptCursor = 0;
        this.render();
        return true;
      case KEY.undo:
        this.undo();
        this.render();
        return true;
      case KEY.redo:
        this.redo();
        this.render();
        return true;
      case KEY.spell:
        this.statusMessage = this.options.speller
          ? 'Spell check is unavailable in the virtual terminal'
          : 'Spell checking is not configured';
        this.render();
        return true;
      case KEY.cursorPos:
        this.statusMessage =
          'line ' + (this.cursor.line + 1) + ', column ' + (this.cursor.column + 1);
        this.render();
        return true;
      case '\x01':
        this.moveHome();
        this.render();
        return true;
      case '\x05':
        this.moveEnd();
        this.render();
        return true;
      case '\x0c':
        this.render();
        return true;
      default:
        return false;
    }
  }

  private handlePrompt(data: string): void {
    if ((this.promptMode === 'search' || this.promptMode === 'replace') && data === '\x1bc') {
      this.caseSensitive = !this.caseSensitive;
      this.statusMessage = this.caseSensitive ? 'Case sensitive' : 'Ignore case';
      this.render();
      return;
    }
    if (this.promptMode === 'search' && data === '\x1bb') {
      this.backwards = !this.backwards;
      this.statusMessage = this.backwards ? 'Search backwards' : 'Search forwards';
      this.render();
      return;
    }
    if (this.promptMode === 'replace-confirm') {
      const answer = data.toLowerCase();
      if (answer === 'y' || answer === 'a') {
        this.record();
        do {
          this.replaceCurrent();
        } while (answer === 'a' && this.replaceQueue.length > 0);
        this.nextReplacement();
      } else if (answer === 'n') {
        this.replaceQueue.shift();
        this.nextReplacement();
      } else if (data === KEY.cancel || data === KEY.escape) {
        this.replaceQueue = [];
        this.promptMode = 'normal';
        this.statusMessage = 'Cancelled';
      }
      this.render();
      return;
    }
    if (this.promptMode === 'help') {
      if (data === KEY.escape || data === KEY.cancel || data === KEY.exit) {
        this.promptMode = 'normal';
        this.statusMessage = '';
        this.render();
      }
      return;
    }
    if (this.promptMode === 'confirm') {
      const answer = data.toLowerCase();
      if (answer === 'y') {
        this.exitAfterWrite = true;
        this.beginWrite();
      } else if (answer === 'n') {
        void this.closeWithoutSaving();
      } else if (answer === 'c' || data === KEY.cancel || data === KEY.escape) {
        this.exitAfterWrite = false;
        this.promptMode = 'normal';
        this.statusMessage = '';
        this.render();
      }
      return;
    }
    if (data === KEY.cancel || data === KEY.escape) {
      this.exitAfterWrite = false;
      this.promptMode = 'normal';
      this.statusMessage = '';
      this.render();
      return;
    }
    if (data === KEY.backspace || data === '\x08') {
      if (this.promptCursor > 0) {
        this.promptText =
          this.promptText.slice(0, this.promptCursor - 1) +
          this.promptText.slice(this.promptCursor);
        this.promptCursor -= 1;
      }
      this.render();
      return;
    }
    if (data === '\x1b[D' && this.promptCursor > 0) {
      this.promptCursor -= 1;
      this.render();
      return;
    }
    if (data === '\x1b[C' && this.promptCursor < this.promptText.length) {
      this.promptCursor += 1;
      this.render();
      return;
    }
    if (data === KEY.enter) {
      void this.submitPrompt();
      return;
    }
    if (data.startsWith(KEY.escape)) {
      return;
    }
    const text = safeText(data);
    if (text) {
      this.promptText =
        this.promptText.slice(0, this.promptCursor) +
        text +
        this.promptText.slice(this.promptCursor);
      this.promptCursor += text.length;
      this.render();
    }
  }

  private async submitPrompt(): Promise<void> {
    const value = this.promptText;
    if (this.promptMode === 'write') {
      this.pending = true;
      this.statusMessage = 'Writing...';
      this.render();
      try {
        const content = this.serialize();
        await this.actions.write(value || this.path, content, this.expectedContent);
        this.path = value || this.path;
        this.displayName = value || this.displayName;
        this.expectedContent = content;
        this.dirty = false;
        this.statusMessage = 'Wrote to disk';
        this.promptMode = 'normal';
        if (this.exitAfterWrite) {
          this.exitAfterWrite = false;
          await this.actions.close();
          this.actions.exited();
        }
      } catch (error) {
        this.exitAfterWrite = false;
        this.statusMessage = String(error);
        this.promptMode = 'normal';
      } finally {
        this.pending = false;
        this.render();
      }
      return;
    }
    if (this.promptMode === 'search') {
      this.promptMode = 'normal';
      if (this.statusMessage === 'Go To Line, Column') {
        this.goTo(value);
      } else if (value) {
        this.searchTerm = value;
        this.findNext(value);
      }
      this.render();
      return;
    }
    if (this.promptMode === 'replace') {
      if (value) {
        this.searchTerm = value;
        this.promptMode = 'read';
        this.statusMessage = 'Replace with:';
        this.promptText = '';
        this.promptCursor = 0;
        this.render();
      }
      return;
    }
    if (this.promptMode === 'read') {
      if (this.statusMessage === 'Replace with:') {
        this.replacement = value;
        this.replaced = 0;
        const range = this.selection();
        this.replaceQueue = this.matches(this.searchTerm).filter(
          (position) =>
            !range ||
            (this.compare(position, range[0]) >= 0 &&
              this.compare(
                {
                  line: position.line,
                  column: position.column + Array.from(this.searchTerm).length,
                },
                range[1],
              ) <= 0),
        );
        const start = this.replaceQueue.findIndex((p) => this.compare(p, this.cursor) >= 0);
        if (start > 0 && !range) {
          this.replaceQueue = [
            ...this.replaceQueue.slice(start),
            ...this.replaceQueue.slice(0, start),
          ];
        }
        this.mark = null;
        this.nextReplacement();
        this.render();
        return;
      }
      if (this.options.view) {
        this.statusMessage = 'File is read-only';
        this.promptMode = 'normal';
        this.render();
        return;
      }
      if (!value) {
        this.promptMode = 'normal';
        this.render();
        return;
      }
      this.pending = true;
      this.statusMessage = 'Reading...';
      this.render();
      try {
        const content = await this.actions.read(value);
        this.record();
        this.insertTextAtCursor(content);
        this.dirty = true;
        this.statusMessage = 'Read ' + content.split(/\r?\n/).length + ' lines';
      } catch (error) {
        this.statusMessage = String(error);
      } finally {
        this.pending = false;
        this.promptMode = 'normal';
        this.render();
      }
    }
  }

  private async requestExit(): Promise<void> {
    if (this.dirty && !this.options.view) {
      this.promptMode = 'confirm';
      this.statusMessage = 'Save modified buffer (ANSWERING "No" WILL DESTROY CHANGES)?';
      this.render();
      return;
    }
    await this.closeWithoutSaving();
  }

  private async closeWithoutSaving(): Promise<void> {
    this.pending = true;
    try {
      await this.actions.close();
      this.actions.exited();
    } catch (error) {
      this.statusMessage = String(error);
      this.pending = false;
      this.promptMode = 'normal';
      this.render();
    }
  }

  private beginWrite(): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
    } else {
      this.promptMode = 'write';
      this.promptText = this.path;
      this.promptCursor = this.promptText.length;
      this.statusMessage = '';
    }
    this.render();
  }

  private record(): void {
    this.history.push({
      lines: this.lines.map((line) => [...line]),
      finalNewline: this.finalNewline,
      cursor: { ...this.cursor },
    });
    if (this.history.length > 100) {
      this.history.shift();
    }
    this.redoHistory = [];
  }

  private undo(): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
      return;
    }
    const state = this.history.pop();
    if (!state) {
      this.statusMessage = 'Nothing to undo';
      return;
    }
    this.redoHistory.push({
      lines: this.lines.map((line) => [...line]),
      finalNewline: this.finalNewline,
      cursor: { ...this.cursor },
    });
    this.restore(state);
  }

  private redo(): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
      return;
    }
    const state = this.redoHistory.pop();
    if (!state) {
      this.statusMessage = 'Nothing to redo';
      return;
    }
    this.history.push({
      lines: this.lines.map((line) => [...line]),
      finalNewline: this.finalNewline,
      cursor: { ...this.cursor },
    });
    this.restore(state);
  }

  private restore(state: HistoryState): void {
    this.lines = state.lines.map((line) => [...line]);
    this.finalNewline = state.finalNewline;
    this.cursor = { ...state.cursor };
    this.clampCursor();
    this.mark = null;
    this.dirty = this.serialize() !== this.expectedContent;
  }

  private insertCharacter(char: string): void {
    if (char === '\t' && this.options.tabsToSpaces) {
      const spaces = this.options.tabSize - (this.cursor.column % this.options.tabSize);
      this.lines[this.cursor.line].splice(this.cursor.column, 0, ...' '.repeat(spaces));
      this.cursor.column += spaces;
    } else {
      this.lines[this.cursor.line].splice(this.cursor.column, 0, char);
      this.cursor.column += 1;
    }
  }

  private insertTextAtCursor(text: string): void {
    for (const char of text.replace(/\r\n/g, '\n').replace(/\r/g, '\n')) {
      if (char === '\n') {
        this.newLine(false, false);
      } else {
        this.insertCharacter(char);
      }
    }
  }

  private newLine(record = true, autoIndent = true): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
      return;
    }
    if (record) {
      this.record();
    }
    const current = this.lines[this.cursor.line];
    const before = current.slice(0, this.cursor.column);
    let after = current.slice(this.cursor.column);
    let indent = '';
    if (this.options.autoIndent && autoIndent) {
      indent = before.join('').match(/^\s*/)?.[0] ?? '';
      after = Array.from(indent).concat(after);
    }
    this.lines[this.cursor.line] = before;
    this.lines.splice(this.cursor.line + 1, 0, after);
    this.cursor.line += 1;
    this.cursor.column = indent.length;
    this.dirty = true;
  }

  private backspace(): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
      return;
    }
    if (this.cursor.column > 0) {
      this.record();
      this.lines[this.cursor.line].splice(this.cursor.column - 1, 1);
      this.cursor.column -= 1;
      this.dirty = true;
    } else if (this.cursor.line > 0) {
      this.record();
      const length = this.lines[this.cursor.line - 1].length;
      this.lines[this.cursor.line - 1].push(...this.lines[this.cursor.line]);
      this.lines.splice(this.cursor.line, 1);
      this.cursor.line -= 1;
      this.cursor.column = length;
      this.dirty = true;
    }
  }

  private deleteForward(): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
      return;
    }
    const line = this.lines[this.cursor.line];
    if (this.cursor.column < line.length) {
      this.record();
      line.splice(this.cursor.column, 1);
      this.dirty = true;
    } else if (this.cursor.line < this.lines.length - 1) {
      this.record();
      line.push(...this.lines[this.cursor.line + 1]);
      this.lines.splice(this.cursor.line + 1, 1);
      this.dirty = true;
    }
  }

  private compare(a: Position, b: Position): number {
    return a.line - b.line || a.column - b.column;
  }

  private selection(): [Position, Position] | null {
    if (!this.mark || this.compare(this.mark, this.cursor) === 0) {
      return null;
    }
    return this.compare(this.mark, this.cursor) < 0
      ? [{ ...this.mark }, { ...this.cursor }]
      : [{ ...this.cursor }, { ...this.mark }];
  }

  private selectedText(range: [Position, Position]): string {
    const [first, last] = range;
    if (first.line === last.line) {
      return this.lines[first.line].slice(first.column, last.column).join('');
    }
    return [
      this.lines[first.line].slice(first.column).join(''),
      ...this.lines.slice(first.line + 1, last.line).map((line) => line.join('')),
      this.lines[last.line].slice(0, last.column).join(''),
    ].join('\n');
  }

  private copyText(): void {
    const range = this.selection();
    this.cutBuffer = range
      ? this.selectedText(range)
      : this.lines[this.cursor.line].join('') +
        (this.cursor.line < this.lines.length - 1 || this.finalNewline ? '\n' : '');
    this.mark = null;
    this.statusMessage = 'Copied text';
  }

  private cutLine(): void {
    if (this.options.view) {
      this.statusMessage = 'File is read-only';
      return;
    }
    const range = this.selection();
    this.record();
    let cut: string;
    if (range) {
      cut = this.selectedText(range);
      const [first, last] = range;
      const prefix = this.lines[first.line].slice(0, first.column);
      const suffix = this.lines[last.line].slice(last.column);
      this.lines.splice(first.line, last.line - first.line + 1, prefix.concat(suffix));
      this.cursor = { ...first };
      this.mark = null;
    } else if (
      this.options.cutFromCursor &&
      this.cursor.column < this.lines[this.cursor.line].length
    ) {
      cut = this.lines[this.cursor.line].splice(this.cursor.column).join('');
    } else if (this.options.cutFromCursor && this.cursor.line < this.lines.length - 1) {
      cut = '\n';
      this.lines[this.cursor.line].push(...this.lines.splice(this.cursor.line + 1, 1)[0]);
    } else {
      cut =
        this.lines[this.cursor.line].join('') +
        (this.cursor.line < this.lines.length - 1 || this.finalNewline ? '\n' : '');
      this.lines.splice(this.cursor.line, 1);
      if (this.lines.length === 0) {
        this.lines.push([]);
        this.finalNewline = false;
      }
      this.cursor.line = Math.min(this.cursor.line, this.lines.length - 1);
      this.cursor.column = 0;
    }
    this.cutBuffer = this.consecutiveCut && !range ? this.cutBuffer + cut : cut;
    this.consecutiveCut = true;
    this.clampCursor();
    this.dirty = true;
  }

  private paste(): void {
    if (this.options.view || !this.cutBuffer) {
      return;
    }
    this.record();
    this.insertTextAtCursor(this.cutBuffer);
    this.mark = null;
    this.dirty = true;
  }

  private moveVertical(delta: number): void {
    this.cursor.line = Math.max(0, Math.min(this.lines.length - 1, this.cursor.line + delta));
    this.clampCursor();
  }

  private moveHorizontal(delta: number): void {
    if (delta < 0 && this.cursor.column === 0 && this.cursor.line > 0) {
      this.cursor.line -= 1;
      this.cursor.column = this.lines[this.cursor.line].length;
    } else if (
      delta > 0 &&
      this.cursor.column >= this.lines[this.cursor.line].length &&
      this.cursor.line < this.lines.length - 1
    ) {
      this.cursor.line += 1;
      this.cursor.column = 0;
    } else {
      this.cursor.column = Math.max(
        0,
        Math.min(this.lines[this.cursor.line].length, this.cursor.column + delta),
      );
    }
  }

  private moveHome(): void {
    if (this.options.smartHome) {
      const first = this.lines[this.cursor.line].findIndex((char) => !/\s/.test(char));
      this.cursor.column = first >= 0 && this.cursor.column !== first ? first : 0;
    } else {
      this.cursor.column = 0;
    }
  }

  private moveEnd(): void {
    this.cursor.column = this.lines[this.cursor.line].length;
  }

  private clampCursor(): void {
    this.cursor.line = Math.max(0, Math.min(this.lines.length - 1, this.cursor.line));
    this.cursor.column = Math.max(
      0,
      Math.min(this.lines[this.cursor.line].length, this.cursor.column),
    );
  }

  private matches(term: string): Position[] {
    if (!term) {
      return [];
    }
    const positions: Position[] = [];
    const length = Array.from(term).length;
    const target = this.caseSensitive ? term : term.toLowerCase();
    for (let line = 0; line < this.lines.length; line += 1) {
      for (let column = 0; column <= this.lines[line].length - length; column += 1) {
        const text = this.lines[line].slice(column, column + length).join('');
        if ((this.caseSensitive ? text : text.toLowerCase()) === target) {
          positions.push({ line, column });
        }
      }
    }
    return positions;
  }

  private findNext(term: string): void {
    const positions = this.matches(term);
    if (this.backwards) {
      positions.reverse();
    }
    const next = positions.find((p) =>
      this.backwards ? this.compare(p, this.cursor) < 0 : this.compare(p, this.cursor) > 0,
    );
    const position = next ?? positions[0];
    if (position) {
      this.cursor = { ...position };
      this.statusMessage = next ? '' : 'Search Wrapped';
    } else {
      this.statusMessage = '"' + term + '" not found';
    }
  }

  private replaceCurrent(): void {
    const position = this.replaceQueue.shift();
    if (!position) {
      return;
    }
    const length = Array.from(this.searchTerm).length;
    const replacement = Array.from(this.replacement);
    this.lines[position.line].splice(position.column, length, ...replacement);
    // Drop overlapping original matches. Never search the replacement itself.
    this.replaceQueue = this.replaceQueue.filter(
      (p) =>
        p.line !== position.line ||
        p.column < position.column ||
        p.column >= position.column + length,
    );
    for (const next of this.replaceQueue) {
      if (next.line === position.line && next.column > position.column) {
        next.column += replacement.length - length;
      }
    }
    this.cursor = { line: position.line, column: position.column + replacement.length };
    this.replaced += 1;
    this.dirty = true;
  }

  private nextReplacement(): void {
    const next = this.replaceQueue[0];
    if (next) {
      this.cursor = { ...next };
      this.promptMode = 'replace-confirm';
      this.statusMessage = 'Replace this instance? Y Yes / N No / A All / ^C Cancel';
    } else {
      this.promptMode = 'normal';
      this.statusMessage = this.replaced
        ? 'Replaced ' + this.replaced + ' occurrence(s)'
        : '"' + this.searchTerm + '" not found or no replacements';
    }
  }

  private goTo(value: string): void {
    const parts = value.split(',');
    const line = Number.parseInt(parts[0], 10);
    const column = Number.parseInt(parts[1] ?? '1', 10);
    if (!Number.isInteger(line) || line < 1 || !Number.isInteger(column) || column < 1) {
      this.statusMessage = 'Invalid line or column';
      return;
    }
    this.cursor.line = Math.min(line - 1, this.lines.length - 1);
    this.cursor.column = Math.min(column - 1, this.lines[this.cursor.line].length);
    this.statusMessage = '';
  }

  private serialize(): string {
    let result = this.lines.map((line) => line.join('')).join('\n');
    if (
      this.finalNewline ||
      (result.length > 0 && !this.options.noNewlines && !result.endsWith('\n'))
    ) {
      result += '\n';
    }
    return result;
  }

  private bodyRows(): number {
    return Math.max(1, this.terminal.rows - (this.showHelp() ? 4 : 2));
  }

  private showHelp(): boolean {
    return !this.options.noHelp && !this.options.minibar;
  }

  private visibleWidth(): number {
    return Math.max(1, this.terminal.cols - (this.options.lineNumbers ? 6 : 0));
  }

  private render(): void {
    const rows = Math.max(6, this.terminal.rows);
    const bodyRows = this.bodyRows();
    const numberWidth = this.options.lineNumbers ? 6 : 0;
    const width = this.visibleWidth();
    this.ensureViewport(bodyRows, width);
    const title = 'GNU nano 8.7' + (this.dirty ? '  Modified' : '');
    let output = '';
    for (let row = 1; row <= rows; row += 1) {
      output += '\x1b[' + row + ';1H\x1b[2K';
      if (row === 1) {
        output += this.centerTitle(title, this.displayName);
      } else if (row >= 2 && row < 2 + bodyRows) {
        output += this.renderLine(row - 2, width);
      } else if (row === 2 + bodyRows) {
        output += '\x1b[7m' + this.statusLine(width) + '\x1b[0m';
      } else if (this.showHelp() && row === rows - 1) {
        output += '^G Get Help  ^O Write Out  ^W Where Is  ^K Cut Text  ^J Justify  ^C Cur Pos';
      } else if (this.showHelp() && row === rows) {
        output +=
          '^X Exit      ^R Read File  ^\\\\ Replace  ^U Paste Text ^T To Spell  ^_ Go To Line';
      }
    }
    const visualCursorRow = this.options.softwrap
      ? this.visualCursorRow(width)
      : this.cursor.line - this.scrollLine;
    const cursorRow = Math.min(2 + Math.max(0, visualCursorRow), 1 + bodyRows);
    const cursorColumn = Math.max(
      1,
      numberWidth +
        (this.options.softwrap
          ? this.cursor.column % width
          : this.cursor.column - this.scrollColumn) +
        1,
    );
    output += '\x1b[' + cursorRow + ';' + Math.min(cursorColumn, this.terminal.cols) + 'H\x1b[?25h';
    this.terminal.write(output);
  }

  private ensureViewport(bodyRows: number, width: number): void {
    if (this.cursor.line < this.scrollLine) {
      this.scrollLine = this.cursor.line;
    } else if (this.cursor.line >= this.scrollLine + bodyRows) {
      this.scrollLine = this.cursor.line - bodyRows + 1;
    }
    if (this.options.softwrap) {
      this.scrollColumn = 0;
      return;
    }
    if (this.cursor.column < this.scrollColumn) {
      this.scrollColumn = this.cursor.column;
    } else if (this.cursor.column >= this.scrollColumn + width) {
      this.scrollColumn = this.cursor.column - width + 1;
    }
  }

  private renderLine(index: number, width: number): string {
    let remaining = index;
    for (let lineIndex = this.scrollLine; lineIndex < this.lines.length; lineIndex += 1) {
      const line = this.lines[lineIndex];
      const chunks =
        this.options.softwrap && !this.options.noWrap
          ? Math.max(1, Math.ceil(line.length / width))
          : 1;
      if (remaining >= chunks) {
        remaining -= chunks;
        continue;
      }
      const start =
        this.options.softwrap && !this.options.noWrap ? remaining * width : this.scrollColumn;
      const selection = this.selection();
      const value = line
        .slice(start, start + width)
        .map((char, offset) => {
          const position = { line: lineIndex, column: start + offset };
          const selected =
            selection &&
            this.compare(position, selection[0]) >= 0 &&
            this.compare(position, selection[1]) < 0;
          return selected ? '\x1b[7m' + char + '\x1b[0m' : char;
        })
        .join('');
      if (!this.options.lineNumbers) {
        return value;
      }
      const prefix = remaining === 0 ? String(lineIndex + 1).padStart(5, ' ') + ' ' : '      ';
      return prefix + value;
    }
    return '';
  }

  private visualCursorRow(width: number): number {
    let row = 0;
    for (let line = this.scrollLine; line < this.cursor.line; line += 1) {
      row += this.options.noWrap ? 1 : Math.max(1, Math.ceil(this.lines[line].length / width));
    }
    if (!this.options.noWrap) {
      row += Math.floor(this.cursor.column / width);
    }
    return row;
  }

  private statusLine(width: number): string {
    if (this.promptMode === 'confirm') {
      return this.statusMessage.slice(0, width);
    }
    if (this.promptMode === 'write') {
      return ('File Name to Write: ' + this.promptText).slice(0, width);
    }
    if (this.promptMode === 'search') {
      return (this.statusMessage || 'Search') + ': ' + this.promptText;
    }
    if (this.promptMode === 'replace') {
      return 'Search (to replace): ' + this.promptText;
    }
    if (this.promptMode === 'read') {
      if (this.statusMessage === 'Replace with:') {
        return 'Replace with: ' + this.promptText;
      }
      return 'File to Read: ' + this.promptText;
    }
    if (this.promptMode === 'help') {
      return 'Help: ^X Exit help  ^Y Page Up  ^V Page Down';
    }
    if (this.statusMessage) {
      return this.statusMessage.slice(0, width);
    }
    const lines = this.lines.length + (this.lines.length === 1 ? ' line' : ' lines');
    const position = this.options.constantShow
      ? 'line ' + (this.cursor.line + 1) + ', column ' + (this.cursor.column + 1)
      : '';
    return ('[ ' + (this.dirty ? 'Modified ' : 'Read ') + lines + ' ] ' + position).slice(0, width);
  }

  private centerTitle(left: string, right: string): string {
    const width = this.terminal.cols;
    if (left.length + right.length + 1 >= width) {
      return (left + ' ' + right).slice(0, width);
    }
    const spaces = Math.max(1, width - left.length - right.length);
    const leftPadding = Math.floor(spaces / 2);
    return ' '.repeat(leftPadding) + left + ' '.repeat(spaces - leftPadding) + right;
  }
}
