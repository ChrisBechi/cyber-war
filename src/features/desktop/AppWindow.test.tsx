import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useWindows } from '../../lib/window-store';
import { AppWindow } from './AppWindow';

const nativeAnimate = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'animate');

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
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    if (nativeAnimate) {
      Object.defineProperty(HTMLElement.prototype, 'animate', nativeAnimate);
    } else {
      Reflect.deleteProperty(HTMLElement.prototype, 'animate');
    }
  });
  const mockLayout = () => {
    for (const [property, style, maximized] of [
      ['offsetLeft', 'left', 0],
      ['offsetTop', 'top', 0],
      ['offsetWidth', 'width', 1280],
      ['offsetHeight', 'height', 684],
    ] as const) {
      vi.spyOn(HTMLElement.prototype, property, 'get').mockImplementation(function (
        this: HTMLElement,
      ) {
        return this.style[style] === '100%' ? maximized : parseFloat(this.style[style]) || 0;
      });
    }
    const animations: Array<{
      cancel: ReturnType<typeof vi.fn>;
      effect: { getComputedTiming: () => { progress: number } };
      playState: string;
      onfinish?: () => void;
    }> = [];
    const animate = vi.fn(() => {
      const animation = {
        cancel: vi.fn(),
        effect: { getComputedTiming: () => ({ progress: 0.4 }) },
        playState: 'running',
      };
      animations.push(animation);
      return animation;
    });
    Object.defineProperty(HTMLElement.prototype, 'animate', { value: animate, configurable: true });
    return { animate, animations };
  };

  it('reverses an active maximize animation without remounting content and releases its animation layer', () => {
    const { animations, animate } = mockLayout();
    render(<WindowFixture />);
    const draft = screen.getByRole('textbox');
    fireEvent.change(draft, { target: { value: 'rascunho preservado' } });
    fireEvent.click(screen.getByRole('button', { name: 'Maximizar Arquivos' }));
    expect(animate).toHaveBeenCalledOnce();
    fireEvent.click(screen.getByRole('button', { name: 'Restaurar Arquivos' }));
    expect(animations[0].cancel).toHaveBeenCalledOnce();
    expect(animate).toHaveBeenCalledTimes(2);
    expect(screen.getByRole('textbox')).toBe(draft);
    expect(draft).toHaveValue('rascunho preservado');
    act(() => animations[1].onfinish?.());
    expect(screen.getByRole('dialog')).not.toHaveAttribute('data-window-sizing');
  });

  it('respects reduced motion without changing the final maximized geometry', () => {
    const { animate } = mockLayout();
    vi.stubGlobal('matchMedia', () => ({ matches: true }));
    render(<WindowFixture />);
    fireEvent.click(screen.getByRole('button', { name: 'Maximizar Arquivos' }));
    expect(animate).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toHaveStyle({ width: '100%', height: '100%' });
  });
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
