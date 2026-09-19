import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import pack from '../../../../content/web/web-core.json';
import { CommunityPanel } from './CommunityPanel';
import { webPageSchema } from './web-model';

afterEach(cleanup);
function page(brandId: string, path: string) {
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
    community: {
      following: [],
      followers: 1,
      feed: [],
      posts: [],
      messages: [],
      notifications: [],
      listing: null,
      nickname: 'lia',
    },
  });
}
it('keeps a failed publication draft and clears it after the server accepts it', async () => {
  const act = vi.fn().mockResolvedValueOnce(false).mockResolvedValueOnce(true);
  render(
    <CommunityPanel page={page('linkup', '/')} act={act} navigate={vi.fn()} pending={false} />,
  );
  const draft = screen.getByRole('textbox', { name: 'Nova publicação' });
  fireEvent.change(draft, { target: { value: 'Meu projeto' } });
  fireEvent.click(screen.getByRole('button', { name: 'Publicar na rede' }));
  await waitFor(() => expect(act).toHaveBeenCalledTimes(1));
  expect(draft).toHaveValue('Meu projeto');
  fireEvent.click(screen.getByRole('button', { name: 'Publicar na rede' }));
  await waitFor(() => expect(draft).toHaveValue(''));
});
it('navigates to the employer and sends a follow action for a profile', () => {
  const act = vi.fn().mockResolvedValue(true),
    navigate = vi.fn();
  render(
    <CommunityPanel
      page={page('linkup', '/pessoas/nara-campos')}
      act={act}
      navigate={navigate}
      pending={false}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Nexora' }));
  expect(navigate).toHaveBeenCalledWith('https://www.nexora.com/');
  fireEvent.click(screen.getByRole('button', { name: 'Seguir' }));
  expect(act).toHaveBeenCalledWith('follow');
});
it('converts an offer to cents and only exposes valid listing transitions', async () => {
  const initial = page('feiralivre', '/anuncios/radio-aurora');
  initial.community!.listing = { status: 'AVAILABLE', offerCents: null };
  const act = vi.fn().mockResolvedValue(true);
  const rendered = render(
    <CommunityPanel page={initial} act={act} navigate={vi.fn()} pending={false} />,
  );
  fireEvent.change(screen.getByRole('spinbutton', { name: 'Sua proposta (R$)' }), {
    target: { value: '80' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Enviar proposta' }));
  await waitFor(() => expect(act).toHaveBeenCalledWith('offer', '8000', undefined));
  expect(screen.queryByRole('button', { name: 'Confirmar retirada' })).toBeNull();
  initial.community!.listing.status = 'RESERVED';
  rendered.rerender(<CommunityPanel page={initial} act={act} navigate={vi.fn()} pending={false} />);
  expect(screen.queryByRole('button', { name: 'Enviar proposta' })).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Confirmar retirada' }));
  expect(act).toHaveBeenCalledWith('sold');
});
