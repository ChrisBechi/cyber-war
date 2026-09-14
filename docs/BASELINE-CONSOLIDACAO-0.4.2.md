# Baseline — consolidação 0.4.2

Data: 2026-09-14. Registro anterior às correções desta consolidação. O fórum já estava em desenvolvimento quando o pedido foi recebido; esses arquivos fazem parte deste ponto de partida. O relatório de estado atual foi lido integralmente. Nenhum resultado histórico foi usado como aprovação atual.

## Ambiente e comandos executados

Windows/PowerShell, React/TypeScript/Vite e Rust/Tauri 2. Não há repositório Git na raiz. Rust disponível em `.tools/cargo` e `.tools/rustup`; Cargo executado offline com essas variáveis configuradas.

| Verificação                                                           | Resultado observado                                                                                                          |
| --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `pnpm typecheck`                                                      | Falhou: 4 opções `exact` inválidas em consultas de testes e discriminação de rota no ForumBoard                              |
| `pnpm lint`                                                           | Falhou: 2 `require-await` nos testes do fórum; Stylelint não executado pelo script encadeado                                 |
| `node node_modules/stylelint/bin/stylelint.mjs "src/**/*.{css,scss}"` | Falhou: 236 erros de formatação no CSS novo do fórum                                                                         |
| `node node_modules/prettier/bin/prettier.cjs --check .`               | Passou                                                                                                                       |
| `pnpm content:check`                                                  | Passou: 12 missões, 11 tópicos, 2 hosts, 303 entradas de ferramentas                                                         |
| `pnpm test -- src/features/forum`                                     | Executou a suíte completa: 20 arquivos, 76 testes aprovados; execução autorizada fora da restrição de leitura do esbuild     |
| `cargo check --offline --manifest-path src-tauri/Cargo.toml`          | Passou                                                                                                                       |
| `cargo test --offline --manifest-path src-tauri/Cargo.toml`           | 36 testes: 35 passaram, 1 falhou. Teste antigo ainda espera rejeição de `bash -c ls`, embora o executor virtual já o suporte |
| Build Tauri/instaladores                                              | Adiado: os gates de tipagem, lint e teste Rust estão vermelhos; nenhum artefato antigo foi considerado validado              |
| Inspeção visual do fórum                                              | Bloqueada: abertura da aba recusada por falha da revisão automática (limite de uso). Não houve contorno por outro mecanismo  |
| Máquina Windows limpa                                                 | Indisponível neste ambiente; não ensaiada                                                                                    |

## Diagnóstico inicial

- Os dois E0716 em `man` e `killall/pkill` já foram corrigidos durante o trabalho anterior com coleções intermediárias de vida útil explícita. O Cargo atual confirma a compilação.
- Falta o comando IPC `end_session`, embora as telas o invoquem.
- `missionBaseline` contém o mundo inteiro, é global e pode apagar dados pessoais e cruzar tentativas simultâneas. Load/restore não normalizam tentativas ativas.
- Existe um único contexto de terminal no mundo.
- O novo fórum precisa finalizar gates de tipagem/lint e validação visual.

## Sequência de checkpoints

1. Estabilizar gates existentes/fórum; completar encerramento de sessão.
2. Separar efeitos temporários por missão/tentativa, migrar saves e normalizar saída/load/restore; testar persistência e concorrência.
3. Isolar terminais e reconstruir a sessão limpa; remover apenas a animação horizontal dos logs.
4. Consolidar contratos de comandos/shell/nano, VFS/associações/mídia, serviços/rede/configurações e classificação do catálogo, sem criar missões.
5. Atualizar documentação, executar regressão, então avaliar novos builds. Não declarar validação nativa/máquina limpa sem executá-la.

Os resultados posteriores serão registrados em `RELATORIO-CORRECOES-0.4.2.md`, mantendo este baseline como evidência do ponto de partida.
