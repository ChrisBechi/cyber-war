import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { perform, useGame } from '../../lib/game-store';
import type * as GameStore from '../../lib/game-store';
import type { World } from '../../lib/api';
import { BLACKWIRE_ONION_ADDRESS, defaultPreferences } from './browser-model';
import { useWindows } from '../../lib/window-store';
import { Browser } from './Browser';

vi.mock('../../lib/game-store', async (original) => ({
  ...(await original<typeof GameStore>()),
  perform: vi.fn(),
}));
beforeEach(() => {
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

it('shows navigation errors without opening real URLs or losing the entered address', async () => {
  vi.mocked(perform).mockRejectedValueOnce(
    new Error('Endereço não encontrado na internet do jogo.'),
  );
  render(<Browser initialAddress="https://example.com" />);
  expect(await screen.findByRole('alert')).toHaveTextContent('Endereço não encontrado');
  expect(screen.getByRole('textbox', { name: 'Endereço' })).toHaveValue('https://example.com');
  expect(perform).toHaveBeenCalledWith(
    'browser_navigate',
    { address: 'https://example.com' },
    expect.anything(),
  );
  expect(defaultPreferences.bookmarks).toHaveLength(6);
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
