import type { KeyboardEvent } from 'react';

export const fileKeyboardShortcuts = [
  { label: 'Criar pasta', keys: 'Ctrl+Shift+N' },
  { label: 'Criar arquivo', keys: 'Ctrl+N' },
  { label: 'Copiar seleção', keys: 'Ctrl+C' },
  { label: 'Recortar seleção', keys: 'Ctrl+X' },
  { label: 'Colar na pasta atual', keys: 'Ctrl+V' },
  { label: 'Selecionar todos', keys: 'Ctrl+A' },
  { label: 'Selecionar item', keys: 'Setas' },
  { label: 'Abrir seleção', keys: 'Enter' },
  { label: 'Renomear item', keys: 'F2' },
  { label: 'Enviar seleção à lixeira', keys: 'Delete' },
];

export function fileKeyboardAction(event: KeyboardEvent<HTMLElement>) {
  if (
    event.defaultPrevented ||
    event.nativeEvent.isComposing ||
    event.altKey ||
    event.metaKey ||
    (event.target instanceof HTMLElement &&
      event.target.closest('input,textarea,select,[contenteditable="true"],[aria-modal="true"]'))
  ) {
    return;
  }
  const key = event.key.toLowerCase();
  if (event.ctrlKey) {
    if (key === 'n') {
      return event.shiftKey ? 'folder' : 'file';
    }
    if (event.shiftKey) {
      return;
    }
    return ({ c: 'copy', x: 'cut', v: 'paste', a: 'all' } as const)[key as 'c' | 'x' | 'v' | 'a'];
  }
  if (event.shiftKey) {
    return;
  }
  const actions = {
    f2: 'rename',
    delete: 'trash',
    enter: 'open',
    arrowup: 'previous',
    arrowleft: 'previous',
    arrowdown: 'next',
    arrowright: 'next',
  } as const;
  return actions[key as keyof typeof actions];
}
