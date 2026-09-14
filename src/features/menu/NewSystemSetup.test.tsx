import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { NewSystemSetup } from './NewSystemSetup';

const next = () => fireEvent.click(screen.getByRole('button', { name: 'Continuar' }));
const tick = () => act(() => vi.advanceTimersByTimeAsync(4000));
const field = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
const option = (name: RegExp) => fireEvent.click(screen.getByRole('option', { name }));
const stage = (name: string) =>
  expect(screen.getByRole('region', { name: 'Instalador do Kali Linux' })).toHaveAttribute(
    'data-stage',
    name,
  );

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());
async function toNetwork() {
  render(
    <NewSystemSetup
      initialHostname="kali"
      initialUsername="kali"
      onCancel={vi.fn()}
      onComplete={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole('menuitem', { name: 'Graphical install' }));
  next();
  next();
  next();
  await tick();
  await tick();
  stage('network');
}
async function toPartitioning() {
  await toNetwork();
  next();
  await tick();
  next();
  next();
  field('Nome completo do novo usuário:', 'Teste');
  next();
  next();
  field('Escolha uma senha para o novo usuário:', 'teste1234');
  field('Digite a senha novamente para verificação:', 'teste1234');
  next();
  next();
  stage('method');
}
describe('installer branches', () => {
  it('shows neighboring networks, refuses unavailable access and cancels link detection cleanly', async () => {
    await toNetwork();
    option(/wlan0:/);
    next();
    stage('wifi');
    option(/Vizinho_5G/);
    next();
    expect(screen.getByRole('alert')).toHaveTextContent('protegida');
    option(/^Casa/);
    next();
    stage('link');
    await act(() => vi.advanceTimersByTimeAsync(1800));
    fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
    await tick();
    stage('network');
    next();
    stage('wifi');
    fireEvent.click(screen.getByLabelText('Não configurar a rede agora'));
    next();
    stage('hostname');
  });
  it('creates a manual USB table and returns from No without formatting', async () => {
    await toPartitioning();
    option(/^Manual$/);
    next();
    option(/SCSI8/);
    next();
    stage('overview');
    next();
    stage('edit');
    field('Tamanho da partição (GB):', '1');
    field('Usar como:', 'ESP');
    next();
    stage('overview');
    next();
    expect(screen.getByRole('alert')).toHaveTextContent('raiz');
    option(/ESPAÇO LIVRE/);
    next();
    field('Tamanho da partição (GB):', '20');
    field('Ponto de montagem:', '/');
    next();
    next();
    stage('confirm');
    expect(screen.getByLabelText('Não')).toBeChecked();
    expect(screen.getByText(/partição #2 de \/dev\/sda como ext4/)).toBeInTheDocument();
    next();
    stage('overview');
    expect(screen.queryByRole('progressbar')).not.toBeInTheDocument();
    option(/#2/);
    next();
    stage('edit');
    field('Tamanho da partição (GB):', '100');
    next();
    expect(screen.getByRole('alert')).toHaveTextContent('ultrapassa');
  });
  it('explains partition options and requires matching encryption phrases', async () => {
    await toPartitioning();
    fireEvent.click(screen.getByRole('button', { name: 'Ajuda' }));
    expect(screen.getByRole('dialog')).toHaveTextContent('Manual');
    expect(screen.getAllByRole('term')).toHaveLength(5);
    fireEvent.click(screen.getByRole('button', { name: 'Fechar' }));
    option(/LVM criptografado/);
    next();
    next();
    next();
    stage('encryption');
    field('Frase secreta de criptografia:', 'protegido123');
    field('Repita a frase secreta:', 'diferente');
    next();
    expect(screen.getByRole('alert')).toHaveTextContent('igual');
    field('Repita a frase secreta:', 'protegido123');
    next();
    await tick();
    stage('overview');
    expect(screen.getByRole('option', { name: /Volume criptografado/ })).toBeInTheDocument();
  });
});
