# CYBER WAR — Milestone 1A: Shell Fidelity Foundation

## Architecture Changes

O terminal e os scripts agora usam o mesmo lexer contextual e AST Rust. A expansão ocorre sobre palavras estruturadas, depois do parsing, sem transformar conteúdo de variáveis em operadores. O executor reutiliza WorldState, VFS, pacotes, rede virtual e handlers existentes.

Componentes novos: `shell/syntax.rs`, `shell/expansion.rs`, `shell/executor.rs`, `shell/control.rs`, `shell_pipeline/streams.rs`, testes de fundação e medições DEV. Componentes integrados: `shell.rs`, `shell_pipeline.rs`, `terminal.rs`, contextos de terminal, comandos IPC, xterm, fila de mutações e pipeline de compatibilidade do Milestone 0.

A arquitetura e os limites estão em [architecture.md](architecture.md). Não houve alteração do tema, fontes ou cores do terminal.

## Lexer

Palavras são segmentadas em literais, parâmetros e substituições, preservando a proteção de aspas/escapes. Há spans em bytes, comentários, concatenação, argumentos vazios, operadores adjacentes e continuidade por backslash-newline. A entrada é limitada antes do parsing. Caracteres de controle rejeitados não podem produzir operadores ocultos.

## Parser / AST

Nodes: `List`, `AndOr`, `Pipeline`, `Command`, `Redirect`, `Word` e `Segment`. Pipes têm precedência maior que listas lógicas; `&&` e `||` associam à esquerda; `;` e newline separam listas. O resultado é `Complete`, `Incomplete` ou `Error`. O terminal conserva a fonte incompleta e mostra continuação; scripts incompletos falham com status 2.

Toda a fonte é validada antes da execução: erro de sintaxe no final não cria arquivos por redirecionamentos anteriores.

## Expansion

Implementados: aspas simples/duplas, escapes, `$VAR`, `${VAR}`, parâmetros posicionais, `"$@"`, `$*`, `$#`, `$?`, PID virtual `$$`, `$!` dos jobs de arquivos, IFS padrão/personalizado, `~`, globbing VFS e `$(...)` aninhado. Substituições removem newlines finais, preservam stderr e isolam cwd/env.

Globs aceitam `*`, `?`, classes e intervalos; preservam padrões sem correspondência e protegem dotfiles. A enumeração de nomes usa o índice do VFS sem copiar o conteúdo dos arquivos. Expansões têm limites incrementais de argumentos e memória; padrões têm limite de tamanho e trabalho.

## Execution

Sequências e curto-circuito consultam o status do comando anterior. Builtins de estado simples afetam o pai; pipelines e invocações de Bash/sh isolam o contexto. Scripts e comandos substituídos executam ASTs, sem host shell. A resolução usa PATH virtual, bindings de pacotes, modos executáveis e shebangs permitidos.

`unset` foi registrado como novo builtin. O payload virtual de coreutils inclui `yes`; saves antigos recebem esse binding uma única vez, preservando arquivos personalizados e exclusões posteriores.

## Streams

Pipes cooperativos: chunks de 4 KiB, capacidade de 64 KiB, estados distintos de dados/espera/EOF e fechamento do consumidor. `yes` e stdin simples de `cat`/`head` executam incrementalmente. Os demais handlers usam adaptadores limitados a 4 MiB. O último estágio determina o status do pipeline; o produtor interrompido por consumidor encerrado recebe 141.

Ctrl+C usa um canal fora do mutex do mundo; Ctrl+D fornece EOF. A saída incremental usa o contrato `cli_contract::Event` e aguarda confirmações do xterm quando há 64 KiB pendentes. O resultado agregado permanece disponível para consumidores antigos sem duplicar a renderização.

Fechar o terminal cancela comandos ativos e descarta comandos ainda na fila. Logout, load, restore e saída enviam cancelamento antes de aguardar a fila de mutações. Há testes de cancelamento com entrada pendente e com saída bloqueada por backpressure.

## Redirections

Suportados `<`, `0<`, `>`, `1>`, `>>`, `1>>`, `2>`, `2>>` e duplicações de stdout/stderr entre 1 e 2. A ordem é preservada: `>out 2>&1` difere de `2>&1 >out`. Há redirecionamento sem comando, `/dev/null` virtual e erro explícito para destino ambíguo.

Descritores arbitrários, fechamento de descritor, heredocs e pipes binários continuam fora do subconjunto. Arquivos e permissões permanecem sujeitos aos limites do VFS.

## Environment

Atribuições isoladas persistem no shell. Prefixos de atribuição são temporários e exportados para o comando; `export NAME`, `export NAME=value`, `export -p` e `unset` são suportados. Scripts filhos recebem variáveis exportadas e o ambiente virtual padrão. `source` compartilha cwd/env, restaura argumentos posicionais e respeita `return`; `exit` encerra a invocação de script.

Os novos estados do shell usam `serde(skip)` e defaults; não são gravados no save. A extensão pontual do binding de `yes` não altera o schema global de saves.

## Process / Jobs

Cada estágio recebe PID virtual e uma linha transitória em `WorldState.processes`, removida ao terminar. Não há criação de processos do computador real. Os jobs existentes de arquivos/pacotes foram preservados.

**Parcial:** background geral, `fg`/`bg`/`wait`, grupos de processos e preempção dentro de handlers legados. A execução ainda ocupa a transação do mundo até completar; operações de outros aplicativos que dependem dela podem aguardar. O canal de entrada/cancelamento/saída é independente dessa transação.

## Compatibility Evidence

Foram acrescentados **50 casos declarativos**, totalizando **68** na suíte schema v2, além dos **76 casos exatos anteriores**. IDs e resultados completos: [shell-compatibility.md](../generated/shell-compatibility.md).

Categorias: parsing/aspas, listas/status, atribuições/ambiente, IFS/posicionais/tilde, globs, substituição, pipes/EOF/cancelamento, redirecionamentos e contexto/resolução. Os testes Rust acrescentam spans, entrada incompleta, 10.000 entradas geradas para o parser, limites de expansão, canais cheios, cancelamento durante espera, save/load e compatibilidade de bindings.

Referência: [manual do Bash 5.2.37 no Debian trixie](https://manpages.debian.org/trixie/bash/bash.1.en.html). A proveniência permanece `DECLARED_EXPECTATIONS`: não foram inventadas capturas de uma instalação externa, nem declarada certificação integral do Bash.

## Performance

Medições em perfil debug, com aquecimento e repetições, capturadas pelo pipeline em `artifacts/shell-performance.json`. O [relatório gerado](../generated/shell-compatibility.md#performance) apresenta as médias atuais e só incorpora o artefato quando seu hash coincide com a evidência capturada.

| Cenário                                                 |      Média |
| ------------------------------------------------------- | ---------: |
| Parsing simples, 10.000 repetições                      |   0,029 ms |
| Parsing de script de 11.776 bytes, 100 repetições       |  15,611 ms |
| Pipeline com 3 estágios e 100.000 linhas, 10 repetições |  34,888 ms |
| Glob com 100 entradas, 5 repetições                     |   3,913 ms |
| Glob com 1.000 entradas, 5 repetições                   |  15,401 ms |
| Glob com 9.957 entradas, 5 repetições                   | 125,577 ms |

O pico observado em um pipe foi **4.096 bytes**, abaixo da capacidade de **65.536 bytes**. São medidas locais de tempo de parede em debug, sem garantia de desempenho para outras máquinas.

O cenário de globs solicitado com 10.000 entradas usa 9.957 arquivos de teste: os 43 nós básicos ocupam o restante da capacidade real de 10.000 nós do VFS. A capacidade não foi artificialmente aumentada para a medição.

## Security

O runtime novo acessa apenas recursos virtuais. Os guards verificam fontes de produção e a barreira de importação do build; os testes comprovam rejeição de tentativas de comandos/paths do host. A ferramenta de captura e medição de referência continua restrita a DEV/test.

Há limites de fonte, profundidade, quantidade de estágios, expansão, trabalho de globbing, filas de stdin/saída/pipes e adaptadores. Excesso retorna erro explícito. O limite de scrollback do xterm não reduz os dados dos pipes nem dos arquivos.

## Regressions

Passaram 166 testes Rust, 211 testes frontend e 24 testes da infraestrutura CLI. A captura de compatibilidade passou pelos 17 testes Rust selecionados, incluindo a matriz exata anterior, e pelos 68 casos declarativos atuais. A validação de conteúdo passou com 12 missões, 11 threads, 8 hosts e 303 entradas Kali. As expectativas alteradas correspondem a recursos agora suportados (`$(...)`, stdin interativo) ou ao novo diagnóstico estruturado com posição; não houve normalização ampla de stdout/stderr para esconder diferenças.

## Shell Readiness

Anterior: `SHELL = PARTIAL`, sem listas gerais, globbing ou pipes cooperativos.

Novo: parsing, listas, expansão, globbing, pipelines, redirecionamentos e contexto possuem READY dentro do subconjunto e condicionado à evidência atual. Jobs e o estado agregado conservador de SHELL continuam PARTIAL pelas limitações descritas. Bash como software completo permanece PARTIAL.

O manifest permite dependências explícitas por capacidade. A verificação rebaixa automaticamente uma capacidade se faltar captura atual ou se qualquer teste exigido falhar. A dívida anterior não foi sobrescrita.

## Coreutils Blockers

As dependências de shell de coreutils 9.7 agora apontam para as sete capacidades de fundação exigidas. A ausência de job control geral não bloqueia comandos que não o utilizam. O bloqueio de VFS permanece: inodes, travessia POSIX de symlinks, permissões especiais e demais contratos de arquivos não foram implementados por este marco.

## Generated Pipeline Results

As contagens finais e a próxima ação vêm de `cli:inventory`, `cli:compat` e `cli:verify`; não são certificações manuais. Os resultados atuais estão no [dashboard](../generated/cli-dashboard.md) e na [fila de implementação](../generated/cli-next-work.md).

| Métrica                                  |      Resultado |
| ---------------------------------------- | -------------: |
| Nomes de comandos                        |          1.794 |
| Executáveis únicos                       |          1.523 |
| Famílias de software                     |            279 |
| Comandos VERIFIED                        |              0 |
| Comandos PARTIAL                         |            117 |
| Comandos UNVERIFIED                      |          1.406 |
| CATALOG_ONLY                             |          1.406 |
| Software VERIFIED / PARTIAL / UNVERIFIED |   0 / 42 / 237 |
| Casos declarativos PASS / FAIL / SKIPPED |     68 / 0 / 0 |
| Gates obrigatórios aprovados / totais    | 3.190 / 35.022 |

`CATALOG_ONLY` é uma classificação e se sobrepõe aos comandos `UNVERIFIED`; não é uma quarta parcela somável. O novo builtin `unset` explica o acréscimo de um nome/executável e de um comando parcial em relação ao inventário anterior.

`cli:verify` passou com **0 erros**. `cli:verify --strict` saiu com código 1 e **1.517 erros**, pois a dívida global de verificação permanece. Os critérios estritos e a baseline de regressão foram preservados. Nenhum software foi promovido a VERIFIED por conveniência.

## Next Recommended Milestone

A fila recalculada retornou `coreutils`, ação `NEEDS_SUBSYSTEM`, com somente `SUBSYSTEM:VFS:PARTIAL` como bloqueio. Portanto, a recomendação é **VFS/POSIX Fidelity**, seguida da auditoria vertical dos contratos coreutils 9.7. Essa conclusão é derivada do grafo atual. Jobs gerais e migração dos handlers legados para consumo incremental permanecem trabalho explícito, sem bloquear capacidades que já possuem evidência suficiente.

## Validação final

| Verificação                        | Resultado                                                              |
| ---------------------------------- | ---------------------------------------------------------------------- |
| TypeScript e ESLint sem warnings   | PASS                                                                   |
| Stylelint e validação de conteúdo  | PASS                                                                   |
| Rustfmt e Clippy com `-D warnings` | PASS                                                                   |
| Rust                               | 166 PASS, 0 FAIL; 5 testes DEV/visuais ignorados na execução padrão    |
| Frontend                           | 211 PASS, 35 arquivos                                                  |
| Infraestrutura CLI                 | 24 PASS                                                                |
| Captura de compatibilidade         | 68/68 casos; export, captura e benchmark DEV executados explicitamente |
| Verificação normal                 | PASS, 0 erros                                                          |
| Verificação estrita                | FAIL esperado pela dívida global, 1.517 erros                          |
| Build web de produção              | PASS                                                                   |
| Build Windows de produção          | PASS, perfil release otimizado, 7 min 05 s                             |
| Prettier                           | PASS em todos os arquivos                                              |

O build web manteve os avisos existentes sobre tamanho do chunk principal e anotação PURE de uma dependência. A validação desta entrega é automatizada; não foi realizada inspeção manual da interação no executável Windows.

Executável produzido em 16/09/2026: `src-tauri/target/release/game-hacker.exe`, **88.930.816 bytes**. Compilação offline com `tauri/custom-protocol`, incorporando o build web; não depende de WSL, Git Bash, Cygwin, Kali ou ferramentas GNU para executar comandos virtuais.

SHA-256: `1306AFFFA1A90544BB3A5EE2F96923C1BB236566FB39E581FCF51A57A443DC91`.

Os logs desta validação estão em `artifacts/m1-*.log`. O executável foi compilado com sucesso; esta etapa não gerou instalador nem realizou teste manual da aplicação.
