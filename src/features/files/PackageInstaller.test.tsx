import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import { PackageInstaller } from './PackageInstaller';
import { confirmPackage, inspectPackage, planPackage } from '../../lib/packages';
import { cancelArchiveJob, waitArchiveJob } from '../../lib/archive';
import type * as Archive from '../../lib/archive';

vi.mock('../../lib/packages', () => ({
  inspectPackage: vi.fn(),
  planPackage: vi.fn(),
  confirmPackage: vi.fn(),
}));
vi.mock('../../lib/archive', async (original) => ({
  ...(await original<typeof Archive>()),
  cancelArchiveJob: vi.fn(),
  waitArchiveJob: vi.fn(),
}));
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(inspectPackage).mockResolvedValue({
    name: 'netscan',
    version: '2.4.1',
    architecture: 'amd64',
    description: 'Network scanner',
    installedSize: 5000,
    downloadSize: 2000,
    dependencies: ['libpacket2 >= 2.0'],
    origin: 'Virtual repository',
    status: 'not-installed',
    valid: true,
  });
  vi.mocked(planPackage).mockResolvedValue({
    id: 'plan-1',
    summary: 'Unpack netscan\nKeep configurations',
  });
  vi.mocked(confirmPackage).mockResolvedValue({
    stdout: 'Setting up netscan',
    stderr: '',
    exitCode: 0,
    job: null,
  });
  vi.mocked(cancelArchiveJob).mockResolvedValue(null);
});
it('inspects first, reviews the native plan and only installs after confirmation', async () => {
  render(<PackageInstaller initialPath="/home/kali/Downloads/netscan.deb" />);
  expect(await screen.findByText('libpacket2 >= 2.0')).toBeVisible();
  expect(screen.getByRole('button', { name: 'Remover' })).toBeDisabled();
  expect(planPackage).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Instalar' }));
  expect(await screen.findByLabelText('Confirmar alteração de pacotes')).toHaveTextContent(
    'Unpack netscan',
  );
  expect(confirmPackage).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Confirmar' }));
  expect(await screen.findByLabelText('Resultado da operação')).toHaveTextContent(
    'Setting up netscan',
  );
  expect(confirmPackage).toHaveBeenCalledWith('plan-1', true);
});
it('releases an unconfirmed plan when closed', async () => {
  const view = render(<PackageInstaller initialPath="/tmp/test.deb" />);
  fireEvent.click(await screen.findByRole('button', { name: 'Instalar' }));
  await screen.findByLabelText('Confirmar alteração de pacotes');
  view.unmount();
  expect(confirmPackage).toHaveBeenCalledWith('plan-1', false);
});
it('shows a running job and allows cancellation through the shared job service', async () => {
  vi.mocked(confirmPackage).mockResolvedValue({ stdout: '', stderr: '', exitCode: 0, job: 4800 });
  vi.mocked(waitArchiveJob).mockImplementation((_id, progress) => {
    progress?.({
      id: 4800,
      pid: 4800,
      name: 'package install',
      progress: 20,
      status: 'Running',
      stdout: '',
      stderr: '',
      exitCode: 0,
      response: null,
    });
    return new Promise(() => undefined);
  });
  render(<PackageInstaller initialPath="/tmp/test.deb" />);
  fireEvent.click(await screen.findByRole('button', { name: 'Instalar' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Confirmar' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Cancelar operação' }));
  await waitFor(() => expect(cancelArchiveJob).toHaveBeenCalledWith(4800));
});
it('reports invalid packages without offering installation', async () => {
  vi.mocked(inspectPackage).mockRejectedValue(new Error('dpkg: corrupted payload'));
  render(<PackageInstaller initialPath="/tmp/broken.deb" />);
  expect(await screen.findByRole('alert')).toHaveTextContent('corrupted payload');
  expect(screen.queryByRole('button', { name: 'Instalar' })).not.toBeInTheDocument();
});
