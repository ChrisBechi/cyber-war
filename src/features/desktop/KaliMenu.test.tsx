import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { softwareCatalog } from '../../lib/software-catalog';
import { KaliMenu } from './KaliMenu';

function setup(settings: Record<string, string> = {}) {
  const callbacks = {
    onLaunch: vi.fn(),
    onFavorite: vi.fn(),
    onBuiltin: vi.fn(),
    onClose: vi.fn(),
    onLock: vi.fn(),
    onLogout: vi.fn(),
  };
  const view = render(<KaliMenu settings={settings} nickname="kali" {...callbacks} />);
  return { ...callbacks, ...view };
}

describe('Kali launcher', () => {
  it('opens with the exact favorite order and searches packages and commands', () => {
    const { onLaunch } = setup();
    const search = screen.getByRole('textbox', { name: 'Pesquisar aplicativos' });
    expect(search).toHaveFocus();
    const rows = document.querySelectorAll('[data-menu-result]');
    expect([...rows].map((row) => row.textContent)).toEqual(
      softwareCatalog.favorites.map(
        (id) => softwareCatalog.entries.find((entry) => entry.id === id)?.name,
      ),
    );
    fireEvent.change(search, { target: { value: 'nmap' } });
    expect(screen.getByRole('button', { name: 'nmap' })).toBeInTheDocument();
    fireEvent.keyDown(search, { key: 'Enter' });
    expect(onLaunch).toHaveBeenCalledWith(expect.objectContaining({ id: 'kali-nmap' }));
    fireEvent.change(search, { target: { value: 'ghidra' } });
    expect(screen.getByText('Nenhum aplicativo encontrado.')).toBeInTheDocument();
  });
  it('navigates categories, subcategories and results with the keyboard', () => {
    setup();
    fireEvent.click(screen.getByRole('tab', { name: '01 - Reconnaissance' }));
    fireEvent.click(screen.getByRole('button', { name: 'Network Information ›' }));
    expect(screen.getByRole('button', { name: 'nmap' })).toBeInTheDocument();
    const nmap = screen.getByRole('button', { name: 'nmap' });
    nmap.focus();
    fireEvent.keyDown(nmap, { key: 'ArrowLeft' });
    expect(screen.getByRole('tabpanel', { name: '01 - Reconnaissance' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab', { name: 'Favorites' }));
    const search = screen.getByRole('textbox');
    fireEvent.keyDown(search, { key: 'ArrowDown' });
    expect(screen.getByRole('button', { name: 'Terminal Emulator' })).toHaveFocus();
    fireEvent.keyDown(document.activeElement ?? search, { key: 'ArrowDown' });
    expect(screen.getByRole('button', { name: 'Root Terminal Emulator' })).toHaveFocus();
  });
  it('uses saved favorites and recents, toggles favorites, and dismisses', () => {
    const { onFavorite, onClose } = setup({
      launcherRecent: '["kali-nmap","terminal"]',
      launcherFavorites: '["kali-nmap"]',
    });
    expect(screen.getByRole('button', { name: 'nmap' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Remover nmap dos favoritos' }));
    expect(onFavorite).toHaveBeenCalledWith('kali-nmap');
    fireEvent.click(screen.getByRole('tab', { name: 'Recently Used' }));
    expect(
      within(screen.getByRole('tabpanel')).getByRole('button', {
        name: 'Terminal Emulator',
      }),
    ).toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole('textbox'), { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.pointerDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
