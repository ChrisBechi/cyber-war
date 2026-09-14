import { useState } from 'react';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ForumBoard } from './ForumBoard';
import { ForumBody } from './ForumElements';
import { boardThreads, emptyForumState, searchThreads } from './forum-model';
import type { ForumDispatch } from './forum-model';

const props = {
  nickname: 'tester',
  reputation: 12,
  flags: [],
  connected: true,
  state: emptyForumState(),
  onMission: vi.fn(),
};
const openLinux = () => {
  fireEvent.click(screen.getByRole('button', { name: 'Linux & sistemas' }));
  fireEvent.click(screen.getByRole('button', { name: /Primeiros passos: encontre seus arquivos/ }));
};

describe('Terminal Board', () => {
  it('navigates categories, a thread, member profile and search without revealing gated story posts', () => {
    render(<ForumBoard {...props} onAction={vi.fn().mockResolvedValue(null)} />);
    expect(screen.getByRole('heading', { name: 'Últimas mensagens' })).toBeInTheDocument();
    expect(screen.queryByText(/alguém consegue me ajudar com um servidor/)).not.toBeInTheDocument();
    openLinux();
    expect(screen.getByRole('heading', { name: /Primeiros passos:/ })).toBeInTheDocument();
    fireEvent.click(
      within(screen.getByRole('article', { name: 'Mensagem de PATCH' })).getByRole('button', {
        name: 'Perfil',
      }),
    );
    expect(screen.getByRole('heading', { name: 'Perfil de PATCH' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Encontrar mensagens' }));
    expect(screen.getByRole('heading', { name: 'Mensagens de PATCH' })).toBeInTheDocument();
  });

  it('creates a topic under the current account and shows it in the thread reader', async () => {
    const submitted = vi.fn();
    function Harness() {
      const [state, setState] = useState(emptyForumState);
      const dispatch: ForumDispatch = (action) => {
        submitted(action);
        if (action.kind === 'createThread') {
          setState((previous) => ({
            ...previous,
            threads: [
              ...previous.threads,
              {
                id: 'player-test',
                title: action.title,
                body: action.body,
                category: action.category,
                author: 'tester',
                createdAt: '2026-09-14T12:00:00Z',
                replies: [],
              },
            ],
          }));
          return Promise.resolve('player-test');
        }
        return Promise.resolve(null);
      };
      return <ForumBoard {...props} state={state} onAction={dispatch} />;
    }
    render(<Harness />);
    fireEvent.click(screen.getByRole('button', { name: 'Linux & sistemas' }));
    fireEvent.click(screen.getByRole('button', { name: 'Novo tópico' }));
    fireEvent.change(screen.getByLabelText('Título do tópico'), {
      target: { value: 'Minha configuração do terminal' },
    });
    fireEvent.change(screen.getByLabelText('Mensagem'), {
      target: { value: 'Compartilhando uma configuração de teste.' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Publicar tópico' }));
    await waitFor(() =>
      expect(
        screen.getByRole('heading', { name: 'Minha configuração do terminal' }),
      ).toBeInTheDocument(),
    );
    expect(submitted).toHaveBeenCalledWith({
      kind: 'createThread',
      category: 'linux',
      title: 'Minha configuração do terminal',
      body: 'Compartilhando uma configuração de teste.',
    });
    expect(screen.getByRole('article', { name: 'Mensagem de tester' })).toBeInTheDocument();
  });

  it('quotes the selected message and preserves the response draft when saving fails', async () => {
    const onAction = vi.fn<ForumDispatch>().mockImplementation((action) => {
      if (action.kind === 'reply') {
        return Promise.reject(new Error('Gravação indisponível'));
      }
      return Promise.resolve(null);
    });
    render(<ForumBoard {...props} onAction={onAction} />);
    openLinux();
    fireEvent.click(screen.getAllByRole('button', { name: 'Citar' })[0]);
    expect(screen.getByText('Citando PATCH')).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText('Sua resposta'), {
      target: { value: 'Minha resposta fica no editor.' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Publicar resposta' }));
    await waitFor(() =>
      expect(screen.getByRole('alert')).toHaveTextContent('Gravação indisponível'),
    );
    expect(screen.getByLabelText('Sua resposta')).toHaveValue('Minha resposta fica no editor.');
    expect(onAction).toHaveBeenCalledWith({
      kind: 'reply',
      threadId: 'linux-first-steps',
      body: 'Minha resposta fica no editor.',
      quoteId: 'linux-first-steps:op',
    });
  });

  it('keeps locked threads read-only and exposes an offline state', () => {
    const view = render(<ForumBoard {...props} onAction={vi.fn().mockResolvedValue(null)} />);
    fireEvent.click(screen.getAllByRole('button', { name: /Bem-vindo ao Terminal Board/ })[0]);
    expect(screen.queryByRole('button', { name: 'Publicar resposta' })).not.toBeInTheDocument();
    expect(screen.getByText('Este tópico está fechado para novas respostas.')).toBeInTheDocument();
    view.rerender(
      <ForumBoard {...props} connected={false} onAction={vi.fn().mockResolvedValue(null)} />,
    );
    expect(screen.getByRole('heading', { name: 'Sem conexão com o fórum' })).toBeInTheDocument();
    expect(screen.queryByRole('article')).not.toBeInTheDocument();
  });

  it('searches message bodies with accents and excludes unavailable mission threads', () => {
    const threads = boardThreads(emptyForumState(), []);
    expect(searchThreads(threads, 'copias', 'SYSOP')).toHaveLength(0);
    expect(searchThreads(threads, 'configuracao')).not.toHaveLength(0);
    expect(searchThreads(threads, 'Hack novo funcionando')).toHaveLength(0);
    expect(
      searchThreads(boardThreads(emptyForumState(), ['V1_COMPLETE']), 'Hack novo funcionando')[0]
        .id,
    ).toBe('release');
  });

  it('renders text and code as inert content without accepting HTML', () => {
    const { container } = render(
      <ForumBody text={'**Uma nota**\n\n<script>alert(1)</script>\n\n```sh\ncat notes.txt\n```'} />,
    );
    expect(container.querySelector('script')).toBeNull();
    expect(screen.getByText('<script>alert(1)</script>')).toBeInTheDocument();
    expect(container.querySelector('strong')).toHaveTextContent('Uma nota');
    expect(container.querySelector('pre')).toHaveTextContent('cat notes.txt');
  });
});
