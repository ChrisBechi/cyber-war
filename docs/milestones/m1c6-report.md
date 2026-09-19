# M1C.6 — Tee GNU 9.7 certificado

**STOP_REASON: `MILESTONE_COMPLETE`. PUBLICATION_STATUS: `PUBLISHED`.**

| Medida                             | Antes                                                  | Depois                                               |
| ---------------------------------- | ------------------------------------------------------ | ---------------------------------------------------- |
| tee                                | PARTIAL, `coreutils/bytes.rs`                          | VERIFIED, `coreutils/tee.rs`                         |
| Casos de projeto                   | 4                                                      | 162 PASS                                             |
| Evidência GNU                      | 0                                                      | 162/162 matches; captura dupla e verify independente |
| Contratos                          | 6 definidos, cobertura incompleta                      | 13/13 PASS                                           |
| Gates obrigatórios                 | Incompletos                                            | 26/26 PASS                                           |
| Known gaps obrigatórios            | Signals/output-error ausentes; stdin acumulado até EOF | 0                                                    |
| Coreutils                          | 8 VERIFIED / 29 PARTIAL                                | 9 VERIFIED / 28 PARTIAL                              |
| Compatibilidade declarativa global | 1178 casos                                             | 1336 PASS, 0 FAIL, 0 SKIPPED                         |

Fingerprint final:
`03d301a2141af39d5a90eb3c023d340fd5ceb44f01070d81eebb5f799905eb02`.
Os snapshots [before](../coreutils/evidence/m1c6-before.json) e
[final](../coreutils/evidence/m1c6-final-state.json) registram contratos, gates,
dependências, referências e builds derivados do tooling.

## Streaming e fan-out

O handler anterior copiava todo stdin para stdout e depois escrevia cada arquivo.
O novo engine abre os destinos na ordem dos operands, conserva handles VFS e
processa blocos de até 1024 bytes. O scheduler entrega stdout antes dos arquivos
para cada bloco. A memória de trabalho usa buffers fixos e estado por destino;
o conteúdo persistido permanece no VFS. Os pipes compartilhados mantêm 64 KiB.

Os testes cobrem criação, truncamento, append, arquivos repetidos, symlinks,
hardlinks, redirecionamentos e produtores finitos e infinitos controlados. Bytes
NUL, UTF-8 inválido, CRLF e entradas sem LF não passam por transformação textual.
As propriedades verificam fragmentação de 1, 2, 3, 7, 31, 255 e 4096 bytes e uma
sequência pseudoaleatória determinística, inclusive `cat | tee -a A B C > out`.

Sucesso, falhas de abertura/escrita e sinais liberam os recursos compartilhados.
Um teste de esgotamento tardio do VFS conserva o prefixo já gravado e mantém o
destino ainda utilizável. `/dev/full` foi acrescentado ao VFS compartilhado:
abre normalmente, aceita leitura de zeros e falha na escrita. A abertura com
truncamento não escreve em dispositivos.

## Descobertas GNU e diferenças corrigidas

O corpus canônico está em `tests/cli/gnu/coreutils/9.7/tee.json`. As expectativas
foram adotadas das execuções do binário locked Alpine/Coreutils 9.7, locale C.
O [código GNU v9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/tee.c)
orientou os probes; não foi usado para inventar expected.

| Modo                                  | Pipe de saída fechado              | Falha em arquivo ou stdout que não seja pipe |
| ------------------------------------- | ---------------------------------- | -------------------------------------------- |
| Padrão                                | Termina por SIGPIPE                | Diagnostica e continua nos destinos válidos  |
| `warn`                                | Diagnostica e continua             | Diagnostica e continua                       |
| `warn-nopipe`, `-p`, `--output-error` | Desativa o destino sem diagnóstico | Diagnostica e continua                       |
| `exit`                                | Diagnostica e termina com status 1 | Diagnostica e termina com status 1           |
| `exit-nopipe`                         | Desativa o destino sem diagnóstico | Diagnostica e termina com status 1           |

Os modos exit também interrompem a abertura dos operands quando ela falha.
Efeitos anteriores permanecem: em `tee --output-error=exit A /dev/full B`, o
bloco chega a stdout e A; B já foi criado, mas fica vazio. No modo warn, B recebe
os mesmos bytes. Diagnósticos de abertura são preservados mesmo se stdout falha.

`-` é um nome de arquivo comum. `--output-error` sem `=` não consome o próximo
operand como modo. Prefixos como `w`, `e` e o argumento vazio são ambíguos;
`warn-n` é aceito. Append em aliases duplica cada bloco no mesmo inode; os casos
com stdin de arquivo confirmam o tamanho de leitura observado de 1024 bytes.
Help e version completos vieram do mesmo pipeline de captura.

`-i` e `--ignore-interrupts` usam a disposição ignorada do estado compartilhado
de sinais. SIGTERM continua terminante. Probes de escrita bloqueada preservam
exatamente 65536 bytes em stdout, A e B antes de SIGINT/SIGTERM, inclusive com
`-i` seguido de SIGTERM. Casos TTY verificam publicação antes de EOF.

Foram corrigidas a rejeição prematura de `/dev/full` na abertura e a perda de
diagnóstico pendente ao desativar stdout. No harness, a espera reconhece o poll
usado por `tee -p`; sinais durante backpressure aguardam o término antes de
drenar o pipe, impedindo que a coleta libere uma escrita adicional.

## Regressão e validação

O tooling invalidou as referências anteriores pelos hashes do harness/lock
compartilhados. Elas foram recapturadas e verificadas independentemente uma vez
após estabilizar essas fontes. Os **1058 registros anteriores mantiveram requests,
bytes, status e efeitos idênticos**; somente metadados de captura foram renovados.
Com tee, são **1220/1220 matches GNU**. Os oito VERIFIED anteriores foram preservados.

| Verificação                                                                      | Resultado                                                               |
| -------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Rust completo, serial                                                            | 276 PASS; 14 testes opcionais ignorados                                 |
| Coreutils na execução final                                                      | 35 PASS; inclui o teste atualizado de append via produtor               |
| Frontend, dois workers                                                           | 243 PASS em 40 arquivos; assertions e timeouts preservados              |
| Tooling                                                                          | 47 PASS                                                                 |
| Legacy                                                                           | 76 casos exatos PASS                                                    |
| Declarativa global                                                               | 1336 PASS, 0 FAIL, 0 SKIPPED                                            |
| Normal, regressão e strict dos nove certificados                                 | PASS                                                                    |
| Strict global                                                                    | FAIL esperado: 1511 entradas da dívida conhecida; zero erros adicionais |
| rustfmt, Clippy `-D warnings`, Prettier, ESLint, Stylelint, TypeScript, conteúdo | PASS                                                                    |
| Web build, Windows release, NSIS e MSI                                           | PASS                                                                    |
| Host Guard                                                                       | PASS                                                                    |

O pipeline também executou os testes opcionais de captura e desempenho exigidos.
O empacotamento declara somente o recurso de licenças de arquivos e nenhum
executável externo. O oracle e o harness continuam restritos a DEV/CI.

## Entrega e próximo item

- Executável: `src-tauri/target/release/game-hacker.exe`.
- NSIS: `src-tauri/target/release/bundle/nsis/Cyber War_0.4.2_x64-setup.exe`.
- MSI: `src-tauri/target/release/bundle/msi/Cyber War_0.4.2_x64_en-US.msi`.

Tamanhos e SHA-256 estão no snapshot final. Logs, probes exploratórios,
artifacts, target e node_modules não fazem parte da publicação.

- NEXT QUEUE ITEM: `coreutils/wc`.
- CLASSIFICATION: `SMALL_SHARED_EXTENSION`.
- COMPLEXITY: 2/5.
- ACTION: `NEEDS_IMPLEMENTATION_AND_EVIDENCE`.
- KNOWN GAPS: display width e files0-from não suportados; classificação de
  palavras com UTF-8 inválido precisa de referência GNU.
- DEPENDENCIES: SHELL.CONTEXT, EXPANSION, GLOBBING, LISTS, PARSING, PIPELINES,
  REDIRECTION; VFS.DIRECTORIES, HARDLINKS, INODES, METADATA, PATHS, PERMISSIONS,
  REGULAR_FILES, SPECIAL_PERMISSIONS, SYMLINKS e TIMESTAMPS.

`wc` não foi implementado. O milestone encerra após a certificação de tee.

Publicado na `main`: commit `3f0dcf5` (`feat(coreutils): certify tee against GNU 9.7`).
