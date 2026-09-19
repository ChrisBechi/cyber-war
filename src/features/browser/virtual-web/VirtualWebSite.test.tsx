import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import pack from '../../../../content/web/web-core.json';
import { perform, useGame } from '../../../lib/game-store';
import * as api from '../../../lib/api';
import type * as Store from '../../../lib/game-store';
import { webPageSchema, type WebPage } from './web-model';
import { VirtualWebSite } from './VirtualWebSite';

vi.mock('../../../lib/game-store', async (original) => ({
  ...(await original<typeof Store>()),
  perform: vi.fn(),
}));
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});
function page(brandId: string, path: string): WebPage {
  const brand = pack.brands.find((b) => b.id === brandId)!;
  return webPageSchema.parse({
    brand,
    canonicalUrl: `https://www.${brand.domain}${path}`,
    status: 200,
    document: pack.documents.find((d) => d.brandId === brandId && d.path === path),
    cards: [],
    related: [],
    categories: [],
    total: 0,
    offset: 0,
    query: '',
    category: '',
    liked: false,
    completed: false,
    cartCount: 0,
    cart: [],
    history: [],
  });
}

it('shows social reactions from the authoritative response and preserves the authored count', async () => {
  const initial = page('linkup', '/publicacoes/nara-campos-caderno');
  initial.community = {
    following: [],
    followers: 1,
    feed: [],
    posts: [],
    messages: [],
    notifications: [],
    listing: null,
    nickname: 'lia',
    connections: [],
    suggested: [],
  };
  const updated = { ...initial, liked: true };
  vi.mocked(perform).mockResolvedValueOnce({
    title: 'LinkUp',
    body: '',
    action: null,
    virtualWeb: updated,
  });
  render(<VirtualWebSite initialPage={initial} navigate={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: /♡ Curtir/ }));
  expect(await screen.findByRole('button', { name: /♥ Curtido/ })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
  expect(perform).toHaveBeenCalledWith(
    'web_interact',
    expect.objectContaining({ id: initial.document!.id, action: 'like' }),
    expect.anything(),
  );
});
it('routes between real platform pages through local navigation and renders controlled text safely', () => {
  const navigate = vi.fn();
  const product = page('shopnow', '/produto/pato-de-borracha');
  product.document!.blocks.push({ kind: 'paragraph', text: '<script>alert("unsafe")</script>' });
  const { container } = render(<VirtualWebSite initialPage={product} navigate={navigate} />);
  expect(
    screen.getByRole('heading', { name: 'Pato de borracha clássico', level: 1 }),
  ).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Casa do Dia' }));
  expect(navigate).toHaveBeenCalledWith('https://www.shopnow.com/vendedor/casa-do-dia');
  fireEvent.click(screen.getByRole('button', { name: 'Entenda o assunto na Wipédia' }));
  expect(navigate).toHaveBeenCalledWith('https://www.wipedia.org/wiki/rubber-duck');
  expect(container.querySelector('script,iframe,a[href],img[src^="http"]')).toBeNull();
  expect(screen.getByText('<script>alert("unsafe")</script>')).toBeInTheDocument();
});
it('updates the cart from the authoritative response and can remove a saved item', async () => {
  const initial = page('shopnow', '/produto/pato-de-borracha');
  const updated = structuredClone(initial);
  updated.cartCount = 1;
  updated.cart = [
    {
      quantity: 1,
      card: {
        id: initial.document!.id,
        url: initial.canonicalUrl,
        title: initial.document!.title,
        summary: initial.document!.summary,
        category: 'Brinquedos',
        author: 'ShopNow',
        visual: 'duck',
        priceCents: 1890,
      },
    },
  ];
  vi.mocked(perform)
    .mockResolvedValueOnce({ title: 'ShopNow', body: '', action: null, virtualWeb: updated })
    .mockResolvedValueOnce({ title: 'ShopNow', body: '', action: null, virtualWeb: initial });
  render(<VirtualWebSite initialPage={initial} navigate={vi.fn()} />);
  fireEvent.click(screen.getByRole('button', { name: 'Adicionar ao carrinho' }));
  await screen.findByRole('button', { name: 'Carrinho 1' });
  fireEvent.click(screen.getByRole('button', { name: 'Carrinho 1' }));
  expect(screen.getByRole('heading', { name: 'Seu carrinho' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Remover' }));
  await screen.findByText('Seu carrinho está vazio.');
  expect(perform).toHaveBeenLastCalledWith(
    'web_interact',
    expect.objectContaining({ id: initial.document!.id, action: 'removeCart' }),
    expect.anything(),
  );
});
it('keeps comment drafts after an error and clears only after a successful save', async () => {
  const thread = page('redditor', '/t/rubber-duck');
  vi.mocked(perform)
    .mockRejectedValueOnce(new Error('Falha ao salvar'))
    .mockResolvedValueOnce({ title: 'Redditor', body: '', action: null, virtualWeb: thread });
  render(<VirtualWebSite initialPage={thread} navigate={vi.fn()} />);
  const input = screen.getByRole('textbox', { name: 'Participe da conversa' });
  fireEvent.change(input, { target: { value: 'Minha experiência com o patinho.' } });
  fireEvent.click(screen.getByRole('button', { name: 'Publicar comentário' }));
  await screen.findByRole('alert');
  expect(input).toHaveValue('Minha experiência com o patinho.');
  fireEvent.click(screen.getByRole('button', { name: 'Publicar comentário' }));
  await screen.findByText('Comentário publicado.');
  await waitFor(() => expect(input).toHaveValue(''));
});
it('supports ingredients and course navigation with saved lesson completion', async () => {
  const recipe = page('cozinha', '/receitas/bolo-de-cenoura');
  const rendered = render(<VirtualWebSite initialPage={recipe} navigate={vi.fn()} />);
  const ingredient = screen.getByRole('checkbox', { name: '250 g de cenoura descascada' });
  fireEvent.click(ingredient);
  expect(ingredient).toBeChecked();
  expect(screen.getByRole('heading', { name: 'Modo de preparo' })).toBeInTheDocument();
  rendered.unmount();
  const navigate = vi.fn();
  const course = render(
    <VirtualWebSite initialPage={page('educa', '/cursos/javascript')} navigate={navigate} />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Iara Mendonça' }));
  expect(navigate).toHaveBeenCalledWith('https://www.educamais.com/professores/iara-mendonca');
  fireEvent.click(screen.getByRole('button', { name: '1. Objetivo: javascript' }));
  expect(navigate).toHaveBeenCalledWith('https://www.educamais.com/cursos/javascript/aula-1');
  course.unmount();
  const lesson = page('educa', '/cursos/javascript/aula-1');
  vi.mocked(perform).mockResolvedValueOnce({
    title: 'Aula',
    body: '',
    action: null,
    virtualWeb: { ...lesson, completed: true },
  });
  render(<VirtualWebSite initialPage={lesson} navigate={navigate} />);
  fireEvent.click(screen.getByRole('button', { name: 'Concluir esta aula' }));
  expect(await screen.findByRole('button', { name: '✓ Aula concluída' })).toBeDisabled();
});
it('uses site search and preserves brand identity on missing routes', () => {
  const navigate = vi.fn();
  const missing = { ...page('nexora', '/sobre'), status: 404, document: null };
  const { container } = render(<VirtualWebSite initialPage={missing} navigate={navigate} />);
  expect(container.querySelector('.web-corporate')).not.toBeNull();
  expect(
    screen.getByRole('heading', { name: 'Esta página não está disponível.' }),
  ).toBeInTheDocument();
  const input = screen.getByRole('textbox', { name: 'Pesquisar em Nexora' });
  fireEvent.change(input, { target: { value: 'NexPhone X2' } });
  fireEvent.submit(input.closest('form')!);
  expect(navigate).toHaveBeenCalledWith(
    'https://www.nexora.com/search?q=NexPhone%20X2&category=&offset=0',
  );
});

it('refreshes a live offer, excludes unavailable cart items and keeps them removable', async () => {
  const initial = page('shopnow', '/produto/nexphone-x2');
  const changed = structuredClone(initial);
  if (changed.document!.detail.kind !== 'product') {
    throw new Error('expected product');
  }
  changed.document!.detail.stock = 'OUT_OF_STOCK';
  changed.document!.detail.priceCents = 189900;
  changed.document!.updatedAt = 1789259400;
  changed.offerHistory = [
    { timestamp: 1789258200, priceCents: 179900, stock: 'IN_STOCK' },
    { timestamp: 1789259400, priceCents: 189900, stock: 'OUT_OF_STOCK' },
  ];
  changed.cartCount = 1;
  changed.cart = [
    {
      quantity: 1,
      available: false,
      card: {
        id: initial.document!.id,
        url: initial.canonicalUrl,
        title: initial.document!.title,
        summary: '',
        category: 'Tecnologia',
        author: 'ShopNow',
        visual: 'phone',
        priceCents: 189900,
        stock: 'OUT_OF_STOCK',
      },
    },
  ];
  vi.spyOn(api, 'request').mockResolvedValueOnce(initial).mockResolvedValueOnce(changed);
  vi.mocked(perform).mockResolvedValueOnce({
    title: 'ShopNow',
    body: '',
    action: null,
    virtualWeb: { ...changed, cartCount: 0, cart: [] },
  });
  render(<VirtualWebSite initialPage={initial} navigate={vi.fn()} />);
  await waitFor(() => expect(api.request).toHaveBeenCalledTimes(1));
  act(() => useGame.setState((state) => ({ revision: state.revision + 1 })));
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Adicionar ao carrinho' })).toBeDisabled(),
  );
  expect(screen.getByText('Histórico de preço e estoque')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Carrinho 1' }));
  expect(screen.getByText('Indisponível · fora do total')).toBeInTheDocument();
  expect(screen.getByText(/Total disponível:.*0,00/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Remover' }));
  await screen.findByText('Seu carrinho está vazio.');
});
