# CYBER WAR — Milestone 1C

GNU Coreutils 9.7 Compatibility & Verification. Relatório técnico de 17/09/2026.

## Baseline

A baseline do projeto é **GNU Coreutils 9.7**, lida do manifest M0. O runtime continua sendo uma implementação Rust virtual: imprimir essa versão não significa executar os binários GNU. O ambiente local disponível possui Coreutils 8.32 do Git; ele não foi aceito como referência 9.7. Não há captura GNU 9.7 nesta entrega.

Referências do comportamento, arquitetura e limites: [subset](coreutils-9.7-subset.md) e [arquitetura](architecture.md). O trabalho preserva a política de certificação por evidência; nenhum status foi promovido manualmente.

## Inventory

O discovery encontrou **37 executáveis Coreutils**. Todos possuem implementação identificada, contrato, flags, lacunas e wave. `true`, `false`, `pwd`, `echo` e `printf` mantêm a classificação de seu registro; não foram somados à família apenas porque existem ferramentas GNU homônimas.

| Wave | Área         | Executáveis                                                         |
| ---- | ------------ | ------------------------------------------------------------------- |
| 1    | foundation   | basename, printenv, dirname, whoami                                 |
| 2    | reading      | cat, head, tail, base64, tee, wc, sha256sum                         |
| 3    | filesystem   | readlink, rmdir, ln, mkdir, realpath, stat, touch, ls, chmod, chown |
| 4    | manipulation | cp, mv, rm                                                          |
| 5    | text         | seq, uniq, cut, sort, tr                                            |
| 6    | execution    | id, yes, groups, uname, date, env                                   |
| 7    | advanced     | df, du                                                              |

As áreas definem a sequência semântica; prontidão das dependências, uso em missões, complexidade e evidência existente ordenam os executáveis dentro da fila. A lista de nomes vem do discovery. O carregamento rejeita contratos ausentes, órfãos, baseline divergente ou implementação desatualizada.

## Architecture Changes

- `coreutils/foundation.rs`: caminhos lexicais, ambiente e identidade efetiva.
- `coreutils/bytes.rs`: cat, head, tail, tee, Base64 e SHA-256 em bytes.
- `coreutils/io.rs`: IO por handles VFS, blobs, limites e streams ordenados.
- `coreutils/options.rs` e `legacy.rs`: parsing compartilhado, ajuda do subset e rejeição de opções desconhecidas antes dos handlers antigos.
- `terminal_text.rs`: wc lê bytes; demais ferramentas textuais conservam seus contratos parciais.
- M1A transporta bytes nos canais e redireções. M1B preserva inode, offset, aliases, permissões e conteúdo binário em handles.
- Pacotes registram todos os 37 executáveis; migração atualiza bindings ausentes em saves baseline, sem reinstalar arquivos já removidos de uma definição atualizada.
- M0 recebe contratos Coreutils, comparação hexadecimal, fingerprints por caso, fila por executável e três gates adicionais: `COMMAND_CONTRACTS`, `GNU_REFERENCE`, `KNOWN_GAPS`.

## Reference Harness

O harness DEV `scripts/cli/coreutils-reference.py --capture` exige **Linux, bubblewrap, GNU 9.7 e locale C**. Compara duas execuções independentes por caso, registra hashes/versão dos binários, argv, stdin/stdout/stderr, exit status e snapshots. A baseline é obtida do manifest; outra versão é rejeitada.

Cada caso recebe uma fixture temporária, sem rede, home do desenvolvedor ou repositório montados. `/usr` é somente leitura. Há timeout de 5 segundos, limite de CPU, memória, tamanho de arquivo e saída. Scripts/setup e eventos interativos do jogo são excluídos de execução no host. Atualização de golden exige `--capture`; substituir uma existente exige também `--replace`.

**Validação realizada:** sintaxe Python aprovada e rejeição explícita do ambiente Windows confirmada. **Limite:** o harness não foi executado ponta a ponta em Linux/bubblewrap nesta máquina; seus snapshots de side effects ainda exigem revisão de equivalência. Não há golden GNU produzido nem resultado GNU aprovado. Expectativas documentais em `tests/cli/references/coreutils/9.7` são identificadas como `DECLARED_EXPECTATIONS`.

## Wave 1 — Foundation

`basename`, `dirname`, `printenv`, `whoami`: caminhos lexicais, sufixos, raízes, múltiplos operandos, NUL, variáveis ausentes, ambiente exportado, argumentos e usuário virtual. A execução explícita por `/usr/bin/NAME` exercita o resolver de pacotes. Testes incluem valores de opções iguais a `--help` e interrupção do parsing de printenv no primeiro operando. **Status: PARTIAL**; faltam captura GNU e contratos/flags completos.

## Wave 2 — Reading and bytes

`cat`, `head`, `tail`, `tee`, `wc`, `base64`, `sha256sum`: leitura binária, NUL/UTF-8 inválido, flags de apresentação de cat, limites positivos/negativos e início em offset, cabeçalhos, append, erros parciais, Base64, digest e verificação básica. wc diferencia bytes e caracteres válidos em C.UTF-8. Casos declarativos e testes Rust exercitam pipes, redireção e persistência em inodes compartilhados. **Status: PARTIAL**; limites e gaps estão individualizados abaixo.

## Wave 3 — Filesystem

Contratos e auditoria de `ls`, `stat`, `touch`, `mkdir`, `rmdir`, `ln`, `readlink`, `realpath`, `chmod`, `chown`. As primitives M1B permanecem comuns ao shell e à interface. O fluxo integrado verifica hardlink, symlink, chmod, cópia com novo inode, rename preservando inode, unlink de apenas uma entry e leitura pelo alias restante. **Status: PARTIAL**; não houve reimplementação completa de todas as matrizes GNU.

## Wave 4 — Manipulation

`cp`, `mv`, `rm`: preservados os comportamentos M1B, regressões de cópia/rename/remoção, permissão dos pais e sucesso parcial. Adicionados contratos explícitos e validação compartilhada de opções. **Status: PARTIAL**; opções interativas, backup, traversal completo e demais lacunas seguem abertas.

## Wave 5 — Text

`sort`, `uniq`, `cut`, `tr`, `seq`: inventariados e contratados; parsers legados recebem a validação de flags. Corrigido status 2 para erro de sort. Há regressão e medição de sort, mas não certificação de records binários, collation, campos e todos os modos numéricos. **Status: PARTIAL**.

## Wave 6 — Execution

`env`: ambiente temporário, `-i`, `-u`, atribuições, execução de programa virtual e restauração do contexto. A resolução de filhos exige executável disponível: `env cd` não usa indevidamente o builtin. Fingerprints incluem dependências dos programas filhos. `yes` participa do scheduler cooperativo também por caminho absoluto. `uname`, `id`, `groups`, `date` foram auditados e contratados. **Status: PARTIAL**; `env` declara dependência em `SHELL.JOBS`, ainda parcial.

## Wave 7 — Advanced

`du`, `df`: contratos, gaps e prioridade registrados. du participa da medição com até 8.000 arquivos. **Status: PARTIAL**; números lógicos do simulador não certificam alocação de blocos ou mounts GNU.

## Per-Command Status

| Command   | Before  | After   | Reference cases | Project cases | Gates | Blockers                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| --------- | ------- | ------- | --------------- | ------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| basename  | PARTIAL | PARTIAL | 0               | 12            | 13/19 | GATE:COMMAND_CONTRACTS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PIPE; GATE:REDIRECTION; GAP:Full GNU help/version banners and POSIXLY_CORRECT parsing remain unverified                                                                                                                                                                                                                                                                                                              |
| printenv  | PARTIAL | PARTIAL | 0               | 7             | 13/19 | GATE:COMMAND_CONTRACTS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PIPE; GATE:REDIRECTION; GAP:Environment listing uses deterministic key order; GAP:GNU insertion order is not modeled                                                                                                                                                                                                                                                                                                 |
| dirname   | PARTIAL | PARTIAL | 0               | 6             | 13/19 | GATE:COMMAND_CONTRACTS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PIPE; GATE:REDIRECTION; GAP:Platform-specific double-slash roots excluded by Linux baseline                                                                                                                                                                                                                                                                                                                          |
| whoami    | PARTIAL | PARTIAL | 0               | 5             | 14/19 | GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PIPE; GATE:REDIRECTION; GAP:User database lookup failures are not modeled                                                                                                                                                                                                                                                                                                                                                                    |
| cat       | PARTIAL | PARTIAL | 0               | 13            | 18/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Interactive transformed input buffers to EOF; GAP:self-output detection and GNU quoting need evidence                                                                                                                                                                                                                                    |
| head      | PARTIAL | PARTIAL | 0               | 8             | 14/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PERMISSIONS; GATE:PIPE; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Count multipliers and NUL delimiter unsupported; GAP:bounded byte mode is not incremental                                                                                                                                                                              |
| tail      | PARTIAL | PARTIAL | 0               | 8             | 14/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PERMISSIONS; GATE:PIPE; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Follow/retry requires process/filesystem notifications; GAP:count multipliers and NUL delimiter unsupported                                                                                                                                                            |
| base64    | PARTIAL | PARTIAL | 0               | 7             | 12/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Decode partial-output behavior and invalid-input diagnostics need full GNU audit                                                                                                                                                    |
| tee       | PARTIAL | PARTIAL | 0               | 4             | 12/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Signal ignoring/output-error modes unsupported; GAP:buffers stdin to EOF                                                                                                                                                            |
| wc        | PARTIAL | PARTIAL | 0               | 4             | 10/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GATE:VERSION; GAP:Display width and files0-from unsupported; GAP:malformed UTF-8 word classification needs reference                                                                                                       |
| sha256sum | PARTIAL | PARTIAL | 0               | 6             | 14/26 | GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PERMISSIONS; GATE:PIPE; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Escaped checksum lists, malformed-line warnings, --strict/--warn/--zero and stdin check semantics incomplete                                                                                                                                                           |
| readlink  | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Canonicalization flags, quiet/verbose, no-newline and multi-operand behavior incomplete    |
| rmdir     | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Parents/verbose and multiple-operand partial success not implemented                       |
| ln        | PARTIAL | PARTIAL | 0               | 1             | 6/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDIN; GATE:TTY; GATE:VERSION; GAP:Force, directories, target-directory, backup, verbose and follow policy incomplete                                                   |
| mkdir     | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Symbolic mode, precise intermediate-directory modes and partial operand failures need work |
| realpath  | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Missing-target policies, relative output, strip and NUL forms unsupported                  |
| stat      | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Filesystem mode, printf, complete formats and GNU default layout incomplete                |
| touch     | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Separate atime/mtime, reference/date/timestamp forms unsupported                           |
| ls        | PARTIAL | PARTIAL | 0               | 3             | 11/26 | GATE:COMMAND_CONTRACTS; GATE:ERRORS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STDIN; GATE:TTY; GATE:VERSION; GAP:Column layout, GNU quoting, LS_COLORS, block accounting and many flags incomplete                                                                                                                                                                      |
| chmod     | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Class-copy, implicit umask, symlink traversal and partial errors need audit                |
| chown     | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Numeric IDs, follow policies and partial success incomplete                                |
| cp        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Interactive/preserve/archive/backup/link flags and full dereference matrix incomplete      |
| mv        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Interactive, backup, exchange and cross-mount moves unsupported                            |
| rm        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Interactive, one-filesystem and full preserve-root option semantics unsupported            |
| seq       | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Floating precision, format, equal-width and GNU errors need audit                          |
| uniq      | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Arbitrary bytes, complete locale folding and group separator forms incomplete              |
| cut       | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Byte/character ranges, complement, output delimiter and binary input incomplete            |
| sort      | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Field keys, binary records, merge/external sort and additional numeric modes incomplete    |
| tr        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Classes, ranges, squeeze, complement and binary sets incomplete                            |
| id        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Flags, users, supplementary group membership and GNU formatting incomplete                 |
| yes       | PARTIAL | PARTIAL | 0               | 1             | 6/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDIN; GATE:TTY; GATE:VERSION; GAP:Help/version and dash-leading operands need audit; GAP:interactive terminal output has 4 MiB cap                                     |
| groups    | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Group output is legacy simulation; GAP:real identity membership not fully integrated       |
| uname     | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Values intentionally virtual; GAP:parser, ordering and errors need evidence                |
| date      | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:General format/timezone/parsing and scheduler clock semantics incomplete                   |
| df        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:No real block allocation or mounts; GAP:GNU df cannot be certified from these numbers      |
| du        | PARTIAL | PARTIAL | 0               | 0             | 3/26  | GATE:COMBINED_FLAGS; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:ERRORS; GATE:EXIT_CODE; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:LONG_FLAGS; GATE:PARSER; GATE:PERMISSIONS; GATE:PIPE; GATE:POSITIONAL_ARGS; GATE:REDIRECTION; GATE:SHORT_FLAGS; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDERR; GATE:STDIN; GATE:STDOUT; GATE:TTY; GATE:VERSION; GAP:Hardlink deduplication, block sizes, dereference policies and GNU layout need audit        |
| env       | PARTIAL | PARTIAL | 0               | 8             | 13/26 | SUBSYSTEM:SHELL.JOBS:PARTIAL; GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:PERMISSIONS; GATE:PIPE; GATE:REDIRECTION; GATE:SIDE_EFFECTS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:STDIN; GATE:TTY; GAP:Split-string, chdir, signals, arbitrary environment variable names and executable-only lookup still need audit                                                                                                               |

Gates são apresentados como aprovados/obrigatórios. A tabela e os detalhes completos podem ser regenerados em [Coreutils compatibility](../generated/coreutils-compatibility.md) e [inventário JSON](../generated/coreutils-inventory.json). Casos compartilhados de integração não devem ser somados como execuções independentes por comando.

## VERIFIED

**Nenhum.** Nenhum executável possui capturas GNU 9.7 e todos ainda têm lacunas conhecidas ou evidência obrigatória incompleta. A família também permanece PARTIAL.

## PARTIAL

**37 executáveis.** Há implementação real; falta certificação dos contratos completos. Blockers específicos estão em “Remaining Coreutils Work” e os gates ausentes no relatório gerado. Os três gates novos impedem usar um caso feliz, uma versão impressa ou uma flag apenas declarada como prova suficiente.

## UNVERIFIED

**Zero dentro de Coreutils.** Isso não muda o status dos comandos de outras famílias nem dos programas existentes apenas no catálogo.

## Cross-System Fixes

1. Pipes e redireções deixavam conteúdo inválido depender de conversões textuais; agora carregam bytes e preservam sua ordem.
2. Append de blobs e escritas que dividem um caractere UTF-8 passam pelos handles VFS; aliases e arquivos abertos após unlink preservam identidade.
3. O scheduler reconhece `yes`/`head` por caminho absoluto e encerra o produtor quando o consumidor fecha.
4. Bindings de executáveis ausentes nos saves baseline recebem migração delimitada pela definição do pacote.
5. A ajuda de `man` prioriza o subset atual e evita documentar handlers antigos removidos.
6. `env` restaura o contexto e não executa um builtin como se fosse um programa externo.
7. O adaptador textual de handles rejeita escritas que exigiriam blob sem cache persistente, antes de alterar o inode; escritas vazias não estendem arquivos nem atualizam seu timestamp.

## Binary Safety

cat/head/tail/tee/Base64/SHA-256 operam sobre bytes. wc usa bytes para tamanho/newlines e decodificação explícita para caracteres; classificação de palavras com UTF-8 malformado ainda precisa de referência. Pipes, redireção, append e snapshots usam dados originais. A apresentação visual pode converter UTF-8 inválido para caracteres de substituição; isso não modifica os bytes encaminhados ao próximo comando ou arquivo.

Sort/uniq/cut/tr ainda não possuem semântica binária completa. Handlers legados que só aceitam texto rejeitam stdin inválido. Conteúdo de arquivo, locale, opções e side effects fora desse subset continuam sujeitos aos gaps declarados.

## Filesystem Fidelity

IO usa VFS M1B e identidades virtuais, sem filesystem host. Escritas por handle respeitam permissões, offset, append, capacidade e aliases; mantêm metadata de inode, atualizam timestamps virtuais e aplicam regras de bits especiais. Blobs imutáveis permitem conteúdo não textual. O schema de save permanece VFS v2.

`cp` cria inode distinto, `mv` preserva inode, `rm` remove a entry e hardlinks restantes continuam legíveis. Symlinks conservam target textual e podem ficar dangling. As matrizes GNU de traversal, flags, diagnósticos e erros parciais não estão completas apenas por usar as primitives M1B.

## Streaming

Canais cooperativos possuem capacidade de **64 KiB** e blocos de **4 KiB**, com backpressure, EOF e fechamento do consumidor. `yes | head` tem produção limitada pelo consumo. Não há criação de processos host.

Adapters finitos ainda acumulam até **4 MiB**; tail, tee, Base64/hash e alguns modos de cat/head não são integralmente incrementais. Arquivos binários têm limite próprio de **32 MiB**. A saída também possui teto. Há cópias em buffers de bytes e apresentação; o teto não equivale ao pico total de memória do processo. Tail follow/retry continua fora do subset.

## Performance

| Comando                         | Fixture                                    | Tempo (ms) |
| ------------------------------- | ------------------------------------------ | ---------- |
| du -s bench                     | 1 arquivos                                 | 12.118     |
| cp -r bench copied              | 1 arquivos                                 | 5.944      |
| rm -r bench                     | 1 arquivos                                 | 6.119      |
| du -s bench                     | 100 arquivos                               | 5.768      |
| cp -r bench copied              | 100 arquivos                               | 53.953     |
| rm -r bench                     | 100 arquivos                               | 27.019     |
| du -s bench                     | 1000 arquivos                              | 14.196     |
| cp -r bench copied              | 1000 arquivos                              | 412.983    |
| rm -r bench                     | 1000 arquivos                              | 212.110    |
| du -s bench                     | 8000 arquivos                              | 64.413     |
| rm -r bench                     | 8000 arquivos                              | 1699.384   |
| cat large \| wc -c              | 1000000 bytes                              | 161.511    |
| sort large > sorted             | 1000000 bytes                              | 73.620     |
| sha256sum large                 | 1000000 bytes                              | 138.028    |
| yes \| head -n 100000 > bounded | produtor yes; consumidor de 100.000 linhas | 15.883     |

Medições DEV em debug, uma amostra por operação, sem comparação com o GNU nativo. Não constituem perfil estatístico nem medição de pico de memória. A criação da fixture fica fora do tempo de cada comando. Cópia recursiva foi medida até 1.000 arquivos para manter espaço para origem e destino; du/rm chegam a 8.000. O benchmark corrigiu sua fixture para exatamente 1.000.000 bytes.

O VFS usa índices e payloads compartilhados, mas adaptação de output, snapshots e atualizações de blobs ainda têm custo de cópia. Escritas repetidas em payload imutável podem exigir reconstrução do conteúdo; otimização incremental e perfis de memória permanecem trabalho futuro.

## Security

Os handlers de gameplay usam exclusivamente World/VFS/processos virtuais. Testes cobrem caminhos Windows/UNC, argumentos inválidos, fuzz de parsers e integridade do VFS. O guard de host e o teste de bundling verificam que execução de referência não entra no runtime. Capturas/benchmarks Rust são `cfg(test)` e ignorados na execução normal até acionamento DEV explícito. O guard não é uma prova formal de sandbox.

## Regression

**190 testes Rust aprovados**, zero falhas; 7 testes DEV ignorados na execução normal. O pipeline executou explicitamente export/capture e benchmarks VFS, shell e Coreutils. **214 testes frontend em 36 arquivos**, **28 testes de tooling**, **76 casos legados exatos** e **186 casos declarativos (104 existentes + 82 novos)** aprovados.

Regressões de pacotes, archives, saves/checkpoints, cinco slots, permissões, missões, terminal e VFS passaram. Permanecem READY as 7 capacidades do shell 1A e as 11 capacidades VFS 1B. O conteúdo valida 12 missões, 11 threads, 8 hosts e 303 entradas do catálogo.

## Generated Pipeline Results

| Métrica                                    | Resultado                   |
| ------------------------------------------ | --------------------------- |
| Nomes de comandos                          | 1797                        |
| Executáveis únicos                         | 1526                        |
| VERIFIED / PARTIAL / UNVERIFIED (comandos) | 0 / 120 / 1406              |
| CATALOG_ONLY                               | 1406                        |
| Famílias                                   | 279                         |
| VERIFIED / PARTIAL / UNVERIFIED (famílias) | 0 / 42 / 237                |
| Coreutils: VERIFIED / PARTIAL / UNVERIFIED | 0 / 37 / 0                  |
| Capturas GNU 9.7 aprovadas                 | 0                           |
| Casos declarativos                         | 186 PASS; 0 FAIL; 0 SKIPPED |
| Gates PASS                                 | 3859                        |
| Gates obrigatórios PASS / total            | 3310 / 35174                |

## Quality Gates

| Gate                                         | Resultado                                                    |
| -------------------------------------------- | ------------------------------------------------------------ |
| Rust fmt                                     | PASS                                                         |
| Clippy all targets/all features, -D warnings | PASS                                                         |
| Rust tests / main / doc-tests                | 190 PASS; zero falhas                                        |
| TypeScript                                   | PASS                                                         |
| ESLint / Stylelint                           | PASS                                                         |
| Conteúdo                                     | PASS                                                         |
| Frontend                                     | 214 PASS / 36 arquivos                                       |
| Tooling CLI                                  | 28 PASS                                                      |
| Compatibilidade declarativa                  | 186/186 PASS                                                 |
| Compatibilidade legada exata                 | 76/76 PASS                                                   |
| Benchmarks shell/VFS/Coreutils               | PASS                                                         |
| cli:inventory / cli:verify                   | PASS; zero erros                                             |
| cli:verify --strict                          | FAIL esperado: 1520 comandos reais sem certificação completa |
| Host guard / boundary de bundling            | PASS                                                         |
| Prettier / git diff --check                  | PASS                                                         |
| Build web                                    | PASS                                                         |
| Build Windows release, tauri/custom-protocol | PASS                                                         |
| Harness GNU: sintaxe / bloqueio Windows      | PASS                                                         |
| Captura real GNU 9.7                         | NÃO EXECUTADA: ambiente indisponível                         |

Executável: `src-tauri/target/release/game-hacker.exe`, **89,470,976 bytes**, SHA-256 `85ceab32b72a6c8b85dfa08e39db90b3b8335209f2c2743b6ed000755b7a0996`. Build otimizado com `tauri/custom-protocol`; não foi gerado instalador nem realizada inspeção manual do executável Windows. O build web mantém o aviso de chunk principal acima de 500 kB. Strict continua bloqueando a dívida real e não foi enfraquecido.

Logs de execução: `artifacts/m1c-*.log`. Capturas e hashes ficam em `artifacts/cli-evidence.json`, `cli-case-actual.json`, `cli-verification.json` e `coreutils-performance.json`. São artefatos locais regeneráveis. Os relatórios em docs conservam o resultado legível.

## Remaining Coreutils Work

Todas as ferramentas precisam de captura GNU 9.7 reproduzível e evidência para cada contrato/flag obrigatório. Além desse bloqueio comum:

| Comando   | Lacunas específicas                                                                                            |
| --------- | -------------------------------------------------------------------------------------------------------------- |
| basename  | Full GNU help/version banners and POSIXLY_CORRECT parsing remain unverified                                    |
| printenv  | Environment listing uses deterministic key order; GNU insertion order is not modeled                           |
| dirname   | Platform-specific double-slash roots excluded by Linux baseline                                                |
| whoami    | User database lookup failures are not modeled                                                                  |
| cat       | Interactive transformed input buffers to EOF; self-output detection and GNU quoting need evidence              |
| head      | Count multipliers and NUL delimiter unsupported; bounded byte mode is not incremental                          |
| tail      | Follow/retry requires process/filesystem notifications; count multipliers and NUL delimiter unsupported        |
| base64    | Decode partial-output behavior and invalid-input diagnostics need full GNU audit                               |
| tee       | Signal ignoring/output-error modes unsupported; buffers stdin to EOF                                           |
| wc        | Display width and files0-from unsupported; malformed UTF-8 word classification needs reference                 |
| sha256sum | Escaped checksum lists, malformed-line warnings, --strict/--warn/--zero and stdin check semantics incomplete   |
| readlink  | Canonicalization flags, quiet/verbose, no-newline and multi-operand behavior incomplete                        |
| rmdir     | Parents/verbose and multiple-operand partial success not implemented                                           |
| ln        | Force, directories, target-directory, backup, verbose and follow policy incomplete                             |
| mkdir     | Symbolic mode, precise intermediate-directory modes and partial operand failures need work                     |
| realpath  | Missing-target policies, relative output, strip and NUL forms unsupported                                      |
| stat      | Filesystem mode, printf, complete formats and GNU default layout incomplete                                    |
| touch     | Separate atime/mtime, reference/date/timestamp forms unsupported                                               |
| ls        | Column layout, GNU quoting, LS_COLORS, block accounting and many flags incomplete                              |
| chmod     | Class-copy, implicit umask, symlink traversal and partial errors need audit                                    |
| chown     | Numeric IDs, follow policies and partial success incomplete                                                    |
| cp        | Interactive/preserve/archive/backup/link flags and full dereference matrix incomplete                          |
| mv        | Interactive, backup, exchange and cross-mount moves unsupported                                                |
| rm        | Interactive, one-filesystem and full preserve-root option semantics unsupported                                |
| seq       | Floating precision, format, equal-width and GNU errors need audit                                              |
| uniq      | Arbitrary bytes, complete locale folding and group separator forms incomplete                                  |
| cut       | Byte/character ranges, complement, output delimiter and binary input incomplete                                |
| sort      | Field keys, binary records, merge/external sort and additional numeric modes incomplete                        |
| tr        | Classes, ranges, squeeze, complement and binary sets incomplete                                                |
| id        | Flags, users, supplementary group membership and GNU formatting incomplete                                     |
| yes       | Help/version and dash-leading operands need audit; interactive terminal output has 4 MiB cap                   |
| groups    | Group output is legacy simulation; real identity membership not fully integrated                               |
| uname     | Values intentionally virtual; parser, ordering and errors need evidence                                        |
| date      | General format/timezone/parsing and scheduler clock semantics incomplete                                       |
| df        | No real block allocation or mounts; GNU df cannot be certified from these numbers                              |
| du        | Hardlink deduplication, block sizes, dereference policies and GNU layout need audit                            |
| env       | Split-string, chdir, signals, arbitrary environment variable names and executable-only lookup still need audit |

Não foram escondidas opções ausentes sob status VERIFIED. O próximo ciclo deve ampliar os contratos verticalmente, começando pela fila de executáveis, capturar GNU e tratar diferenças observadas antes de promover.

## Dependency Graph

| Nó                 | Estado derivado                   | Consequência                                                               |
| ------------------ | --------------------------------- | -------------------------------------------------------------------------- |
| Coreutils          | NEEDS_SUBSYSTEM                   | SUBSYSTEM:SHELL.JOBS:PARTIAL                                               |
| Shell              | PARTIAL; 7 capacidades READY      | SHELL.JOBS segue PARTIAL                                                   |
| VFS                | PARTIAL; 11 capacidades READY     | VFS.DEVICES segue PARTIAL                                                  |
| coreutils/basename | NEEDS_IMPLEMENTATION_AND_EVIDENCE | Sem bloqueio de subsystem; contratos/flags, lacunas e referência pendentes |

A dependência de jobs pertence a env e afeta o agregado familiar; não impede o avanço de comandos independentes. Filas completas: [famílias](../generated/cli-next-work.md) e [executáveis Coreutils](../generated/coreutils-compatibility.md#executable-queue).

## Next Recommended Milestone

A fila por executável coloca **coreutils/basename** primeiro, com `NEEDS_IMPLEMENTATION_AND_EVIDENCE`. O próximo ciclo recomendado é fechar seus contratos e flags, preparar a captura Linux/GNU 9.7 e resolver as diferenças observadas; seguir então a ordem recalculada. Na fila familiar, Coreutils permanece primeiro e exige completar **SHELL.JOBS**, introduzido como dependência explícita de env. A recomendação vem das filas geradas; não promove nem inicia automaticamente outra família.
