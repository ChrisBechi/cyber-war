import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { useWindows } from '../../lib/window-store';
import { AppWindow } from './AppWindow';

function WindowFixture() {
  const { windows, focus } = useWindows();
  return (
    <>
      <div className="window-layer">
        {windows.map((model) => (
          <AppWindow key={model.id} model={model}>
            <input aria-label="Rascunho" defaultValue="" />
          </AppWindow>
        ))}
      </div>
      <button data-window-task="files" onClick={() => focus('files')}>
        Reabrir Arquivos
      </button>
    </>
  );
}

describe('window transitions', () => {
  it('exits cinematic terminal fullscreen with Escape while preserving its content', () => {
    useWindows.getState().reset();
    useWindows.getState().open('terminal');
    render(<WindowFixture />);
    const draft = screen.getByRole('textbox');
    fireEvent.change(draft, { target: { value: 'comando parcial' } });
    act(() => useWindows.getState().update('terminal', { fullscreen: true, maximized: true }));
    expect(screen.getByRole('dialog')).toHaveClass('is-fullscreen');
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.getByRole('dialog')).not.toHaveClass('is-fullscreen');
    expect(screen.getByRole('textbox')).toHaveValue('comando parcial');
  });
  beforeEach(() => {
    useWindows.getState().reset();
    useWindows.getState().open('files');
  });
  it('switches to the restore control and restores the previous window geometry', () => {
    render(<WindowFixture />);
    const original = useWindows.getState().windows[0];
    fireEvent.click(screen.getByRole('button', { name: 'Maximizar Arquivos' }));
    expect(screen.getByRole('dialog')).toHaveStyle({
      width: '100%',
      height: '100%',
      left: '0px',
      top: '0px',
    });
    const restore = screen.getByRole('button', { name: 'Restaurar Arquivos' });
    // A double click on a control must not trigger the titlebar's maximize gesture.
    fireEvent.doubleClick(restore);
    expect(useWindows.getState().windows[0].maximized).toBe(true);
    fireEvent.click(restore);
    expect(screen.getByRole('dialog')).toHaveStyle({
      width: `${original.width}px`,
      height: `${original.height}px`,
      left: `${original.x}px`,
      top: `${original.y}px`,
    });
    expect(screen.getByRole('button', { name: 'Maximizar Arquivos' })).toBeInTheDocument();
  });
  it('keeps the content mounted during minimization and rapid restoration', () => {
    render(<WindowFixture />);
    const draft = screen.getByRole('textbox', { name: 'Rascunho' });
    fireEvent.change(draft, { target: { value: 'texto não salvo' } });
    fireEvent.click(screen.getByRole('button', { name: 'Maximizar Arquivos' }));
    fireEvent.click(screen.getByRole('button', { name: 'Minimizar Arquivos' }));
    const hidden = screen.getByRole('dialog', { hidden: true });
    expect(hidden).toHaveClass('is-minimized');
    expect(hidden).toHaveAttribute('inert');
    expect(hidden.style.display).toBe('flex');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Reabrir Arquivos' }));
    expect(screen.getByRole('dialog')).not.toHaveAttribute('inert');
    expect(screen.getByRole('textbox', { name: 'Rascunho' })).toBe(draft);
    expect(draft).toHaveValue('texto não salvo');
    expect(screen.getByRole('button', { name: 'Restaurar Arquivos' })).toBeInTheDocument();
  });
});
