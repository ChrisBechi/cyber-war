import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import {
  archiveOperation,
  startArchiveOperation,
  cancelArchiveJob,
  type ArchiveInspection,
  type ArchiveJob,
} from '../../lib/archive';
import type * as Archive from '../../lib/archive';
import { ArchiveViewer } from './ArchiveViewer';
import { CompressDialog, ExtractDialog } from './ArchiveDialogs';

vi.mock('../../lib/archive', async (original) => ({
  ...(await original<typeof Archive>()),
  archiveOperation: vi.fn(),
  startArchiveOperation: vi.fn(),
  cancelArchiveJob: vi.fn(),
}));
const info: ArchiveInspection = {
  path: '/home/kali/Downloads/backup.zip',
  format: 'ZIP',
  integrity: 'VALID',
  encrypted: false,
  compressedSize: 250,
  storageSize: 250,
  originalSize: 600,
  modifiedAt: 12,
  entries: [
    {
      path: 'folder/notes.txt',
      type: 'file',
      originalSize: 600,
      compressedSize: 200,
      permissions: 420,
      modifiedAt: 10,
      owner: 'kali',
      group: 'kali',
      crc: 0,
      linkTarget: null,
      encrypted: false,
    },
  ],
};
beforeEach(() => {
  vi.mocked(archiveOperation)
    .mockReset()
    .mockResolvedValue({ inspection: info, entries: [], error: null });
  vi.mocked(startArchiveOperation)
    .mockReset()
    .mockResolvedValue({ inspection: null, entries: [], error: null });
  vi.mocked(cancelArchiveJob).mockReset().mockResolvedValue(null);
});
it('opens only the index, navigates inferred folders and extracts the selected entry', async () => {
  render(<ArchiveViewer initialPath={info.path} />);
  fireEvent.click(await screen.findByRole('button', { name: '▸ folder' }));
  expect(archiveOperation).toHaveBeenCalledTimes(1);
  expect(startArchiveOperation).not.toHaveBeenCalled();
  fireEvent.click(screen.getByLabelText('Selecionar folder/notes.txt'));
  fireEvent.click(screen.getByRole('button', { name: 'Extrair seleção…' }));
  fireEvent.change(screen.getByLabelText('Destino'), { target: { value: '/tmp/output' } });
  fireEvent.click(screen.getByRole('button', { name: 'Extrair' }));
  await waitFor(() =>
    expect(startArchiveOperation).toHaveBeenCalledWith(
      expect.objectContaining({
        operation: 'extract',
        path: info.path,
        options: { destination: '/tmp/output', overwrite: 'ask', selected: ['folder/notes.txt'] },
      }),
      expect.any(Function),
    ),
  );
});
it('keeps an encrypted extraction open after a wrong password and clears the secret', async () => {
  vi.mocked(startArchiveOperation).mockRejectedValue(new Error('incorrect password'));
  render(
    <ExtractDialog
      archive={{ ...info, encrypted: true }}
      destination="/tmp/private"
      asRoot={false}
      close={vi.fn()}
    />,
  );
  const password = screen.getByLabelText('Senha');
  expect(password).toHaveAttribute('type', 'password');
  fireEvent.change(password, { target: { value: 'wrong-secret' } });
  fireEvent.click(screen.getByRole('button', { name: 'Extrair' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('incorrect password');
  expect(password).toHaveValue('');
  expect(screen.getByRole('dialog')).toBeVisible();
});
it('offers the five archive formats and cancels a running native job', async () => {
  const job = { id: 4800, progress: 45, status: 'Running' } as ArchiveJob;
  vi.mocked(startArchiveOperation).mockImplementation((_request, progress) => {
    progress?.(job);
    return new Promise(() => undefined);
  });
  render(
    <CompressDialog
      directory="/home/kali"
      paths={['/home/kali/qa']}
      asRoot={false}
      close={vi.fn()}
    />,
  );
  expect(screen.getAllByRole('option')).toHaveLength(5);
  fireEvent.click(screen.getByRole('button', { name: 'Criar' }));
  expect(await screen.findByRole('status')).toHaveTextContent('45%');
  fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
  expect(cancelArchiveJob).toHaveBeenCalledWith(4800);
});
it('surfaces invalid headers and uses the password dialog for integrity checks', async () => {
  vi.mocked(archiveOperation).mockResolvedValue({
    inspection: { ...info, encrypted: true },
    entries: [],
    error: null,
  });
  render(<ArchiveViewer initialPath={info.path} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Testar integridade' }));
  fireEvent.change(screen.getByLabelText('Senha'), { target: { value: 'qa-password' } });
  fireEvent.click(screen.getByRole('button', { name: 'Testar' }));
  await waitFor(() =>
    expect(archiveOperation).toHaveBeenLastCalledWith({
      operation: 'test',
      path: info.path,
      password: 'qa-password',
      asRoot: false,
    }),
  );
  expect(await screen.findByRole('status')).toHaveTextContent('Nenhum erro');
});
