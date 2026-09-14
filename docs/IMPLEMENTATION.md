# Execução das especificações

## Leitura e precedência

O diretório original continha documentação e um pacote starter em `specs/`, sem uma aplicação implementada. Foram inventariados 280 arquivos textuais de referência (79 conteúdos distintos por SHA-256). As cópias e os arquivos ZIP legados foram comparados para evitar tratar versões antigas como requisitos adicionais. O starter foi lido e extraído sem substituir a documentação original: 15 specs, 8 skills e os arquivos de configuração/código-base.

A referência principal é `Game_Hacker_Documentacao_Consolidada_v3_2`. Complementos canônicos posteriores têm precedência sobre roteiros antigos:

- `GAME_HACKER_PROGRESSAO_TECNICA_E_DESIGN_DE_MISSOES.md`: conhecimento prévio é válido; técnica conta pelo resultado; não há bloqueio artificial por nível.
- `GAME_HACKER_SESSAO_2_REPUTATION_CANONICA.md`: APROVADO é o fechamento da Sessão 2, substituindo a antiga invasão paga final.
- Os documentos recentes em `Sessão 3/` definem BREACH, SIGNAL e GHOSTMARKET, reservados para expansão posterior.

Gregory permanece um amigo humano, sem traição secreta. O tio se limita à recuperação de fotos e à curiosidade opcional sobre a amante; não foi ligado ao submundo. A Garota é leiga e sua evidência permanece no mundo. Os detalhes canônicos de NULL, Mikhail e das rotas finais foram preservados como referência, sem antecipar a Sessão 3 no recorte inicial.

## Cobertura por spec

| Spec             | Implementação e verificação                                                                                                                                                                                                                            |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 00 — Produto     | Aplicação desktop offline; computadores, arquivos e redes pertencem à simulação. Intro → menu → campanha → LifeOS.                                                                                                                                     |
| 01 — Arquitetura | React apresenta; Rust decide. IPC Tauri, `GameService`, VFS/rede/missões e SQLite. Sem API HTTP de gameplay.                                                                                                                                           |
| 02 — LifeOS      | Painéis, launcher, notificações, 12 aplicativos internos, foco, arraste, resize, minimizar/maximizar, quatro workspaces. Ícones derivam de `/home/kali/Desktop`.                                                                                       |
| 03 — Terminal    | Tokenizer próprio, quotes, redirecionamento simples, histórico, autocomplete e comandos T0–T3. `lab` e `nmap` usam somente entidades virtuais. Comando desconhecido falha.                                                                             |
| 04 — VFS         | Árvore persistente com metadados, caminhos relativos/absolutos, permissões/owner/group, criação/edição/cópia/movimentação/remoção, `sudo`/`su`. GUI e terminal compartilham estado. Edição antiga não sobrescreve alterações externas silenciosamente. |
| 05 — Rede        | Hosts, DNS, sub-rede/gateway, Wi-Fi, portas, serviços, credenciais, firewall, estado corrigido e sessão SSH. Consultas externas são rejeitadas; não se abre socket real.                                                                               |
| 06 — Missões     | JSON/Serde, requirements, triggers, stages, choices, outcomes, recompensas, efeitos e eventos. Missões paralelas, alternativas da V2 e ordem Wi-Fi/pendrive verificadas.                                                                               |
| 07 — Saves       | Cinco slots, manual/autosave, 20 checkpoints por slot, SHA-256 e restauração transacional. Corrupção, rollback de SQL e isolamento entre slots testados.                                                                                               |
| 08 — Sandbox     | Allowlist de builtins; nenhum input chega ao SO. Host acessado somente para o diretório fixo de dados do app. CSP bloqueia navegação/requisições externas da UI.                                                                                       |
| 09 — Windows     | Configuração e pipeline NSIS/MSI, ícone original, execução sem Node/Rust no computador do jogador. Assinatura comercial e ensaio em máquina limpa são passos de distribuição, descritos em QA.                                                         |
| 10 — Qualidade   | Gate `pnpm check`: Prettier, ESLint, Stylelint, TypeScript, validação Zod de conteúdo, Vitest, rustfmt, Clippy com warnings como erro e testes Rust.                                                                                                   |
| 11 — Roadmap     | Base M0–M4 implementada; fidelidade M5 inclui workspaces, processos, permissões, configurações e navegador. Monaco permanece futuro, como explicitado no starter; o editor atual é simples.                                                            |
| 12 — Contratos   | Comandos IPC para terminal, saves, checkpoints, VFS, missões, mensagens, configurações e navegação fictícia. Respostas validadas na borda frontend.                                                                                                    |
| 13 — Convenções  | TypeScript estrito, código em inglês, texto em português, erros tipados, sem `unwrap()` em runtime, componentes separados por aplicativo e migração considerada.                                                                                       |
| 14 — Conteúdo    | Diretórios de conteúdo separados, missões em JSON, schemas Zod estritos no tooling e Serde no core. IDs e referências de serviços verificados.                                                                                                         |

## Decisões de implementação

O `GameService` clona o mundo, executa a intenção, avalia os efeitos e persiste snapshot/checkpoints em uma transação antes de publicar o estado. Uma falha de disco/SQLite não produz uma recompensa ou edição apenas em memória. Saves não são enviados pelo frontend: a UI solicita ações e recebe o mundo autoritativo.

O terminal implementa um subconjunto explícito, não Bash completo. Operadores de shell não suportados retornam erro. Arquivos são texto com limite de 1 MiB e até 10.000 nós por VFS. Symlinks, streaming de blobs, múltiplas sessões SSH simultâneas e expansão completa da internet fictícia são extensões futuras.

O recorte condensa a Sessão 1 em dez definições de missão. Ele valida os sistemas e os principais resultados narrativos; não representa todo o roteiro de diálogos, assets audiovisuais, chamadas, minigames e todas as horas de campanha previstas na documentação. O final mantém o computador disponível e sinaliza a passagem para REPUTATION.

O scanner e os instrumentos `lab` são ferramentas de gameplay explícitas. A rede possui dois hosts sem tráfego real. Processos e usuários são modelos mínimos; não há VM, escalonador de SO ou execução de código arbitrário.

Os instaladores são builds locais de desenvolvimento. Nenhum certificado, conta de assinatura ou release público foi criado. Os workflows estão preparados; não se afirma execução remota do GitHub Actions sem um repositório e um run real.
