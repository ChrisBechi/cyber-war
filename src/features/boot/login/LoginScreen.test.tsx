import { act, cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { request } from '../../../lib/api';
import type * as Api from '../../../lib/api';
import { useGame } from '../../../lib/game-store';
import { LoginScreen } from './LoginScreen';

vi.mock('../../../lib/api', async (importOriginal) => ({
  ...(await importOriginal<typeof Api>()),
  desktopRuntime: true,
  request: vi.fn(),
}));

const world = {
  nickname: 'maria',
  hostname: 'orion',
  settings: { loginUsername: 'maria', loginPassword: 'segredo' },
};

describe('LoginScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useGame.setState({ world: world as never });
  });
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });
  it('accepts the configured account and rejects invalid credentials', () => {
    useGame.setState({ world: world as never });
    const onSuccess = vi.fn();
    render(<LoginScreen onCancel={vi.fn()} onSuccess={onSuccess} />);
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'errada' } });
    fireEvent.submit(screen.getByRole('button', { name: 'Entrar' }).closest('form')!);
    expect(screen.getByRole('alert')).toHaveTextContent('incorretos');
    expect(onSuccess).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'segredo' } });
    fireEvent.submit(screen.getByRole('button', { name: 'Entrar' }).closest('form')!);
    expect(onSuccess).toHaveBeenCalledOnce();
  });

  it.each([
    ['America/Sao_Paulo', '14 set, 00:00'],
    ['America/Manaus', '13 set, 23:00'],
    ['America/Belem', '14 set, 00:00'],
    ['invalid-zone', '14 set, 00:00'],
    [undefined, '14 set, 00:00'],
  ])('uses the saved timezone %s instead of the computer timezone', (timezone, label) => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-09-14T03:00:00Z'));
    useGame.setState({
      world: {
        ...world,
        settings: { ...world.settings, language: 'pt-BR', timezone, keyboard: 'us-intl' },
      } as never,
    });
    render(<LoginScreen onCancel={vi.fn()} onSuccess={vi.fn()} />);
    expect(screen.getByText(label)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Idioma: Português (Brasil)' }));
    expect(screen.getByRole('dialog', { name: 'Idioma e região' })).toHaveTextContent(
      'Inglês americano (internacional)',
    );
  });

  it('updates the clock across midnight without interacting and clears its timer', async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-09-14T02:59:59Z'));
    const { unmount } = render(<LoginScreen onCancel={vi.fn()} onSuccess={vi.fn()} />);
    expect(screen.getByText('13 set, 23:59')).toBeInTheDocument();
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(screen.getByText('14 set, 00:00')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Data e hora:/ }));
    expect(screen.getByRole('dialog', { name: 'Data e hora' })).toHaveTextContent(
      'segunda-feira, 14 de setembro de 2026',
    );
    unmount();
    await act(async () => {
      await vi.runOnlyPendingTimersAsync();
    });
    expect(vi.getTimerCount()).toBe(0);
  });

  it('lets the session menu focus login, clear credentials, and return to the menu', () => {
    const onCancel = vi.fn();
    const onSuccess = vi.fn();
    render(<LoginScreen onCancel={onCancel} onSuccess={onSuccess} />);
    const open = () => fireEvent.click(screen.getByRole('button', { name: 'Opções de sessão' }));
    open();
    fireEvent.click(screen.getByRole('button', { name: 'Entrar no Kali Linux' }));
    expect(screen.getByLabelText('Senha')).toHaveFocus();
    expect(onSuccess).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'segredo' } });
    open();
    fireEvent.click(screen.getByRole('button', { name: 'Trocar usuário' }));
    expect(screen.getByLabelText('Usuário')).toHaveValue('');
    expect(screen.getByLabelText('Senha')).toHaveValue('');
    expect(screen.getByLabelText('Usuário')).toHaveFocus();
    open();
    fireEvent.click(screen.getByRole('button', { name: 'Voltar ao menu do jogo' }));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it('applies accessibility settings and restores focus when a panel is dismissed', () => {
    render(<LoginScreen onCancel={vi.fn()} onSuccess={vi.fn()} />);
    const button = screen.getByRole('button', { name: 'Acessibilidade' });
    fireEvent.click(button);
    fireEvent.click(screen.getByRole('checkbox', { name: 'Texto maior' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Alto contraste' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Mostrar senha' }));
    expect(screen.getByRole('main')).toHaveClass('login-large-text', 'login-high-contrast');
    expect(screen.getByLabelText('Senha')).toHaveAttribute('type', 'text');
    fireEvent.click(screen.getByRole('checkbox', { name: 'Mostrar senha' }));
    expect(screen.getByLabelText('Senha')).toHaveAttribute('type', 'password');
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(button).toHaveFocus();
    expect(button).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(button);
    fireEvent.pointerDown(screen.getByRole('main'));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('restarts the virtual system, clears credentials and returns to login after 3 seconds', () => {
    vi.useFakeTimers();
    const onSuccess = vi.fn();
    render(<LoginScreen onCancel={vi.fn()} onSuccess={onSuccess} />);
    fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'segredo' } });
    fireEvent.click(screen.getByRole('button', { name: 'Energia' }));
    fireEvent.click(screen.getByRole('button', { name: 'Reiniciar sistema…' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirmar reinício' }));
    expect(screen.getByRole('status')).toHaveTextContent('Reiniciando');
    void act(() => {
      vi.advanceTimersByTime(2999);
    });
    expect(screen.queryByLabelText('Senha')).not.toBeInTheDocument();
    void act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(screen.getByLabelText('Senha')).toHaveValue('');
    expect(screen.getByLabelText('Usuário')).toHaveValue('maria');
    expect(onSuccess).not.toHaveBeenCalled();
  });

  it('only quits the game after confirming shutdown, and reports a native failure', async () => {
    vi.mocked(request).mockRejectedValueOnce(new Error('native failure'));
    render(<LoginScreen onCancel={vi.fn()} onSuccess={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: 'Energia' }));
    fireEvent.click(screen.getByRole('button', { name: 'Desligar…' }));
    fireEvent.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Cancelar' }));
    expect(request).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Desligar…' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirmar desligamento' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Não foi possível desligar');
    expect(request).toHaveBeenCalledWith('quit_game', {}, expect.anything());
    expect(screen.getByLabelText('Usuário')).toBeInTheDocument();
  });
});
