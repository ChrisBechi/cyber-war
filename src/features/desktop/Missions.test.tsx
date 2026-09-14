import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, expect, it, vi } from 'vitest';
import { Missions } from './Missions';
import { act, useGame } from '../../lib/game-store';

vi.mock('../../lib/game-store', async (original) => ({
  ...(await original<Record<string, unknown>>()),
  act: vi.fn(),
}));
const mission = (id: string, status: 'active' | 'available' | 'completed') => ({
  id,
  status,
  title: id,
  contact: 'VEX',
  description: 'Missão de teste',
  objective: '',
  hint: '',
  choices: [],
});
beforeEach(() => {
  vi.clearAllMocks();
  useGame.setState({ busy: false });
});

it('keeps other missions visible but disables accepting them while one is active', () => {
  useGame.setState({ missions: [mission('A', 'active'), mission('B', 'available')] });
  render(<Missions />);
  const blocked = screen.getByRole('button', { name: 'Aguardando missão atual' });
  expect(blocked).toBeDisabled();
  fireEvent.click(blocked);
  expect(act).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Sair da tentativa e voltar ao computador' }));
  expect(act).toHaveBeenCalledWith('mission_abort_attempt', { missionId: 'A' });
});
it('allows accepting an available mission after the active one completes', () => {
  useGame.setState({ missions: [mission('A', 'completed'), mission('B', 'available')] });
  render(<Missions />);
  const accept = screen.getByRole('button', { name: 'Aceitar' });
  expect(accept).toBeEnabled();
  fireEvent.click(accept);
  expect(act).toHaveBeenCalledWith('mission_start', { missionId: 'B' });
});
