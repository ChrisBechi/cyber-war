# M1C.5 — Base64 GNU 9.7 certificado

**STOP_REASON: `MILESTONE_COMPLETE`. PUBLICATION_STATUS: `PUBLISHED`.**

| Medida                              | Antes                                          | Depois                                                |
| ----------------------------------- | ---------------------------------------------- | ----------------------------------------------------- |
| Base64                              | PARTIAL, handler em `coreutils/bytes.rs`       | VERIFIED, engine incremental em `coreutils/base64.rs` |
| Casos de projeto Base64             | 7                                              | 304 PASS                                              |
| Referências GNU Base64              | 0                                              | 304/304 matches, capture duplo e verify independente  |
| Contratos Base64                    | 6                                              | 14/14 PASS                                            |
| Gates obrigatórios Base64           | Evidência local desatualizada                  | 26/26 PASS                                            |
| Known gaps obrigatórios             | Decode parcial e diagnóstico de input inválido | 0                                                     |
| Coreutils, snapshot local           | 0 VERIFIED / 37 PARTIAL                        | 8 VERIFIED / 29 PARTIAL                               |
| Coreutils, baseline publicada M1C.4 | 7 VERIFIED / 30 PARTIAL                        | 8 VERIFIED / 29 PARTIAL                               |
| Casos declarativos globais          | 881 sem execução local fresca                  | 1178 PASS, 0 FAIL, 0 SKIPPED                          |
| Próximo item calculado              | Base64 na baseline publicada                   | tee; não implementado                                 |

Fingerprint da execução final:
`18985deb61304b97ace0c8202a03e9278dc8431c09d70841c04f8111e815caa8`.

Os snapshots [before](../coreutils/evidence/m1c5-before.json) e
[final](../coreutils/evidence/m1c5-final-state.json) registram estado, dependências,
contratos, gates, hashes de captura, builds e fila derivados do tooling. A cópia
local inicial tinha alterações compartilhadas sem execução fresca; por isso o
pipeline rebaixava os sete comandos previamente publicados. Os números locais
não foram confundidos com a certificação histórica.

## Implementação e limites de memória

O handler anterior acumulava a entrada inteira e tratava falha de decode como
tudo-ou-nada. O novo engine usa o scheduler compartilhado de streams, handles
VFS, pipes de 64 KiB, VirtualTty, backpressure e sinais direcionados por PID.

O estado de transformação ocupa no máximo 64 bytes: encode mantém carry de
0–2 bytes e coluna de wrapping; decode conserva quartetos, padding e bits
residuais. O conteúdo permanece binário, sem conversão intermediária para String.

Os adaptadores compartilhados em `shell/stdio.rs` reproduzem o buffering da
baseline: blocos de entrada de 30.720 bytes para encode e 4.096 para decode,
mais stdout musl limitado a 1.024 bytes entre escritas. O scheduler alimenta
blocos de até 4.096 bytes. Memória transitória e saída pendente ficam limitadas
por esses tamanhos, sem crescer com a entrada total. O limite geral de captura
de saída continua 4 MiB.

O encode publica blocos completos antes de EOF e preserva carry e wrapping entre
chunks. O decode retém apenas a parte incompleta do quarteto entre blocos e
publica seu prefixo válido no erro ou EOF. Sinais descartam buffers ainda não
publicados e liberam descritores/processos. Não foi criado executor host.

## Descoberta GNU e diferenças corrigidas

As primeiras 256 observações estão preservadas como
[GNU_PROBE histórico](../coreutils/evidence/m1c5-base64-finite-probe.json).
A certificação usa o corpus canônico de 304 casos em
`tests/cli/gnu/coreutils/9.7/base64.json`, nunca o probe isoladamente.
Todos os expected vieram do binário locked Alpine 3.22.0 / Coreutils 9.7-r1,
locale C, em duas execuções idênticas, seguidas de reprodução independente.

| Observação GNU 9.7                                        | Consequência implementada                          |
| --------------------------------------------------------- | -------------------------------------------------- |
| `Zg` decodifica para byte 66, status 0                    | Padding ausente não é rejeitado genericamente      |
| `Zg=` produz byte 66 e erro                               | Preservar saída parcial antes do diagnóstico       |
| `Zg==Zg==` produz bytes 6666                              | Aceitar grupos completos concatenados              |
| `Zh==` produz byte 66 e erro de bits residuais            | Validar estrutura sem apagar o prefixo             |
| LF é aceito; CR não é ignorado sem `-i`                   | Não usar uma regra genérica de whitespace          |
| `-w -0` é aceito; espaço final no número é inválido       | Reproduzir parsing observado                       |
| `u64::MAX` e overflow decimal positivo desativam wrapping | Tratar a sentinela como largura zero, sem LF final |

A comparação inicial encontrou três divergências de wrapping extremo: o
runtime ainda acrescentava LF para a sentinela. A correção foi feita no parser
e entrou no teste de independência de chunks.

Os probes interativos adicionais mostraram outra diferença: a primeira versão
incremental publicava pequenos chunks cedo demais. O GNU aguarda completar seu
bloco de leitura ou EOF. Foram adicionados os adaptadores de IO compartilhados,
o atraso do quarteto incompleto e a publicação de linhas compatível com musl.
Dez casos com interrupção comparam os bytes já visíveis, inclusive 41.040 bytes
no probe de encode maior. O teste nativo repete esses casos com chunks de
1, 7, 255 e 4.096 bytes.

O [código GNU v9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/basenc.c)
orientou os probes de buffering; a execução do oracle determinou os resultados.

## Saída parcial, arquivos e integração

Para `Zm9vZg!` com `-d`, GNU grava `666f6f66` (foof), escreve
`base64: invalid input\n` em stderr e termina com status 1. O runtime reproduz
isso em stdout, pipe, `>` e `>>`; o append conserva também o conteúdo anterior.

O corpus cobre entrada vazia, NUL, bytes 0–255, UTF-8 inválido, CRLF, boundaries,
padding, whitespace/lixo, erros tardios após múltiplos blocos, opções combinadas,
abreviações, precedência, aliases de invocação, help/version completos, arquivos,
permissões, diretórios, symlinks e hardlinks.

Os testes de transformação usam chunks de 1, 2, 3, 4, 5, 7, 31, 255 e 4.096
bytes, além de fragmentação pseudoaleatória determinística. Há fuzz limitado
e corpus diferencial GNU, sem usar roundtrip como oracle. O produtor infinito
`base64 /dev/zero | head -c 100000` exercita backpressure, consumidor encerrado
e limpeza de handles. A disponibilidade real do pacote foi testada.

As amostras debug de encode de 65.539 bytes e decode para 65.539 bytes estão
registradas no snapshot final, com duração e tamanho de saída. São medidas
locais do tooling, sem transformá-las em gate de compatibilidade ou promessa
de desempenho.

## Recertificação compartilhada

Mudanças no parser, streams, harness e lock invalidaram legitimamente a
evidência anterior. Foram renovadas e verificadas independentemente as
754 referências dos sete comandos: basename 48, dirname 27, printenv 36,
whoami 31, cat 91, head 215 e tail 306. Com Base64, são **1058/1058 matches GNU**.

O harness TTY agora reconhece leitura `readv` de musl, inclusive quando WSL
expõe `wchan=0`, e aguarda a fila canônica esvaziar antes de enviar o sinal.
O bridge virtual verifica input consumido e usa bytes brutos nas observações.

A revisão final também excluiu caches Python de execução da enumeração de
fontes do fingerprint. Esses arquivos temporários são ignorados pelo Git;
a evidência final foi renovada usando apenas as fontes relevantes.

A recertificação revelou uma corrida nos probes direcionados antigos de tail:
uma espera de kernel podia ainda representar o bloqueio anterior ao append.
Os requests passaram a aguardar quantidades explícitas de saída já observadas
no GNU. Nos dois probes de bytes, a barreira inclui o prefixo inicial.
As expectativas de bytes foram preservadas. A captura final e o verify
independente dos 306 casos passaram; um teste do tooling previne a regressão
da contagem dessas barreiras.

`VFS.EVENTS`, `VFS.WATCH`, `TTY.CANONICAL_IO` e `SIGNALS.STREAMS`
permanecem READY pelo pipeline. A dívida dos subsistemas gerais não foi ocultada.

## Validação final

| Verificação                                        | Resultado                                                                           |
| -------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Rust completo                                      | 271 PASS, 0 FAIL, 14 testes opcionais ignorados                                     |
| Frontend completo                                  | 243 PASS em 40 arquivos                                                             |
| Tooling                                            | 45 PASS, 0 FAIL                                                                     |
| Legacy                                             | 76 casos exatos PASS                                                                |
| Declarativa global                                 | 1178 PASS, 0 FAIL, 0 SKIPPED                                                        |
| GNU                                                | 1058 matches; capture duplo e verify independente dos oito comandos                 |
| Verificação normal e regressão                     | PASS                                                                                |
| Strict Foundation, cat, head, tail e base64        | PASS; todos os oito comandos preservados                                            |
| Strict global                                      | FAIL esperado: 1512 entradas de dívida fora deste fechamento; nenhum erro adicional |
| rustfmt e Clippy                                   | PASS, all-targets/all-features com -D warnings                                      |
| Prettier, ESLint, Stylelint, TypeScript e conteúdo | PASS                                                                                |
| Web build e Windows release                        | PASS                                                                                |
| Instaladores NSIS e MSI                            | PASS                                                                                |
| Host Guard                                         | PASS                                                                                |

A suíte Rust completa é serial porque fixtures de integração compartilham
controles globais de cancelamento. A execução concorrente anterior sofreu um
timeout em tail; o caso passou isolado e na suíte serial. O comando `rust:test`
e o CI usam a mesma execução serial, sem excluir testes nem ampliar timeouts.
O pipeline também executou os testes opcionais de captura e benchmarks exigidos.

O reporter de QA web possui exclusão de compilação no próprio arquivo para
builds release. O Host Guard exige esse gate e testa que sua remoção volta a
reprovar IO host. O manifesto de empacotamento não inclui Python, WSL, harness
ou referências GNU; esses componentes continuam DEV/CI only.

## Build entregue

- Executável: `src-tauri/target/release/game-hacker.exe`.
- NSIS: `src-tauri/target/release/bundle/nsis/Cyber War_0.4.2_x64-setup.exe`.
- MSI: `src-tauri/target/release/bundle/msi/Cyber War_0.4.2_x64_en-US.msi`.

Tamanhos e SHA-256 estão no snapshot final. O erro anterior de acesso negado ao
empacotador MSI foi resolvido na execução autorizada do build final.

## Próximo item calculado — não iniciado

- NEXT QUEUE ITEM: `coreutils/tee`.
- CLASSIFICATION: `SMALL_SHARED_EXTENSION`.
- COMPLEXITY: 2/5.
- ACTION: `NEEDS_IMPLEMENTATION_AND_EVIDENCE`.
- KNOWN GAPS: modos de ignorar sinais/output-error; stdin acumulado até EOF.
- DEPENDENCIES: SHELL.CONTEXT, EXPANSION, GLOBBING, LISTS, PARSING, PIPELINES,
  REDIRECTION; VFS.DIRECTORIES, HARDLINKS, INODES, METADATA, PATHS, PERMISSIONS,
  REGULAR_FILES, SPECIAL_PERMISSIONS, SYMLINKS e TIMESTAMPS.

A fila foi recalculada com evidência fresca. O limite solicitado encerra este
milestone após Base64, mesmo que o tooling permita continuar a wave.

## Continuidade de interface e saves

As alterações anteriores também estão na build: lixeira por save ocupado com
confirmação, exclusão transacional de snapshots/autosave/checkpoints, preservação
dos cinco slots e dos demais saves, confirmação de sobrescrita, “Sessão X”
abaixo do nome e remoção da tela “Você começou…”. O primeiro save foi verificado
sobre uma cópia; o banco original foi preservado. A homologação em outra máquina
continua registrada separadamente, conforme a escolha do usuário.

Logs, target, node_modules e capturas exploratórias locais permanecem fora do
commit. O remoto aceitou o push normal para `main`: `8dcdc8e` registra a
internet virtual e os saves; `7a37649` registra a certificação Base64 M1C.5.
A atualização de publicação altera somente documentação e não exige repetir
as suítes já aprovadas.
