import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { perform, useGame } from '../../lib/game-store';
import type * as GameStore from '../../lib/game-store';
import type { World } from '../../lib/api';
import { request } from '../../lib/api';
import type * as Api from '../../lib/api';
import { BLACKWIRE_ONION_ADDRESS, defaultPreferences } from './browser-model';
import { useWindows } from '../../lib/window-store';
import { Browser } from './Browser';

vi.mock('../../lib/game-store', async (original) => ({
  ...(await original<typeof GameStore>()),
  perform: vi.fn(),
}));
vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  request: vi.fn(),
}));
beforeEach(() => {
  vi.mocked(request).mockReset();
  useGame.setState({ world: { settings: {}, vfs: { nodes: {} } } as World, busy: false });
  useWindows.getState().reset();
  vi.mocked(perform).mockImplementation((command, args) => {
    if (command === 'browser_preferences_save') {
      useGame.setState({
        world: {
          ...useGame.getState().world!,
          settings: { browserPreferences: JSON.stringify(args.preferences) },
        },
      });
      return Promise.resolve(null);
    }
    return Promise.resolve({ title: String(args.address), body: 'Virtual', action: null });
  });
});

it('opens virtual text files as escaped content and supports reload and navigation back', async () => {
  const path = '/home/kali/Documents/hello world.html';
  const file = { id: path, name: 'hello world.html', kind: 'file', metadata: {} };
  vi.mocked(request).mockImplementation((command) =>
    Promise.resolve(command === 'vfs_stat' ? file : '<script>window.hostAccess = true</script>'),
  );
  render(<Browser initialAddress="file:///home/kali/Documents/hello%20world.html" />);
  await screen.findByRole('heading', { name: 'hello world.html' });
  expect(screen.getByText('<script>window.hostAccess = true</script>')).toBeVisible();
  expect(document.querySelector('.browser-local-file script')).toBeNull();
  expect(request).toHaveBeenCalledWith('vfs_read', { path }, expect.anything());
  expect(perform).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Recarregar página' }));
  await screen.findByRole('heading', { name: 'hello world.html' });
  await visit('https://www.wipedia.org');
  fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
  await screen.findByRole('heading', { name: 'hello world.html' });
  expect(screen.getByRole('textbox', { name: 'Endereço' })).toHaveValue(
    'file:///home/kali/Documents/hello%20world.html',
  );
});

it('previews a virtual image through VFS reads and shows file errors without a server error page', async () => {
  const path = '/home/kali/kali-waves.png';
  vi.mocked(request).mockImplementation((command) =>
    Promise.resolve(
      command === 'vfs_stat'
        ? { id: path, name: 'kali-waves.png', kind: 'file', metadata: {} }
        : '',
    ),
  );
  const { rerender } = render(<Browser initialAddress={`file://${path}`} />);
  expect(await screen.findByRole('img', { name: 'kali-waves.png' })).toHaveAttribute(
    'src',
    '/assets/kali-waves.png',
  );
  vi.mocked(request).mockRejectedValue(new Error('permission denied'));
  rerender(<Browser initialAddress="file:///root/private.txt" navigationId={1} />);
  await screen.findByRole('heading', { name: 'Arquivo indisponível' });
  expect(screen.getByRole('alert')).toHaveTextContent('permission denied');
  expect(screen.queryByRole('heading', { name: 'Servidor não encontrado' })).toBeNull();
  expect(perform).not.toHaveBeenCalled();
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

async function visit(address: string) {
  const input = screen.getByRole('textbox', { name: 'Endereço' });
  fireEvent.change(input, { target: { value: address } });
  fireEvent.submit(input.closest('form')!);
  await screen.findByRole('heading', { name: address });
}
function menuItem(name: string) {
  fireEvent.click(screen.getByRole('button', { name: 'Menu do navegador' }));
  fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name }));
}

it('keeps tab histories independent and supports back/forward/reload and closing the last tab', async () => {
  render(<Browser />);
  await visit('https://www.wipedia.org');
  await visit('https://www.archive.org');
  fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
  await screen.findByRole('heading', { name: 'https://www.wipedia.org' });
  fireEvent.click(screen.getByRole('button', { name: 'Avançar' }));
  await screen.findByRole('heading', { name: 'https://www.archive.org' });
  fireEvent.click(screen.getByRole('button', { name: 'Nova aba' }));
  expect(screen.getByRole('button', { name: 'Voltar' })).toBeDisabled();
  await visit('https://www.mercado.com.br');
  fireEvent.click(screen.getByRole('tab', { name: 'https://www.archive.org' }));
  expect(screen.getByRole('heading', { name: 'https://www.archive.org' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Recarregar página' }));
  await screen.findByRole('heading', { name: 'https://www.archive.org' });
  fireEvent.click(screen.getByRole('button', { name: 'Fechar aba https://www.archive.org' }));
  expect(screen.getByRole('tab', { selected: true })).toHaveTextContent(
    'https://www.mercado.com.br',
  );
  fireEvent.click(screen.getByRole('button', { name: 'Fechar aba https://www.mercado.com.br' }));
  expect(screen.getAllByRole('tab')).toHaveLength(1);
  expect(screen.getByRole('tab')).toHaveTextContent('Nova aba');
});

it('ignores stale responses and does not resurrect a closed tab', async () => {
  let resolve!: (value: unknown) => void;
  vi.mocked(perform).mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  render(<Browser initialAddress="https://www.wipedia.org" />);
  await visit('https://www.archive.org');
  await act(async () => {
    resolve({ title: 'stale', body: '', action: null });
    await Promise.resolve();
  });
  expect(screen.queryByRole('heading', { name: 'stale' })).toBeNull();
  vi.mocked(perform).mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  fireEvent.click(screen.getByRole('button', { name: 'Nova aba' }));
  const input = screen.getByRole('textbox', { name: 'Endereço' });
  fireEvent.change(input, { target: { value: 'https://www.fakebook.com' } });
  fireEvent.submit(input.closest('form')!);
  fireEvent.click(screen.getByRole('button', { name: 'Fechar aba https://www.fakebook.com' }));
  await act(async () => {
    resolve({ title: 'closed', body: '', action: null });
    await Promise.resolve();
  });
  expect(screen.getAllByRole('tab')).toHaveLength(1);
  expect(screen.getByRole('heading', { name: 'https://www.archive.org' })).toBeInTheDocument();
});

it('saves edited bookmarks and preferences, keeps drafts on failure and starts a fresh tab on remount', async () => {
  const view = render(<Browser />);
  await visit('https://www.wipedia.org');
  fireEvent.click(screen.getByRole('button', { name: 'Editar favorito' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Nome do favorito' }), {
    target: { value: 'Biblioteca' },
  });
  vi.mocked(perform).mockRejectedValueOnce(new Error('disk full'));
  fireEvent.click(screen.getByRole('button', { name: 'Salvar favorito' }));
  await screen.findByRole('alert');
  expect(screen.getByRole('textbox', { name: 'Nome do favorito' })).toHaveValue('Biblioteca');
  fireEvent.click(screen.getByRole('button', { name: 'Salvar favorito' }));
  await waitFor(() =>
    expect(screen.queryByRole('textbox', { name: 'Nome do favorito' })).toBeNull(),
  );
  fireEvent.pointerDown(document.body);
  menuItem('Configurações');
  fireEvent.change(screen.getByRole('textbox', { name: 'Endereço da página inicial' }), {
    target: { value: 'https://www.archive.org' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Salvar página inicial' }));
  await waitFor(() =>
    expect(JSON.parse(useGame.getState().world!.settings.browserPreferences)).toMatchObject({
      home: 'https://www.archive.org',
    }),
  );
  view.unmount();
  render(<Browser />);
  expect(screen.getByRole('tab')).toHaveTextContent('Nova aba');
  expect(
    within(screen.getByRole('navigation', { name: 'Barra de favoritos' })).getByRole('button', {
      name: 'Biblioteca',
    }),
  ).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Página inicial' }));
  await screen.findByRole('heading', { name: 'https://www.archive.org' });
});

it('adds/removes a favorite, adjusts zoom and hides the bookmarks bar without changing page content', async () => {
  render(<Browser />);
  await visit(`https://${BLACKWIRE_ONION_ADDRESS}`);
  fireEvent.click(screen.getByRole('button', { name: 'Adicionar aos favoritos' }));
  fireEvent.click(screen.getByRole('button', { name: 'Salvar favorito' }));
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Editar favorito' })).toBeEnabled(),
  );
  fireEvent.click(
    screen.getByRole('button', { name: `Remover https://${BLACKWIRE_ONION_ADDRESS}` }),
  );
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Adicionar aos favoritos' })).toBeEnabled(),
  );
  fireEvent.pointerDown(document.body);
  fireEvent.click(screen.getByRole('button', { name: 'Menu do navegador' }));
  fireEvent.click(screen.getByRole('button', { name: 'Aumentar zoom' }));
  // jsdom 26 retains style.zoom but does not expose it through getComputedStyle.
  await waitFor(() =>
    expect(document.querySelector<HTMLElement>('.browser-zoom-page')?.style.zoom).toBe('110%'),
  );
  expect(JSON.parse(useGame.getState().world!.settings.browserPreferences)).toMatchObject({
    zoom: 110,
  });
  fireEvent.click(screen.getByRole('button', { name: 'Configurações' }));
  fireEvent.click(screen.getByRole('checkbox', { name: 'Mostrar barra de favoritos' }));
  await waitFor(() =>
    expect(screen.queryByRole('navigation', { name: 'Barra de favoritos' })).toBeNull(),
  );
  expect(
    screen.getByRole('heading', { name: `https://${BLACKWIRE_ONION_ADDRESS}` }),
  ).toBeInTheDocument();
});

it('dismisses menus outside, supports shortcuts and exposes only virtual downloads/extensions', async () => {
  render(<Browser />);
  menuItem('Downloads');
  fireEvent.click(screen.getByRole('button', { name: 'Abrir pasta Downloads' }));
  expect(useWindows.getState().windows[0]).toMatchObject({
    id: 'files',
    path: '/home/kali/Downloads',
  });
  fireEvent.click(screen.getByRole('button', { name: 'Extensões' }));
  expect(screen.getByText(/Nenhuma extensão instalada/)).toBeInTheDocument();
  fireEvent.pointerDown(document.body);
  expect(screen.queryByRole('dialog')).toBeNull();
  fireEvent.keyDown(screen.getByRole('textbox', { name: 'Endereço' }), { ctrlKey: true, key: 't' });
  expect(screen.getAllByRole('tab')).toHaveLength(2);
  fireEvent.keyDown(screen.getByRole('textbox', { name: 'Endereço' }), { ctrlKey: true, key: 'w' });
  expect(screen.getAllByRole('tab')).toHaveLength(1);
  fireEvent.focus(screen.getByRole('textbox', { name: 'Endereço' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'Endereço' }), {
    target: { value: 'arch' },
  });
  fireEvent.click(
    within(document.querySelector('.browser-suggestions') as HTMLElement).getByRole('button'),
  );
  await screen.findByRole('heading', { name: 'https://www.archive.org' });
});

it('shows the server-not-found page with the failed hostname and local help', async () => {
  vi.mocked(perform).mockImplementationOnce(() => {
    const error = new Error('Endereço não encontrado na internet do jogo.');
    useGame.setState({ error: String(error) });
    return Promise.reject(error);
  });
  render(<Browser initialAddress="https://www.kikolouro.com.br/music?album=1" />);
  expect(await screen.findByRole('alert')).toHaveTextContent('Servidor não encontrado');
  expect(screen.getByRole('alert')).toHaveTextContent('www.kikolouro.com.br.');
  expect(useGame.getState().error).toBe('');
  expect(screen.queryByText('Uma janela para o mundo.')).toBeNull();
  expect(screen.getByRole('textbox', { name: 'Endereço' })).toHaveValue(
    'https://www.kikolouro.com.br/music?album=1',
  );
  expect(perform).toHaveBeenCalledWith(
    'browser_navigate',
    { address: 'https://www.kikolouro.com.br/music?album=1' },
    expect.anything(),
  );
  fireEvent.click(screen.getByRole('button', { name: 'Saiba mais...' }));
  expect(screen.getByText('Endereço não encontrado na internet do jogo.')).toBeVisible();
  expect(perform).toHaveBeenCalledTimes(1);
  expect(defaultPreferences.bookmarks).toHaveLength(7);
});

it('retries the failed URL without adding history and isolates the error to its tab', async () => {
  render(<Browser />);
  await visit('https://www.wipedia.org');
  vi.mocked(perform).mockRejectedValueOnce(new Error('Endereço não encontrado.'));
  const address = 'https://www.example.com/path?retry=1';
  const input = screen.getByRole('textbox', { name: 'Endereço' });
  fireEvent.change(input, { target: { value: address } });
  fireEvent.submit(input.closest('form')!);
  await screen.findByRole('heading', { name: 'Servidor não encontrado' });
  fireEvent.click(screen.getByRole('button', { name: 'Nova aba' }));
  expect(screen.queryByRole('heading', { name: 'Servidor não encontrado' })).toBeNull();
  expect(screen.getByText('Uma janela para o mundo.')).toBeVisible();
  fireEvent.click(screen.getByRole('tab', { name: address }));
  fireEvent.click(screen.getByRole('button', { name: 'Tentar novamente' }));
  await screen.findByRole('heading', { name: address });
  expect(perform).toHaveBeenLastCalledWith('browser_navigate', { address }, expect.anything());
  expect(screen.queryByRole('heading', { name: 'Servidor não encontrado' })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Voltar' }));
  await screen.findByRole('heading', { name: 'https://www.wipedia.org' });
});

it('opens the virtual page source from the developer menu without executing it', async () => {
  vi.mocked(perform).mockResolvedValueOnce({
    title: 'Biblioteca',
    body: '<script>alert("unsafe")</script>',
    action: 'recover',
  });
  render(<Browser initialAddress="https://www.wipedia.org" />);
  await screen.findByRole('heading', { name: 'Biblioteca' });
  fireEvent.change(screen.getByRole('textbox', { name: 'Código de recuperação' }), {
    target: { value: 'private-value' },
  });
  menuItem('Exibir código fonte');
  expect(screen.getByRole('tab', { name: 'Código fonte' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  const source = screen.getByRole('textbox', { name: 'Código fonte da página' });
  expect(source).toHaveAttribute('readonly');
  const markup = (source as HTMLTextAreaElement).value;
  expect(markup).toContain('<!doctype html>');
  expect(markup).toContain('<h1>Biblioteca</h1>');
  expect(markup).toContain('&lt;script&gt;');
  expect(markup).toContain('Código correlacionado');
  expect(markup).not.toContain('private-value');
  expect(markup).not.toContain('browser-navigation');
  expect(document.querySelector('.browser-developer-source script')).toBeNull();
  await visit('https://www.archive.org');
  await waitFor(() =>
    expect(
      screen.getByRole<HTMLTextAreaElement>('textbox', { name: 'Código fonte da página' }).value,
    ).toContain('<h1>https://www.archive.org</h1>'),
  );
});

it('records actual navigation and action outcomes in Network and keeps developer tools open', async () => {
  vi.mocked(perform).mockResolvedValueOnce({
    title: 'Downloads',
    body: 'Download page',
    action: 'download',
  });
  render(<Browser initialAddress="https://www.archive.org" />);
  await screen.findByRole('heading', { name: 'Downloads' });
  fireEvent.click(screen.getByRole('button', { name: 'Baixar Cyber Siege' }));
  await screen.findByRole('heading', { name: 'https://www.archive.org' });
  menuItem('Modo desenvolvedor');
  fireEvent.click(screen.getByRole('tab', { name: 'Rede' }));
  let network = screen.getByRole('tabpanel', { name: 'Rede' });
  expect(within(network).getAllByText('Concluída')).toHaveLength(3);
  expect(within(network).getByText('download')).toBeVisible();
  vi.mocked(perform).mockRejectedValueOnce(new Error('Sem conexão de rede.'));
  const input = screen.getByRole('textbox', { name: 'Endereço' });
  fireEvent.change(input, { target: { value: 'https://www.missing.local' } });
  fireEvent.submit(input.closest('form')!);
  await screen.findByRole('heading', { name: 'Servidor não encontrado' });
  network = screen.getByRole('tabpanel', { name: 'Rede' });
  expect(within(network).getByText('Falhou')).toBeVisible();
  fireEvent.click(within(network).getByRole('button', { name: 'https://www.missing.local' }));
  expect(within(network).getByText('Sem conexão de rede.', { selector: 'dd' })).toBeVisible();
  expect(network).not.toHaveTextContent('404');
  fireEvent.change(within(network).getByRole('textbox', { name: 'Filtrar requisições' }), {
    target: { value: 'missing' },
  });
  expect(within(network).queryByText('download', { selector: 'td' })).toBeNull();
  fireEvent.click(within(network).getByRole('button', { name: 'Limpar registros' }));
  expect(within(network).getByText('Nenhuma requisição encontrada.')).toBeVisible();
});

it('runs bounded console queries against the page, preserves command history, and rejects host access', async () => {
  render(<Browser />);
  await visit('https://www.wipedia.org');
  fireEvent.keyDown(screen.getByRole('textbox', { name: 'Endereço' }), { key: 'F12' });
  const command = screen.getByRole('textbox', { name: 'Comando do console' });
  const run = (value: string) => {
    fireEvent.change(command, { target: { value } });
    fireEvent.submit(command.closest('form')!);
  };
  run("document.querySelector('h1').textContent");
  let output = screen.getByRole('list', { name: 'Resultados do console' });
  expect(output).toHaveTextContent('https://www.wipedia.org');
  run('document.querySelectorAll("h1").length');
  expect(within(output).getByText('1')).toBeVisible();
  run('document.querySelector(".browser-navigation")');
  expect(within(output).getByText('null')).toBeVisible();
  run('window.__TAURI_INTERNALS__.invoke("quit_game")');
  expect(output).toHaveTextContent('Expressão não suportada');
  expect(perform).toHaveBeenCalledTimes(1);
  fireEvent.keyDown(command, { key: 'ArrowUp' });
  expect(command).toHaveValue('window.__TAURI_INTERNALS__.invoke("quit_game")');
  run('console.clear()');
  output = screen.getByRole('list', { name: 'Resultados do console' });
  expect(output).toBeEmptyDOMElement();
  run('console.log("<script>test</script>")');
  expect(output).toHaveTextContent('<script>test</script>');
  expect(output.querySelector('script')).toBeNull();
  fireEvent.click(screen.getByRole('tab', { name: 'Rede' }));
  expect(screen.getByRole('table')).toHaveTextContent('https://www.wipedia.org');
  fireEvent.click(screen.getByRole('tab', { name: 'Console' }));
  expect(output).toHaveTextContent('<script>test</script>');
  fireEvent.click(screen.getByRole('button', { name: 'Nova aba' }));
  await visit('https://www.archive.org');
  expect(screen.getByRole('list', { name: 'Resultados do console' })).toBeEmptyDOMElement();
  expect(screen.getByRole('list', { name: 'Eventos de navegação' })).not.toHaveTextContent(
    'wipedia',
  );
  fireEvent.click(screen.getByRole('tab', { name: 'Rede' }));
  expect(screen.getByRole('table')).not.toHaveTextContent('wipedia');
  fireEvent.click(screen.getByRole('button', { name: 'Limpar registros' }));
  fireEvent.click(screen.getByRole('tab', { name: 'https://www.wipedia.org' }));
  expect(screen.getByRole('table')).toHaveTextContent('https://www.wipedia.org');
  fireEvent.click(screen.getByRole('tab', { name: 'Console' }));
  expect(screen.getByRole('list', { name: 'Resultados do console' })).toHaveTextContent(
    '<script>test</script>',
  );
});

it('exposes persisted preferences and temporary session state and supports history/reload/source shortcuts', async () => {
  render(<Browser />);
  await visit('https://www.wipedia.org');
  const address = screen.getByRole('textbox', { name: 'Endereço' });
  fireEvent.keyDown(address, { ctrlKey: true, key: 'h' });
  expect(screen.getByText('Histórico desta sessão')).toBeVisible();
  fireEvent.click(screen.getByRole('button', { name: 'Limpar histórico da sessão' }));
  expect(screen.getByText('Nenhuma página visitada.')).toBeVisible();
  fireEvent.keyDown(address, { key: 'F5' });
  await screen.findByRole('heading', { name: 'https://www.wipedia.org' });
  expect(perform).toHaveBeenCalledTimes(3);
  fireEvent.keyDown(address, { ctrlKey: true, key: 'u' });
  expect(screen.getByRole('textbox', { name: 'Código fonte da página' })).toBeVisible();
  fireEvent.click(screen.getByRole('tab', { name: 'Storage' }));
  const storage = screen.getByRole('tabpanel', { name: 'Storage' });
  expect(storage).toHaveTextContent('7 favoritos');
  expect(storage).toHaveTextContent('Configuração padrão');
  fireEvent.click(screen.getByRole('tab', { name: 'Sessão' }));
  expect(screen.getByRole('tabpanel', { name: 'Sessão' })).toHaveTextContent('1 página(s)');
  fireEvent.click(screen.getByRole('button', { name: 'Fechar modo desenvolvedor' }));
  expect(screen.queryByRole('region', { name: 'Modo desenvolvedor' })).toBeNull();
});

it('reopens a URL shortcut after browsing elsewhere, even when its address is unchanged', async () => {
  vi.mocked(perform).mockImplementation((_command, args) =>
    Promise.resolve({ title: String(args.address), body: 'Virtual', action: null }),
  );
  const { rerender } = render(
    <Browser initialAddress="https://www.wipedia.org" navigationId={1} />,
  );
  await waitFor(() =>
    expect(screen.getByRole('heading', { name: 'https://www.wipedia.org' })).toBeInTheDocument(),
  );
  fireEvent.click(screen.getByRole('button', { name: 'archive' }));
  await waitFor(() =>
    expect(screen.getByRole('heading', { name: 'https://www.archive.org' })).toBeInTheDocument(),
  );
  rerender(<Browser initialAddress="https://www.wipedia.org" navigationId={2} />);
  await waitFor(() =>
    expect(screen.getByRole('heading', { name: 'https://www.wipedia.org' })).toBeInTheDocument(),
  );
  expect(vi.mocked(perform).mock.calls.map(([command, args]) => [command, args.address])).toEqual([
    ['browser_navigate', 'https://www.wipedia.org'],
    ['browser_navigate', 'https://www.archive.org'],
    ['browser_navigate', 'https://www.wipedia.org'],
  ]);
});
