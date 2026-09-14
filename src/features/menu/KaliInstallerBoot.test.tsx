import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { KaliInstallerBoot } from './KaliInstallerBoot';

describe('KaliInstallerBoot', () => {
  it('moves the single selection with keyboard and keeps focus on the selected entry', () => {
    const onContinue = vi.fn();
    render(<KaliInstallerBoot onContinue={onContinue} onCancel={vi.fn()} />);
    const graphical = screen.getByRole('menuitem', { name: 'Graphical install' });
    expect(graphical).toHaveFocus();
    fireEvent.keyDown(graphical, { key: 'ArrowDown' });
    const install = screen.getByRole('menuitem', { name: 'Install' });
    expect(install).toHaveFocus();
    expect(install).toHaveClass('is-selected');
    expect(graphical).not.toHaveClass('is-selected');
    fireEvent.keyDown(install, { key: 'Home' });
    expect(graphical).toHaveFocus();
    fireEvent.click(graphical);
    expect(onContinue).toHaveBeenCalledOnce();
  });

  it('returns to the game menu with Escape without starting installation', () => {
    const onContinue = vi.fn();
    const onCancel = vi.fn();
    render(<KaliInstallerBoot onContinue={onContinue} onCancel={onCancel} />);
    fireEvent.keyDown(screen.getByRole('menuitem', { name: 'Graphical install' }), {
      key: 'Escape',
    });
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onContinue).not.toHaveBeenCalled();
  });
});
