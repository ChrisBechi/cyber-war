import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { request } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import type * as Api from '../../lib/api';
import type * as GameStore from '../../lib/game-store';
import type * as AppSettings from '../../lib/app-settings';
import { MainMenu } from './MainMenu';

vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  desktopRuntime: true,
  request: vi.fn(),
}));
vi.mock('../../lib/game-store', async (original) => ({
  ...(await original<typeof GameStore>()),
  perform: vi.fn(() => Promise.resolve()),
}));
vi.mock('../../lib/audio-manager', () => ({
  audioManager: { music: vi.fn(() => 'music'), stop: vi.fn(), play: vi.fn(), setVolumes: vi.fn() },
}));
vi.mock('../../lib/app-settings', async (original) => ({
  ...(await original<typeof AppSettings>()),
  saveAppSettings: vi.fn(() => Promise.resolve()),
}));
const slots = () =>
  Array.from({ length: 5 }, (_, index) => ({
    slotIndex: index + 1,
    occupied: false,
    label: '',
    playtimeSeconds: 0,
    currentMission: null,
    updatedAt: null,
    session: null,
  }));
async function open(label: RegExp) {
  fireEvent.click(screen.getByRole('button', { name: label }));
  await act(() => vi.advanceTimersByTimeAsync(500));
}
beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  useGame.setState({ error: '', busy: false });
  vi.mocked(request).mockResolvedValue(slots());
});
afterEach(() => vi.useRealTimers());
describe('MainMenu', () => {
  it('keeps Continue disabled without saves and opens the five real slots', async () => {
    render(<MainMenu onPlay={vi.fn()} />);
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByRole('button', { name: 'CONTINUAR' })).toBeDisabled();
    await open(/INICIAR HISTÓRIA/);
    expect(screen.getAllByRole('button', { name: /Slot vazio/ })).toHaveLength(5);
    expect(screen.getByRole('button', { name: /INICIAR HISTÓRIA/ })).toBeEnabled();
    expect(request).toHaveBeenCalledWith('list_save_slots', {}, expect.anything());
  });
  it('loads an occupied slot and preserves fixed order and campaign metadata', async () => {
    const data = slots();
    const saved = {
      ...data[2],
      occupied: true,
      label: 'kali',
      playtimeSeconds: 7260,
      currentMission: 'curiosity_01',
      session: 2,
      updatedAt: '2026-09-13T12:00:00Z',
    };
    vi.mocked(request).mockResolvedValue([saved, data[4], data[0], data[3], data[1]]);
    const onPlay = vi.fn();
    render(<MainMenu onPlay={onPlay} />);
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByRole('button', { name: 'CONTINUAR' })).toBeEnabled();
    await open(/^CONTINUAR$/);
    const entries = within(screen.getByRole('group', { name: 'Slots da campanha' })).getAllByRole(
      'button',
      { name: /^0[1-5]/ },
    );
    expect(entries.map((entry) => entry.textContent?.slice(0, 2))).toEqual([
      '01',
      '02',
      '03',
      '04',
      '05',
    ]);
    expect(entries[2]).toHaveTextContent('2 h 1 min · Sessão 2 · curiosity_01');
    fireEvent.click(screen.getByRole('button', { name: 'CONTINUAR →' }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(perform).toHaveBeenCalledWith(
      'load_slot',
      { slotIndex: 3, manual: false },
      expect.anything(),
    );
    expect(onPlay).toHaveBeenCalledWith(false, true);
  });
  it('requires confirmation before replacing an occupied campaign and starts it once', async () => {
    vi.mocked(request).mockResolvedValue(
      slots().map((slot) => ({
        ...slot,
        occupied: slot.slotIndex === 1,
        label: 'Campanha anterior',
      })),
    );
    const onPlay = vi.fn();
    render(<MainMenu onPlay={onPlay} />);
    await act(async () => {
      await Promise.resolve();
    });
    await open(/INICIAR HISTÓRIA/);
    fireEvent.click(screen.getByRole('button', { name: 'INICIAR HISTÓRIA →' }));
    expect(perform).not.toHaveBeenCalled();
    expect(screen.getByRole('alertdialog')).toHaveTextContent('SUBSTITUIR A CAMPANHA DO SLOT 01?');
    expect(screen.getByRole('alertdialog')).toHaveTextContent(
      'O save atual deste slot, incluindo seu progresso e checkpoints, será sobrescrito.',
    );
    fireEvent.click(screen.getByRole('button', { name: 'CANCELAR' }));
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(perform).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'INICIAR HISTÓRIA →' }));
    fireEvent.click(screen.getByRole('button', { name: 'SUBSTITUIR E INICIAR' }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.getByTestId('system-setup')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Graphical install' }));
    const expectStage = (stage: string) =>
      expect(screen.getByRole('region', { name: 'Instalador do Kali Linux' })).toHaveAttribute(
        'data-stage',
        stage,
      );
    const next = () => fireEvent.click(screen.getByRole('button', { name: 'Continuar' }));
    const progress = async (stage: string) => {
      expectStage(stage);
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '0');
      await act(() => vi.advanceTimersByTimeAsync(3800));
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');
      await act(() => vi.advanceTimersByTimeAsync(200));
    };
    for (const stage of ['language', 'location', 'keyboard']) {
      expectStage(stage);
      next();
    }
    await progress('media');
    await progress('components');
    expectStage('network');
    next();
    await progress('link');
    expectStage('hostname');
    next();
    expectStage('domain');
    next();
    expectStage('fullname');
    fireEvent.change(screen.getByLabelText('Nome completo do novo usuário:'), {
      target: { value: 'Kali Player' },
    });
    next();
    expectStage('username');
    next();
    expectStage('password');
    fireEvent.change(screen.getByLabelText('Escolha uma senha para o novo usuário:'), {
      target: { value: '1234' },
    });
    fireEvent.change(screen.getByLabelText('Digite a senha novamente para verificação:'), {
      target: { value: '1234' },
    });
    next();
    for (const stage of ['timezone', 'method', 'disk', 'scheme']) {
      expectStage(stage);
      next();
    }
    await progress('partitioning');
    expectStage('overview');
    next();
    expectStage('confirm');
    fireEvent.click(screen.getByLabelText('Sim'));
    next();
    await progress('format');
    await progress('base');
    expectStage('software');
    next();
    await progress('packages');
    expectStage('complete');
    next();
    await progress('finishing');
    const bootMenu = screen.getByRole('menu', { name: 'Menu de inicialização' });
    expect(
      within(bootMenu)
        .getAllByRole('menuitem')
        .map((item) => item.textContent),
    ).toEqual(['Kali GNU/Linux', 'Advanced options for Kali GNU/Linux', 'UEFI Firmware Settings']);
    expect(perform).not.toHaveBeenCalled();
    const boot = within(bootMenu).getByRole('menuitem', { name: 'Kali GNU/Linux' });
    expect(boot).toHaveFocus();
    fireEvent.keyDown(boot, { key: 'ArrowDown' });
    const advanced = within(bootMenu).getByRole('menuitem', {
      name: 'Advanced options for Kali GNU/Linux',
    });
    expect(advanced).toHaveFocus();
    expect(advanced).toHaveClass('is-selected');
    expect(boot).not.toHaveClass('is-selected');
    fireEvent.keyDown(advanced, { key: 'Home' });
    expect(boot).toHaveFocus();
    fireEvent.click(boot);
    await act(async () => {
      for (let index = 0; index < 40; index += 1) {
        await Promise.resolve();
      }
    });
    expect(perform).toHaveBeenCalledWith(
      'new_game',
      { slotIndex: 1, nickname: 'kali', hostname: 'lifeos', overwrite: true },
      expect.anything(),
    );
    expect(onPlay).toHaveBeenCalledExactlyOnceWith(true);
  });
  it('deletes only the chosen save after confirmation and keeps the other slots', async () => {
    const data = slots().map((slot) => ({
      ...slot,
      occupied: [1, 3].includes(slot.slotIndex),
      label: [1, 3].includes(slot.slotIndex) ? `Campanha ${slot.slotIndex}` : '',
    }));
    vi.mocked(request).mockResolvedValue(data);
    render(<MainMenu onPlay={vi.fn()} />);
    await act(async () => {
      await Promise.resolve();
    });
    await open(/INICIAR HISTÓRIA/);
    expect(screen.getAllByRole('button', { name: /^Excluir save/ })).toHaveLength(2);
    fireEvent.click(screen.getByRole('button', { name: 'Excluir save do slot 03: Campanha 3' }));
    expect(screen.getByRole('alertdialog')).toHaveTextContent('EXCLUIR O SAVE DO SLOT 03?');
    expect(screen.getByRole('alertdialog')).toHaveTextContent('Campanha 3');
    expect(screen.getByRole('alertdialog')).toHaveTextContent('será excluído permanentemente');
    fireEvent.click(screen.getByRole('button', { name: 'CANCELAR' }));
    expect(request).not.toHaveBeenCalledWith(
      'delete_save_slot',
      expect.anything(),
      expect.anything(),
    );
    expect(screen.getAllByRole('button', { name: /^Excluir save/ })).toHaveLength(2);

    let finish!: (value: typeof data) => void;
    vi.mocked(request).mockReturnValueOnce(
      new Promise((resolve) => {
        finish = resolve;
      }),
    );
    fireEvent.click(screen.getByRole('button', { name: 'Excluir save do slot 03: Campanha 3' }));
    fireEvent.click(screen.getByRole('button', { name: 'EXCLUIR SAVE' }));
    expect(screen.getByRole('button', { name: 'AGUARDE…' })).toBeDisabled();
    expect(request).toHaveBeenCalledWith(
      'delete_save_slot',
      { slotIndex: 3, confirmed: true },
      expect.anything(),
    );
    await act(async () => {
      finish(data.map((slot) => (slot.slotIndex === 3 ? slots()[2] : slot)));
      await Promise.resolve();
    });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: /^03 Slot vazio/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(screen.getAllByRole('button', { name: /^Excluir save/ })).toHaveLength(1);
    expect(
      screen.getByRole('button', { name: 'Excluir save do slot 01: Campanha 1' }),
    ).toBeInTheDocument();
    expect(perform).not.toHaveBeenCalled();
  });

  it('keeps a failed deletion available for retry and disables Continue after deleting the last save', async () => {
    const data = slots();
    data[0] = { ...data[0], occupied: true, label: 'kali' };
    vi.mocked(request).mockResolvedValue(data);
    useGame.setState({ slot: 1 });
    render(<MainMenu onPlay={vi.fn()} />);
    await act(async () => {
      await Promise.resolve();
    });
    await open(/^CONTINUAR$/);
    fireEvent.click(screen.getByRole('button', { name: 'Excluir save do slot 01: kali' }));
    vi.mocked(request).mockRejectedValueOnce(new Error('Falha de gravação'));
    fireEvent.click(screen.getByRole('button', { name: 'EXCLUIR SAVE' }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(within(screen.getByRole('alertdialog')).getByRole('alert')).toHaveTextContent(
      'Falha de gravação',
    );
    expect(
      screen.getByRole('button', { name: 'Excluir save do slot 01: kali' }),
    ).toBeInTheDocument();
    vi.mocked(request).mockResolvedValueOnce(slots());
    fireEvent.click(screen.getByRole('button', { name: 'EXCLUIR SAVE' }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'CONTINUAR →' })).toBeDisabled();
    expect(screen.getAllByRole('button', { name: /Slot vazio/ })).toHaveLength(5);
    expect(useGame.getState().world).toBeNull();
    await open(/Voltar ao menu/);
    expect(screen.getByRole('button', { name: 'CONTINUAR' })).toBeDisabled();
  });

  it('only exits through the native command after confirmation', async () => {
    render(<MainMenu onPlay={vi.fn()} />);
    await act(async () => {
      await Promise.resolve();
    });
    fireEvent.click(screen.getByRole('button', { name: 'SAIR' }));
    expect(screen.getByRole('alertdialog')).toHaveTextContent('SAIR DO CYBER WAR?');
    expect(request).not.toHaveBeenCalledWith('quit_game', expect.anything(), expect.anything());
    fireEvent.click(screen.getByRole('button', { name: 'CANCELAR' }));
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'SAIR' }));
    fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button', { name: 'SAIR' }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(request).toHaveBeenCalledWith('quit_game', {}, expect.anything());
  });
});
