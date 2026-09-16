# CYBER WAR — Milestone 0

## Architecture

A infraestrutura de inventário e verificação está implementada. O runtime ganhou
um registro canônico de nomes/disponibilidade e índices de catálogo usados pelo
autocomplete. A execução continua nos serviços existentes do mundo virtual.

O tooling DEV descobre os registros reais via Rust, valida manifestos Zod,
classifica executáveis/aliases/launchers, executa fixtures isoladas e calcula
gates, status, dashboard, fila e diff. O bridge de captura só compila em testes.
Não foi implementado outro shell ou outro terminal.

Foram preparados contratos de stdin, EOF, cancelamento, stdout/stderr em chunks,
TTY, espera de entrada e término para futuras engines interativas. Os ciclos de
archive e nano continuam no modelo existente. A auditoria identificou 76 handlers
nativos com retorno legado `GameResult<String>`; o relatório inclui também os
adaptadores de catálogo, totalizando 1.752 nomes com esse contrato.

As interfaces, fontes, política de status, limitações e workflow estão no
[README](README.md).

## Inventory

| Métrica                                       | Quantidade |
| --------------------------------------------- | ---------: |
| Nomes registrados no runtime                  |      1.793 |
| Executáveis únicos, incluindo CATALOG_ONLY    |      1.522 |
| Alias declarado (`.` → `source`)              |          1 |
| Nomes de launcher dentro do conjunto de nomes |        270 |
| Entradas de launcher do catálogo              |        303 |
| Launchers GUI: 18 internos + 1 de pacote      |         19 |
| Famílias de software                          |        279 |
| Executáveis REAL_COMPAT                       |      1.515 |
| Executáveis FICTIONAL_NATIVE                  |          7 |
| Executáveis CATALOG_ONLY                      |      1.406 |

Os conjuntos de launchers são visões diferentes, não valores para somar ao
denominador de executáveis. `sh` e `bash` continuam contratos separados; compartilhar
uma implementação não basta para declarar um alias. A descoberta também registra
bindings de payloads, propriedade dos pacotes e disponibilidade por instalação.

## Compatibility

| Estado     | Executáveis únicos | Famílias |
| ---------- | -----------------: | -------: |
| VERIFIED   |                  0 |        0 |
| PARTIAL    |                116 |       42 |
| UNVERIFIED |              1.406 |      237 |

O alias permanece PARTIAL; launchers ficam fora dessa contagem de executáveis.
Compatibilidade REAL_COMPAT: 0/1.515 executáveis e 0/243 famílias com executáveis.
Os demais grupos incluem famílias fictícias ou apenas launchers.

Nenhum software recebeu certificação por possuir `--help`, nome no catálogo ou
alguns testes passando. Esses números não medem cobertura total de funcionalidades.
Os pilotos cobrem nove nomes, oito casos com flags e quatro casos de erro.

## Subsystem Readiness

| Subsistema      | Estado  | Principal limite                                          |
| --------------- | ------- | --------------------------------------------------------- |
| SHELL           | PARTIAL | Listas lógicas, globbing e pipes concorrentes incompletos |
| VFS             | PARTIAL | Inodes, resolução de symlinks e sticky bit incompletos    |
| USERS           | PARTIAL | Identidades virtuais fixas                                |
| PERMISSIONS     | PARTIAL | Modelo limitado de proprietário/grupo/modo                |
| PROCESS         | PARTIAL | Sem ciclo genérico de todos os processos                  |
| TTY             | PARTIAL | Streams interativos genéricos pendentes                   |
| SIGNALS         | PARTIAL | Cancelamentos específicos de jobs                         |
| CLOCK           | PARTIAL | Tempo de jogo e contadores do VFS                         |
| NETWORK         | PARTIAL | Hosts/serviços sem sockets/listeners genéricos            |
| DNS             | PARTIAL | Resolver limitado aos dados virtuais                      |
| HTTP            | PARTIAL | Rotas virtuais com semântica parcial                      |
| PACKAGES        | PARTIAL | Contratos CLI nativos incompletos                         |
| ARCHIVES        | PARTIAL | Flags e streams ainda parciais                            |
| WIFI            | PARTIAL | Metadados de APs, sem frames/handshakes                   |
| PACKETS         | MISSING | Sem modelo de captura de pacotes                          |
| REMOTE_HOSTS    | PARTIAL | Contextos SSH de hosts predefinidos                       |
| SERVICES        | PARTIAL | Sem gerenciador completo de serviços                      |
| VIRTUAL_TARGETS | PARTIAL | Alvos de missão, sem engine geral                         |
| SESSIONS        | PARTIAL | Contextos de terminal/SSH limitados                       |

## Blockers

- Subsistemas não estão READY para os contratos completos das famílias.
- A maioria dos executáveis ainda é um adaptador de catálogo.
- Muitos gates de parser, flags, erros, TTY, sinais e efeitos não têm evidências.
- Não há snapshot oficial Kali ou capturas de um ambiente externo certificado.
- Ferramentas de rede e frameworks dependem de sockets, packets, serviços e sessões
  ainda incompletos. O milestone não simulou essas capacidades por textos estáticos.

## Reference Versions

Entre as 243 famílias REAL_COMPAT com executáveis, **12 têm versão fixada** e
**231 não têm**. Considerando também famílias apenas com launcher: 275 famílias
REAL_COMPAT, 12 com versão e 263 sem versão. Existem quatro famílias fictícias.

| Família        | Versão de referência declarada |
| -------------- | ------------------------------ |
| bash           | 5.2.37                         |
| coreutils      | 9.7                            |
| curl           | 8.14.1                         |
| grep           | 3.11                           |
| iproute2       | 6.15.0                         |
| nano           | 8.7                            |
| nmap           | 7.98                           |
| openssh-client | 10.0p2                         |
| procps         | 4.0.4                          |
| sudo           | 1.9.17p1                       |
| systemd        | 257.5                          |
| wget           | 1.25                           |

São referências herdadas do projeto, não atestados de equivalência. As referências
dos pilotos estão vinculadas a coreutils 9.7 e bash 5.2.37, com proveniência
`DECLARED_EXPECTATIONS`. A interface de coleta externa está preparada e não tem
fallback para executar programas no computador do jogador.

## Pilot Verification

**18/18 casos passaram**, usando o dispatch real em um `WorldState` novo por caso.

| Piloto           | Evidência                                                                            |
| ---------------- | ------------------------------------------------------------------------------------ |
| cat: seis casos  | stdin, arquivo, ausente, permissão negada, pipe e redirecionamento com efeito no VFS |
| ls: três casos   | listagem, ocultos/flags combinadas e consistência com arquivo criado por touch       |
| cd / pwd         | mudança de cwd persistida e saída correspondente                                     |
| head / tail      | stdin, flags e seleção de linhas                                                     |
| bash: três casos | `$?`, comando inexistente e stdout/stderr separados                                  |
| source / `.`     | mesmo argumento com espaço e retorno 7 propagados pelo canônico e alias              |

Os casos de ls usam diretório dedicado vazio, evitando depender dos arquivos
iniciais do jogo. Matchers preservam espaços, números e quebras de linha. Testes
do comparador cobrem efeitos de VFS, processos, rede e pacotes; os pilotos de
execução não alegam implementar engines novas para esses subsistemas.

## Generated Artifacts

- [Dashboard](../generated/cli-dashboard.md) e [inventário](../generated/cli-inventory.md).
- [Famílias](../generated/cli-software-families.md) e [fila](../generated/cli-next-work.md).
- [Evidências](../generated/cli-verification.md) e [diff](../generated/cli-diff.md).
- [Versões ausentes](../generated/unpinned-software.md), [catálogo](../generated/catalog-only-software.md),
  [parciais](../generated/partial-cli.md), [verificados](../generated/verified-cli.md) e
  [retornos legados](../generated/cli-legacy-results.md).
- JSON completo: `artifacts/cli-inventory.json`.
- JSON de fila, verificação, captura, fingerprint, diff, desempenho e resultado:
  `artifacts/cli-*.json`.
- Índice compacto: `docs/cli/inventory.json`.
- Baseline de dívida: `docs/cli-compatibility/debt-baseline.json`.

O status só existe como resultado do pipeline. O baseline protege contra regressão
e não transforma dívida em sucesso. O CI publica relatórios como artifacts.

## Tests

| Suíte                                 | Resultado                                                         |
| ------------------------------------- | ----------------------------------------------------------------- |
| Runtime Rust completo                 | 154 passaram; 4 fixtures DEV ignoradas por padrão                 |
| Frontend                              | 208 passaram em 34 arquivos                                       |
| Tooling de compatibilidade            | 22 passaram                                                       |
| Compatibilidade golden legada         | 76 casos exatos passaram                                          |
| Pilotos declarativos v2               | 18 passaram                                                       |
| Registry export / capture DEV         | Executados explicitamente com sucesso                             |
| Host guard e bloqueio real do bundler | PASS; importação DEV intencional foi rejeitada                    |
| Comparação com baseline               | Sem regressões                                                    |
| Modo normal / estrito                 | Normal passou; estrito rejeitou a dívida existente, como esperado |

As quatro fixtures ignoradas não são erros silenciados: export/capture do tooling
e exportadores visuais DEV precisam de execução explícita.

## Quality Gates

Formatação completa, lint, Stylelint, TypeScript, validação de conteúdo, rustfmt,
Clippy sem warnings, testes Rust/frontend/tooling e builds web/nativo passaram.

O executável de produção foi compilado com `tauri/custom-protocol`, incluindo os
assets web finais: `src-tauri/target/release/game-hacker.exe`, 88.832.000 bytes.
SHA-256: `B7D7416B07D2CEE217C52596480DB61E7FC0D07585595653370DD9EA9A28F2BF`.
O build release terminou em 6m13s; não foi gerado um novo instalador MSI/NSIS.

Medição local com registry em cache e `cli:inventory --check`: inventário e
verificação em aproximadamente 1,04 s; carga/validação do manifesto em 7,66 ms.
100.000 consultas: command 2,78 ms, software 4,58 ms e metadados de verificação
4,41 ms. Essa execução não escreve Markdown; os números são indicativos deste
ambiente, não limites de desempenho para CI.

O build web mantém os avisos existentes sobre tamanho de chunk e anotação de
dependência do Zod. Eles não são falhas nem foram ocultados.

## Security

O runtime CLI continua operando no mundo virtual. Não foi adicionado executor de
programas, shell, sockets ou rede externa do host. Captura e referência ficam no
tooling DEV; compile guards Rust, guard de fontes e plugin Vite protegem essa
fronteira. Os testes executam comandos virtuais reais e verificam o guard.

Essa verificação é uma proteção de fontes/build e não uma prova formal de sandbox
do sistema operacional. APIs legítimas preexistentes do aplicativo, como salvar
dados ou ler a bateria, permanecem separadas da execução de comandos do jogador.

## Next Recommended Milestone

A fila calculada escolheu **coreutils**, `W1_CORE_LINUX`, prioridade `P0`, usado em
12 missões por referência textual. A ação é `NEEDS_SUBSYSTEM`, bloqueada por
`SHELL:PARTIAL` e `VFS:PARTIAL`. O próximo milestone deve completar e testar as
capacidades desses subsistemas necessárias ao contrato de coreutils 9.7, depois
ampliar os casos dos executáveis. Bash vem em seguida na fila atual.

O vínculo a missões é uma busca de referências textuais no conteúdo, não um grafo
formal de dependências de gameplay. A prioridade e a wave explícitas continuam
precedendo essa heurística.
