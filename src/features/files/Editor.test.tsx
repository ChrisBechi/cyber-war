import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { request } from '../../lib/api';
import type * as Api from '../../lib/api';
import { perform } from '../../lib/game-store';
import { Editor } from './Editor';

vi.mock('../../lib/api', async (original) => ({
  ...(await original<typeof Api>()),
  request: vi.fn(),
}));
vi.mock('../../lib/game-store', () => ({ perform: vi.fn() }));

describe('shared file editor', () => {
  beforeEach(() => {
    vi.mocked(request).mockReset();
    vi.mocked(perform).mockReset();
  });
  it('opens virtual content and supplies its original version when saving', async () => {
    vi.mocked(request).mockResolvedValue('created in terminal');
    vi.mocked(perform).mockResolvedValue(null);
    render(<Editor initialPath="/home/kali/Documents/notes.txt" />);
    const editor = screen.getByRole('textbox', { name: 'Conteúdo do arquivo' });
    await waitFor(() => expect(editor).toHaveValue('created in terminal'));
    fireEvent.change(editor, { target: { value: 'edited in GUI' } });
    fireEvent.click(screen.getByRole('button', { name: /Salvar/ }));
    await waitFor(() =>
      expect(perform).toHaveBeenCalledWith(
        'vfs_write',
        {
          path: '/home/kali/Documents/notes.txt',
          content: 'edited in GUI',
          expectedContent: 'created in terminal',
          asRoot: false,
        },
        expect.anything(),
      ),
    );
    await waitFor(() =>
      expect(screen.getByText('Salvo no computador virtual')).toBeInTheDocument(),
    );
  });
  it('keeps root access local to the explicitly elevated editor instance', async () => {
    vi.mocked(request).mockResolvedValue('private');
    vi.mocked(perform).mockResolvedValue(null);
    render(<Editor initialPath="/root/notes.txt" asRoot />);
    await waitFor(() =>
      expect(screen.getByRole('textbox', { name: 'Conteúdo do arquivo' })).toHaveValue('private'),
    );
    expect(request).toHaveBeenCalledWith(
      'vfs_read',
      { path: '/root/notes.txt', asRoot: true },
      expect.anything(),
    );
    fireEvent.click(screen.getByRole('button', { name: /Salvar/ }));
    await waitFor(() =>
      expect(perform).toHaveBeenCalledWith(
        'vfs_write',
        {
          path: '/root/notes.txt',
          content: 'private',
          expectedContent: 'private',
          asRoot: true,
        },
        expect.anything(),
      ),
    );
  });
  it('preserves the buffer when a concurrent terminal edit prevents saving', async () => {
    vi.mocked(request).mockResolvedValue('old');
    vi.mocked(perform).mockRejectedValue(new Error('conflict'));
    render(<Editor initialPath="/home/kali/Documents/conflict.txt" />);
    const editor = screen.getByRole('textbox', { name: 'Conteúdo do arquivo' });
    await waitFor(() => expect(editor).toHaveValue('old'));
    fireEvent.change(editor, { target: { value: 'my unsaved text' } });
    fireEvent.click(screen.getByRole('button', { name: /Salvar/ }));
    await waitFor(() => expect(screen.getByRole('button', { name: /Salvar/ })).toBeEnabled());
    expect(editor).toHaveValue('my unsaved text');
    expect(screen.getByText(/Alterações não salvas/)).toBeInTheDocument();
  });
});
