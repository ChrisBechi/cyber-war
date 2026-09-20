# M1C.8 — sha256sum GNU 9.7 certificado

**STOP_REASON: `MILESTONE_COMPLETE`. PUBLICATION_STATUS: `PENDING`.**

| Medida                         | Antes                                                 | Depois                                       |
| ------------------------------ | ----------------------------------------------------- | -------------------------------------------- |
| sha256sum                      | PARTIAL; arquivos e listas acumulados                 | VERIFIED; hashing e verificação incrementais |
| Casos de projeto / GNU         | 6 / 0                                                 | 585 PASS / 585 matches                       |
| Contratos / gates obrigatórios | Cobertura incompleta                                  | 18/18 PASS; 26/26 PASS                       |
| Known gaps obrigatórios        | Escaping, listas malformadas, warn/strict/zero, stdin | 0                                            |
| Coreutils                      | 10 VERIFIED / 27 PARTIAL                              | 11 VERIFIED / 26 PARTIAL                     |
| GNU global                     | 1641 matches                                          | 2226 matches                                 |
| Declarativa global             | 1753 PASS                                             | 2332 PASS; 0 FAIL; 0 SKIPPED                 |

Fingerprint: `e7c115e28430fa88f1f447c66859012108eb35fb282ed80f9b201f5c046f5764`.
Valores derivados, dependências, gates e hashes dos instaladores estão nos snapshots
[before](../coreutils/evidence/m1c8-before.json) e [final](../coreutils/evidence/m1c8-final-state.json).

**Arquitetura.** SHA-256 usa o crate interno sha2, com estado incremental por
arquivo. O scheduler fornece blocos de até 4096 bytes via VFS, pipes e VirtualTty.
O engine mantém separadamente o descritor da lista e o arquivo em verificação;
nenhum deles é materializado inteiro. Parser, emissão, verificação e resumo de
diagnósticos são componentes distintos. Sinais, backpressure e fechamento usam
a infraestrutura existente. O antigo handler com buffering foi removido.

**Formato e parsing.** As expectativas vêm exclusivamente de duas execuções
idênticas do GNU locked, seguidas de verify independente. As listas de roundtrip
são emissões GNU preservadas em fixtures e verificadas contra o golden canônico.
O corpus cobre formatos normal, binário e tag, escaping, zero, listas válidas,
malformadas, erros de filesystem, múltiplos operands e opções incompatíveis.
O [fonte GNU v9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/digest.c)
orientou os probes, sem fornecer expectativas calculadas pelo projeto.

Diferenças observadas e corrigidas: `--zero` desativa escaping; a última opção
entre warn/quiet/status controla a apresentação; status ainda permite erros de
abertura e ausência de registros válidos. Malformações não tornam o resultado
malsucedido sem strict quando há verificações bem-sucedidas. O parser aceita
listas BSD invertidas com estado persistente, trata comentários, CRLF e NUL
conforme as observações. Uma lista em stdin rejeita registros de arquivo `-`;
uma lista regular pode verificar stdin, cuja segunda leitura observa EOF.
Ignore-missing com todos os arquivos ausentes falha por não verificar arquivo algum.

**Limites e propriedades.** Certificação no locale C e paths UTF-8 do VFS.
O buffer de registro tem limite deliberado de 8320 bytes (duas vezes o limite
de path VFS, mais cabeçalho), além de um chunk; excesso é drenado sem crescimento.
Whitespace inicial é compactado sem alterar significado e comentários extensos
são ignorados. Nomes binários inexistentes preservam bytes em resultados e
diagnósticos; não há conversão lossy no parser. Não há snapshot de arquivos:
leituras usam os handles existentes. Não foi ampliada a política de mutações do VFS.

Propriedades cobrem vetores SHA-256 conhecidos, chunks 1, 2, 3, 7, 31, 63, 64,
65, 255, 1024 e 4096, fragmentação pseudoaleatória, bytes arbitrários, roundtrips
escapados e parser sem panic. Pipelines reais testam mais de 4 MiB, redireção e
append; TTY Pending/Data/EOF, produtores infinitos controlados, SIGINT/SIGTERM
e limpeza de handles/processos/waits também são exercitados.

**Regressão e evidência.** Os dez goldens anteriores, com 1641 observações,
permaneceram byte a byte intactos. A mudança compartilhada do harness exigiu
revalidação dupla com recibos separados; uma interação TTY de base64 teve
divergência entre execuções e passou na repetição, sem mudar expectativas.
Os 564 casos iniciais de sha256sum permaneceram iguais após 21 regressões adicionais.
O capturador DEV agora observa o status real do executável no primeiro estágio
de pipes, preservando o status normal do último estágio no shell. A coleta
continua usando índice e arquivos por caso, com cache limitado; não há JSON
global monolítico nem execução host no gameplay.

| Validação final                                                                | Resultado                                                               |
| ------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| Rust completo, serial                                                          | 287 PASS; 14 opcionais ignorados                                        |
| Frontend                                                                       | 243 PASS em 40 arquivos                                                 |
| Tooling / legacy                                                               | 54 / 76 PASS                                                            |
| Declarativa global                                                             | 2332 PASS                                                               |
| Normal, regressão e strict dos onze certificados                               | PASS                                                                    |
| Strict global                                                                  | FAIL esperado: 1509 entradas da dívida conhecida; zero erros adicionais |
| rustfmt, Clippy -D warnings, Prettier, ESLint, Stylelint, TypeScript, conteúdo | PASS                                                                    |
| Web, release Windows, NSIS/MSI e Host Guard                                    | PASS                                                                    |

Empacotamento 0.4.2: somente o recurso de licenças; nenhum externalBin.

- NEXT QUEUE ITEM: `coreutils/readlink`.
- CLASSIFICATION: `OUT_OF_SCOPE_FAMILY`.
- COMPLEXITY: 1/5.
- ACTION: `NEEDS_IMPLEMENTATION_AND_EVIDENCE`.
- KNOWN GAPS: Canonicalization flags, quiet/verbose, no-newline and multi-operand behavior incomplete.
- DEPENDENCIES: SHELL.CONTEXT, SHELL.EXPANSION, SHELL.GLOBBING, SHELL.LISTS, SHELL.PARSING, SHELL.PIPELINES, SHELL.REDIRECTION, VFS.DIRECTORIES, VFS.HARDLINKS, VFS.INODES, VFS.METADATA, VFS.PATHS, VFS.PERMISSIONS, VFS.REGULAR_FILES, VFS.SPECIAL_PERMISSIONS, VFS.SYMLINKS, VFS.TIMESTAMPS.
- WAVE_READINESS: **NOT_READY**.

Pela política de wave herdada (reading/text), os próximos cinco itens têm 0 SMALL_SHARED_EXTENSION e 5 OUT_OF_SCOPE_FAMILY: readlink (1/5), rmdir (1/5), ln (2/5), mkdir (2/5), realpath (2/5). Há 5/5 conjuntos de dependências prontos, 0 bloqueios de subsistema e 0 fronteiras arquiteturais sinalizadas. A transição de família requer uma wave com escopo próprio; esta política não a autoriza automaticamente.
Nenhum comando seguinte foi implementado.
