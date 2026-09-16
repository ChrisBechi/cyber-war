export type DesktopAction =
  | 'terminal'
  | 'files'
  | 'browser'
  | 'editor'
  | 'settings'
  | 'processes'
  | 'menu'
  | 'help'
  | 'lock'
  | 'desktop'
  | 'maximize'
  | 'minimize'
  | 'next'
  | 'previous'
  | 'workspace-1'
  | 'workspace-2'
  | 'workspace-3'
  | 'workspace-4';

type Shortcut = {
  action: DesktopAction;
  group: string;
  label: string;
  keys: string;
  key: string;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
};

export const desktopKeyboardShortcuts: Shortcut[] = [
  {
    action: 'terminal',
    group: 'Aplicativos',
    label: 'Novo terminal',
    keys: 'Ctrl+Alt+T',
    key: 't',
    ctrl: true,
    alt: true,
  },
  {
    action: 'files',
    group: 'Aplicativos',
    label: 'Abrir Arquivos',
    keys: 'Ctrl+Alt+E',
    key: 'e',
    ctrl: true,
    alt: true,
  },
  {
    action: 'browser',
    group: 'Aplicativos',
    label: 'Abrir navegador',
    keys: 'Ctrl+Alt+B',
    key: 'b',
    ctrl: true,
    alt: true,
  },
  {
    action: 'editor',
    group: 'Aplicativos',
    label: 'Abrir HackPad',
    keys: 'Ctrl+Alt+H',
    key: 'h',
    ctrl: true,
    alt: true,
  },
  {
    action: 'settings',
    group: 'Aplicativos',
    label: 'Abrir configurações',
    keys: 'Ctrl+Alt+S',
    key: 's',
    ctrl: true,
    alt: true,
  },
  {
    action: 'processes',
    group: 'Aplicativos',
    label: 'Abrir Task Manager',
    keys: 'Ctrl+Alt+P',
    key: 'p',
    ctrl: true,
    alt: true,
  },
  {
    action: 'next',
    group: 'Janelas',
    label: 'Próxima janela',
    keys: 'Alt+PageDown',
    key: 'pagedown',
    alt: true,
  },
  {
    action: 'previous',
    group: 'Janelas',
    label: 'Janela anterior',
    keys: 'Alt+PageUp',
    key: 'pageup',
    alt: true,
  },
  {
    action: 'maximize',
    group: 'Janelas',
    label: 'Maximizar / restaurar janela',
    keys: 'Ctrl+Alt+M',
    key: 'm',
    ctrl: true,
    alt: true,
  },
  {
    action: 'minimize',
    group: 'Janelas',
    label: 'Minimizar janela',
    keys: 'Ctrl+Alt+N',
    key: 'n',
    ctrl: true,
    alt: true,
  },
  {
    action: 'desktop',
    group: 'Janelas',
    label: 'Mostrar desktop / restaurar janelas',
    keys: 'Ctrl+Alt+D',
    key: 'd',
    ctrl: true,
    alt: true,
  },
  {
    action: 'workspace-1',
    group: 'Sistema',
    label: 'Área de trabalho 1',
    keys: 'Ctrl+Alt+1',
    key: '1',
    ctrl: true,
    alt: true,
  },
  {
    action: 'workspace-2',
    group: 'Sistema',
    label: 'Área de trabalho 2',
    keys: 'Ctrl+Alt+2',
    key: '2',
    ctrl: true,
    alt: true,
  },
  {
    action: 'workspace-3',
    group: 'Sistema',
    label: 'Área de trabalho 3',
    keys: 'Ctrl+Alt+3',
    key: '3',
    ctrl: true,
    alt: true,
  },
  {
    action: 'workspace-4',
    group: 'Sistema',
    label: 'Área de trabalho 4',
    keys: 'Ctrl+Alt+4',
    key: '4',
    ctrl: true,
    alt: true,
  },
  {
    action: 'lock',
    group: 'Sistema',
    label: 'Bloquear sessão',
    keys: 'Ctrl+Alt+L',
    key: 'l',
    ctrl: true,
    alt: true,
  },
  {
    action: 'menu',
    group: 'Sistema',
    label: 'Menu de aplicativos',
    keys: 'Ctrl+Esc',
    key: 'escape',
    ctrl: true,
  },
  { action: 'help', group: 'Sistema', label: 'Mostrar atalhos de teclado', keys: 'F1', key: 'f1' },
];

export function desktopKeyboardAction(event: KeyboardEvent): DesktopAction | undefined {
  if (event.isComposing || event.getModifierState('AltGraph')) {
    return;
  }
  if (event.key === 'Meta' && !event.ctrlKey && !event.altKey && !event.shiftKey) {
    return 'menu';
  }
  if (event.metaKey) {
    return;
  }
  if (event.altKey && !event.ctrlKey && event.key === 'Tab') {
    return event.shiftKey ? 'previous' : 'next';
  }
  return desktopKeyboardShortcuts.find(
    (shortcut) =>
      shortcut.key === event.key.toLowerCase() &&
      !!shortcut.ctrl === event.ctrlKey &&
      !!shortcut.alt === event.altKey &&
      !!shortcut.shift === event.shiftKey,
  )?.action;
}
