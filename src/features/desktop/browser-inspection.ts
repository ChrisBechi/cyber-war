export type BrowserRequest = {
  id: number;
  tabId: number;
  address: string;
  operation: string;
  startedAt: number;
  status: 'pending' | 'success' | 'error' | 'blocked';
  durationMs: number | null;
  error: string;
};

export const requestStatus: Record<BrowserRequest['status'], string> = {
  pending: 'Pendente',
  success: 'Concluída',
  error: 'Falhou',
  blocked: 'Bloqueada',
};

/** Serialize only the virtual page. Editable values never become source code. */
function sourceClone(container: Element) {
  const copy = container.cloneNode(true) as HTMLElement;
  if (copy.matches('input')) {
    copy.removeAttribute('value');
  }
  if (copy.matches('textarea')) {
    copy.textContent = '';
  }
  copy.querySelectorAll('input').forEach((input) => input.removeAttribute('value'));
  copy.querySelectorAll('textarea').forEach((textarea) => {
    textarea.textContent = '';
  });
  return copy;
}

export function serializeBrowserPage(container: HTMLElement, title: string) {
  const copy = sourceClone(container);
  const heading = document.createElement('title');
  heading.textContent = title;
  return [
    '<!doctype html>',
    '<html lang="pt-BR">',
    '<head>',
    '  <meta charset="utf-8">',
    `  ${heading.outerHTML}`,
    '</head>',
    '<body>',
    copy.innerHTML.replace(/></g, '>\n<'),
    '</body>',
    '</html>',
  ].join('\n');
}

export const consoleHelp = [
  'document.title · document.URL · location.href · location.hostname',
  'document.body.innerHTML · document.body.textContent',
  'document.querySelector("h1") · document.querySelector("h1").textContent',
  'document.querySelectorAll("button").length',
  'console.log("mensagem") · console.clear() · help',
  'Consultas ao HTML exibido. Expressões JavaScript fora desta lista não são suportadas.',
].join('\n');

/** Deliberately bounded console grammar; never evaluate code in the Tauri WebView. */
export function evaluateBrowserConsole(
  command: string,
  context: {
    title: string;
    address: string;
    element: HTMLElement | null;
  },
): string {
  const expression = command.trim().replace(/;$/, '').trim();
  if (expression === 'help') {
    return consoleHelp;
  }
  if (expression === 'document.title') {
    return JSON.stringify(context.title);
  }
  if (expression === 'document.URL' || expression === 'location.href') {
    return JSON.stringify(context.address || 'about:newtab');
  }
  if (expression === 'location.hostname') {
    return JSON.stringify(context.address ? new URL(context.address).hostname : '');
  }
  if (expression === 'document.body.innerHTML' || expression === 'document.body.textContent') {
    if (!context.element) {
      return 'null';
    }
    const copy = sourceClone(context.element);
    return expression.endsWith('innerHTML') ? copy.innerHTML : copy.textContent || '';
  }
  const query =
    /^document\.(querySelector|querySelectorAll)\(("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')\)(?:\.(textContent|innerHTML|outerHTML|tagName|length))?$/.exec(
      expression,
    );
  if (query) {
    const [, method, literal, property] = query;
    const selector = readLiteral(literal);
    if (typeof selector !== 'string') {
      throw new Error('O seletor deve ser um texto.');
    }
    const copy = context.element ? sourceClone(context.element) : null;
    if (method === 'querySelectorAll') {
      const elements = Array.from(copy?.querySelectorAll(selector) ?? []);
      if (property === 'length') {
        return String(elements.length);
      }
      if (property) {
        throw new Error('Use .length com querySelectorAll.');
      }
      return elements.length ? elements.map((element) => element.outerHTML).join('\n') : '[]';
    }
    const element = copy?.querySelector(selector);
    if (!element) {
      return 'null';
    }
    if (property === 'length') {
      throw new Error('Use querySelectorAll para contar elementos.');
    }
    if (property === 'textContent') {
      return element.textContent ?? '';
    }
    if (property === 'innerHTML') {
      return element.innerHTML;
    }
    if (property === 'tagName') {
      return element.tagName;
    }
    return element.outerHTML;
  }
  const log = /^console\.log\(([\s\S]*)\)$/.exec(expression);
  if (log) {
    const value = readLiteral(log[1].trim());
    return typeof value === 'string' ? value : JSON.stringify(value, null, 2);
  }
  throw new Error('Expressão não suportada. Digite help para ver os comandos disponíveis.');
}

function readLiteral(literal: string): unknown {
  if (literal.startsWith("'") && literal.endsWith("'")) {
    return literal.slice(1, -1).replace(/\\(['\\])/g, '$1');
  }
  return JSON.parse(literal);
}
