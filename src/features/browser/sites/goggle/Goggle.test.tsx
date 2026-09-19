import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import { request } from '../../../../lib/api';
import { perform, useGame } from '../../../../lib/game-store';
import type * as Api from '../../../../lib/api';
import type * as GameStore from '../../../../lib/game-store';
import type { World } from '../../../../lib/api';
import { Browser } from '../../../desktop/Browser';
import { GoggleSite } from './GoggleSite';
import { SearchBox } from './SearchBox';
import { GOGGLE_HOME, goggleProvider, goggleRoute } from './goggle-model';
import type { GoggleSession, SearchResponse } from './goggle-model';

vi.mock('../../../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  request: vi.fn(),
}));
vi.mock('../../../../lib/game-store', async (original) => ({
  ...(await original<typeof GameStore>()),
  perform: vi.fn(),
}));
const navigate = vi.fn();
const page = { title: 'Goggle', body: 'Pesquisa virtual', action: null };
const doc = {
  id: 'orion',
  title: 'Orion Technologies — Sistemas inteligentes',
  description: 'Pesquisa e infraestrutura.',
  url: 'https://www.orion.com',
  domain: 'www.orion.com',
  type: 'COMPANY' as const,
  imageId: null,
};
let session: GoggleSession;
let response: SearchResponse;
beforeEach(() => {
  vi.clearAllMocks();
  session = { account: null, history: [], historyEnabled: false };
  response = { query: 'Orion', documents: [doc], total: 1, offset: 0, images: [] };
  useGame.setState({
    world: { settings: {}, vfs: { nodes: {} } } as World,
    revision: 0,
    busy: false,
  });
  vi.mocked(request).mockImplementation((command, args) => {
    if (command === 'goggle_session') {
      return Promise.resolve(structuredClone(session));
    }
    if (command === 'search_query') {
      return Promise.resolve({ ...response, query: args.query });
    }
    if (command === 'search_suggestions') {
      return Promise.resolve(['orion empresa', 'orion tecnologia']);
    }
    if (command === 'search_voice_options') {
      return Promise.resolve(['orion empresa', 'archive cyber siege']);
    }
    if (command === 'search_image_files') {
      return Promise.resolve(['/home/kali/Pictures/unknown.svg']);
    }
    return Promise.reject(new Error(`Unexpected command ${command}`));
  });
  vi.mocked(perform).mockImplementation((command, args) => {
    if (command === 'browser_navigate') {
      return Promise.resolve(page);
    }
    if (command === 'goggle_logout') {
      session.account = null;
    }
    if (command === 'goggle_authenticate') {
      session.account = {
        id: 'test',
        email: String(args.email),
        displayName: typeof args.displayName === 'string' ? args.displayName : 'Chris',
        avatar: null,
        apps: ['images', 'account'],
      };
    }
    if (command === 'goggle_preferences') {
      session.historyEnabled = Boolean(args.historyEnabled);
      if (args.clearHistory) {
        session.history = [];
      }
    }
    return Promise.resolve(null);
  });
});
const site = (address = GOGGLE_HOME) =>
  render(<GoggleSite address={address} page={page} navigate={navigate} />);
const input = () => screen.getByRole('combobox', { name: 'Pesquisar no Goggle' });

it('renders the complete light home and hides authenticated navigation', async () => {
  const { container } = site();
  expect(screen.getByRole('img', { name: 'Goggle' })).toHaveTextContent('Goggle');
  expect(input()).toBeInTheDocument();
  for (const name of [
    'Abrir teclado virtual',
    'Pesquisar por voz',
    'Pesquisar por imagem',
    'Pesquisar Goggle',
    'Estou com sorte',
    'Fazer login',
    'Sobre',
    'Como funciona a Pesquisa',
    'Privacidade',
    'Termos',
  ]) {
    expect(screen.getByRole('button', { name })).toBeVisible();
  }
  await waitFor(() =>
    expect(request).toHaveBeenCalledWith('goggle_session', {}, expect.anything()),
  );
  expect(screen.queryByRole('button', { name: 'Mail' })).toBeNull();
  expect(screen.queryByRole('button', { name: 'Imagens' })).toBeNull();
  expect(screen.queryByRole('button', { name: 'Aplicativos Goggle' })).toBeNull();
  expect(container.querySelector('input[type="file"]')).toBeNull();
  expect(container.querySelector('a[href^="http"]')).toBeNull();
});

it('submits through the form and button, preserving punctuation, case and accents in the URL', () => {
  site();
  fireEvent.change(input(), { target: { value: 'Orion & Ações' } });
  fireEvent.keyDown(input(), { key: 'Enter' });
  expect(navigate).toHaveBeenLastCalledWith(
    `${GOGGLE_HOME}/search?q=Orion%20%26%20A%C3%A7%C3%B5es`,
  );
  fireEvent.change(input(), { target: { value: 'tecnologia' } });
  fireEvent.click(screen.getByRole('button', { name: 'Pesquisar Goggle' }));
  expect(navigate).toHaveBeenLastCalledWith(goggleProvider.searchUrl('tecnologia'));
});

it('lucky navigates to the ranked first result or to a zero-result search', async () => {
  site();
  fireEvent.change(input(), { target: { value: 'Orion' } });
  fireEvent.click(screen.getByRole('button', { name: 'Estou com sorte' }));
  await waitFor(() => expect(navigate).toHaveBeenCalledWith(doc.url));
  response.documents = [];
  response.total = 0;
  fireEvent.change(input(), { target: { value: 'nothing' } });
  fireEvent.click(screen.getByRole('button', { name: 'Estou com sorte' }));
  await waitFor(() =>
    expect(navigate).toHaveBeenLastCalledWith(goggleProvider.searchUrl('nothing')),
  );
});

it('supports autocomplete selection with arrows and mouse and rejects stale responses', async () => {
  const search = vi.fn();
  render(<SearchBox search={search} imageSearch={vi.fn()} />);
  fireEvent.focus(input());
  fireEvent.change(input(), { target: { value: 'orion' } });
  await screen.findByRole('option', { name: 'orion empresa' });
  fireEvent.keyDown(input(), { key: 'ArrowDown' });
  fireEvent.keyDown(input(), { key: 'ArrowDown' });
  expect(screen.getByRole('option', { name: 'orion tecnologia' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  fireEvent.keyDown(input(), { key: 'Enter' });
  expect(search).toHaveBeenCalledWith('orion tecnologia', false);
  fireEvent.focus(input());
  fireEvent.change(input(), { target: { value: 'orion' } });
  fireEvent.click(await screen.findByRole('option', { name: 'orion empresa' }));
  expect(search).toHaveBeenCalledWith('orion empresa', false);
  let finish!: (value: string[]) => void;
  vi.mocked(request).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  fireEvent.focus(input());
  fireEvent.change(input(), { target: { value: 'old' } });
  await waitFor(() => expect(finish).toBeDefined());
  fireEvent.change(input(), { target: { value: '' } });
  await act(async () => {
    finish(['old private result']);
    await Promise.resolve();
  });
  expect(screen.queryByRole('option')).toBeNull();
});

it('virtual keyboard handles letters, numbers, shift, space, backspace and Enter', async () => {
  const search = vi.fn();
  render(<SearchBox search={search} imageSearch={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: 'Abrir teclado virtual' }));
  const keyboard = within(screen.getByRole('region', { name: 'Teclado virtual QWERTY' }));
  fireEvent.click(keyboard.getByRole('button', { name: 'a' }));
  await waitFor(() => expect(input()).toHaveValue('a'));
  fireEvent.click(keyboard.getByRole('button', { name: '1' }));
  await waitFor(() => expect(input()).toHaveValue('a1'));
  fireEvent.click(keyboard.getByRole('button', { name: 'Espaço' }));
  await waitFor(() => expect(input()).toHaveValue('a1 '));
  fireEvent.click(keyboard.getByRole('button', { name: 'Shift' }));
  fireEvent.click(keyboard.getByRole('button', { name: 'B' }));
  await waitFor(() => expect(input()).toHaveValue('a1 B'));
  fireEvent.click(keyboard.getByRole('button', { name: 'Backspace' }));
  await waitFor(() => expect(input()).toHaveValue('a1 '));
  fireEvent.click(keyboard.getByRole('button', { name: 'Enter' }));
  expect(search).toHaveBeenCalledWith('a1', false);
});

it('refreshes visible suggestions when narrative state changes', async () => {
  render(<SearchBox search={vi.fn()} imageSearch={vi.fn()} />);
  fireEvent.focus(input());
  fireEvent.change(input(), { target: { value: 'orion' } });
  await screen.findByRole('option', { name: 'orion empresa' });
  vi.mocked(request).mockResolvedValue([]);
  act(() => useGame.setState({ revision: 1 }));
  await waitFor(() => expect(screen.queryByRole('option')).toBeNull());
  await waitFor(() => expect(request).toHaveBeenCalledTimes(2));
});

it('simulates voice with narrative options without requesting a real microphone', async () => {
  const getUserMedia = vi.fn();
  Object.defineProperty(navigator, 'mediaDevices', { configurable: true, value: { getUserMedia } });
  site();
  fireEvent.click(screen.getByRole('button', { name: 'Pesquisar por voz' }));
  expect(screen.getByRole('dialog', { name: 'Pesquisar por voz' })).toBeVisible();
  fireEvent.click(await screen.findByRole('button', { name: 'orion empresa' }));
  expect(navigate).toHaveBeenCalledWith(goggleProvider.searchUrl('orion empresa'));
  expect(getUserMedia).not.toHaveBeenCalled();
});

it('image search selects only VFS files or submits a virtual URL to Rust', async () => {
  site();
  fireEvent.click(screen.getByRole('button', { name: 'Pesquisar por imagem' }));
  const dialog = screen.getByRole('dialog', { name: 'Pesquisar por imagem' });
  expect(dialog.querySelector('input[type="file"]')).toBeNull();
  fireEvent.click(
    await within(dialog).findByRole('button', { name: '/home/kali/Pictures/unknown.svg' }),
  );
  expect(navigate).toHaveBeenLastCalledWith(
    `${GOGGLE_HOME}/search?q=&image=%2Fhome%2Fkali%2FPictures%2Funknown.svg`,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Pesquisar por imagem' }));
  fireEvent.click(screen.getByRole('tab', { name: 'Usar URL' }));
  fireEvent.change(screen.getByRole('textbox', { name: 'URL da imagem virtual' }), {
    target: { value: 'https://www.orion.com/media/campus.svg' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Pesquisar imagem' }));
  expect(navigate).toHaveBeenLastCalledWith(
    `${GOGGLE_HOME}/search?q=&image=https%3A%2F%2Fwww.orion.com%2Fmedia%2Fcampus.svg`,
  );
});

it('authenticated header has exactly nine app dots, explicit app status and logout', async () => {
  session.account = {
    id: 'test',
    email: 'chris@goggle.com',
    displayName: 'Chris',
    avatar: 'https://google.com/avatar.jpg',
    apps: ['account', 'images'],
  };
  const { container } = site();
  await screen.findByRole('button', { name: 'Mail' });
  expect(screen.getByRole('button', { name: 'Imagens' })).toBeVisible();
  expect(container.querySelectorAll('.goggle-app-grid i')).toHaveLength(9);
  expect(container.querySelector('img[src^="http"]')).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Aplicativos Goggle' }));
  expect(screen.getByRole('button', { name: 'Goggle Mail Em breve' })).toBeVisible();
  fireEvent.click(screen.getByRole('button', { name: 'Goggle Images Disponível' }));
  expect(navigate).toHaveBeenLastCalledWith(`${GOGGLE_HOME}/images`);
  fireEvent.click(screen.getByRole('button', { name: 'Conta Goggle: Chris' }));
  expect(screen.getByRole('button', { name: 'Gerenciar sua conta' })).toBeVisible();
  fireEvent.click(screen.getByRole('button', { name: 'Sair' }));
  await screen.findByRole('button', { name: 'Fazer login' });
  expect(screen.queryByRole('button', { name: 'Mail' })).toBeNull();
});

it('creates a virtual account with the requested @goggle.com branding', async () => {
  site(`${GOGGLE_HOME}/login`);
  fireEvent.click(screen.getByRole('button', { name: 'Criar conta' }));
  fireEvent.change(screen.getByLabelText('Seu nome'), { target: { value: 'Chris' } });
  fireEvent.change(screen.getByLabelText('E-mail Goggle'), {
    target: { value: 'chris@goggle.com' },
  });
  fireEvent.change(screen.getByLabelText('Senha fictícia'), { target: { value: 'virtual-test' } });
  fireEvent.click(screen.getByRole('button', { name: 'Criar conta' }));
  await waitFor(() =>
    expect(perform).toHaveBeenCalledWith(
      'goggle_authenticate',
      { email: 'chris@goggle.com', password: 'virtual-test', displayName: 'Chris' },
      expect.anything(),
    ),
  );
  expect(navigate).toHaveBeenCalledWith(GOGGLE_HOME);
});

it('clears persisted search history through the core', async () => {
  session.history = ['Orion'];
  session.historyEnabled = true;
  site(`${GOGGLE_HOME}/account`);
  await screen.findByRole('button', { name: 'Orion' });
  fireEvent.click(screen.getByRole('button', { name: 'Limpar histórico' }));
  await screen.findByText('Nenhuma pesquisa salva.');
  expect(perform).toHaveBeenCalledWith(
    'goggle_preferences',
    { historyEnabled: true, clearHistory: true },
    expect.anything(),
  );
});

it('shows preserved query, results, zero-results and engine failures', async () => {
  const { rerender } = site(`${GOGGLE_HOME}/search?q=Orion%20%26%20A%C3%A7%C3%B5es`);
  expect(input()).toHaveValue('Orion & Ações');
  fireEvent.click(await screen.findByRole('button', { name: doc.title }));
  expect(navigate).toHaveBeenCalledWith(doc.url);
  response.total = 0;
  response.documents = [];
  rerender(
    <GoggleSite
      key="empty"
      address={`${GOGGLE_HOME}/search?q=missing`}
      page={page}
      navigate={navigate}
    />,
  );
  await screen.findByRole('heading', { name: 'Nenhum resultado encontrado' });
  vi.mocked(request).mockImplementation((command) =>
    command === 'goggle_session'
      ? Promise.resolve(session)
      : Promise.reject(new Error('Sem conexão')),
  );
  rerender(
    <GoggleSite
      key="error"
      address={`${GOGGLE_HOME}/search?q=error`}
      page={page}
      navigate={navigate}
    />,
  );
  expect(await screen.findByRole('alert')).toHaveTextContent('Sem conexão');
});

it('renders only bundled image previews and supports reverse lookup', async () => {
  response.documents = [{ ...doc, id: 'orion-image', type: 'IMAGE', imageId: 'orion-campus' }];
  response.images = [
    {
      id: 'orion-campus',
      asset: 'orion-campus',
      virtualUrl: 'https://www.orion.com/media/campus.svg',
      documentIds: ['orion'],
      entities: ['company:orion'],
    },
  ];
  site(`${GOGGLE_HOME}/images?q=Orion`);
  expect(await screen.findByRole('img', { name: doc.title })).toHaveAttribute(
    'src',
    '/assets/goggle/orion-campus.svg',
  );
  fireEvent.click(screen.getByRole('button', { name: `Buscar origem: ${doc.title}` }));
  expect(navigate).toHaveBeenCalledWith(
    expect.stringContaining('image=https%3A%2F%2Fwww.orion.com'),
  );
});

it('integrates all domain aliases with browser navigation and preserves queries outside addressKey', async () => {
  expect(goggleRoute('goggle.com')).not.toBeNull();
  expect(goggleRoute('www.goggle.com')).not.toBeNull();
  expect(goggleRoute('https://goggle.com.evil.test')).toBeNull();
  expect(goggleRoute('https://goggle.com@evil.test')).toBeNull();
  render(<Browser initialAddress="goggle.com" />);
  await screen.findByRole('combobox', { name: 'Pesquisar no Goggle' });
  fireEvent.change(input(), { target: { value: 'Orion Ações' } });
  fireEvent.submit(input().closest('form')!);
  await screen.findByRole('button', { name: doc.title });
  expect(input()).toHaveValue('Orion Ações');
  expect(screen.getByRole('textbox', { name: 'Endereço' })).toHaveValue(
    goggleProvider.searchUrl('Orion Ações'),
  );
  expect(perform).toHaveBeenCalledWith(
    'browser_navigate',
    { address: goggleProvider.searchUrl('Orion Ações') },
    expect.anything(),
  );
});
