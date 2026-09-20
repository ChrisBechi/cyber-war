# M1C.9 — Filesystem / Path GNU 9.7

**STOP_REASON: `MILESTONE_COMPLETE`. PUBLICATION_STATUS: `PENDING`.**

| Comando  | Status final | GNU matches | Contratos PASS | Gates obrigatórios PASS | Gaps |
| -------- | ------------ | ----------- | -------------- | ----------------------- | ---- |
| readlink | VERIFIED     | 309         | 12/12          | 23/23                   | 0    |
| realpath | VERIFIED     | 422         | 14/14          | 23/23                   | 0    |
| mkdir    | VERIFIED     | 387         | 16/16          | 23/23                   | 0    |
| rmdir    | VERIFIED     | 341         | 14/14          | 23/23                   | 0    |
| ln       | VERIFIED     | 591         | 17/17          | 26/26                   | 0    |

| Medida             | Antes                    | Depois                       |
| ------------------ | ------------------------ | ---------------------------- |
| Wave               | 5 PARTIAL                | 5 VERIFIED                   |
| Coreutils          | 11 VERIFIED / 26 PARTIAL | 16 VERIFIED / 21 PARTIAL     |
| GNU global         | 2226 matches             | 4276 matches                 |
| Declarativa global | 2332 PASS                | 4381 PASS; 0 FAIL; 0 SKIPPED |

Os valores são derivados do tooling; os cinco checkpoints registram o fechamento individual na ordem readlink → realpath → mkdir → rmdir → ln. O [estado final](../coreutils/evidence/m1c9-final-state.json) contém hashes, dependências, contratos e validações; o [baseline](../coreutils/evidence/m1c9-before.json) é `e5e7937`, posterior à implementação M1C.8 `2e1562e`.

**Implementação compartilhada.** A resolução mantém `..`, pontos e barras finais até a operação interpretar sua semântica. Canonicalização distingue existência obrigatória, último componente ausente e caminhos inteiramente descritivos; não impõe o limite de 40 links do lookup comum a cadeias canônicas acíclicas. Readlink cru devolve o texto armazenado. Realpath aplica modos físico/lógico/lexical, relative-to e relative-base.

Mkdir aplica modes simbólicos/octais, umask e herança setgid no VFS, preservando diretórios já criados quando outro operand falha. Rmdir respeita o último symlink, `-p`, ignore-nonempty e a grafia dos ancestrais nos diagnósticos. Ln usa identidade real de inode, substituição atômica, links simbólicos relativos e backups virtuais; confirmação usa o scheduler, pipes, descritores e VirtualTty existentes. Propriedades exercitam escrita por alias, remoção de um nome, persistência de symlinks e integração entre os cinco comandos.

| GNU observado                                                                 | Cyber War antes / causa                                                         | Correção e evidência                                                                           |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Canonicalização varia com -e/-f/-m/-s, componentes inexistentes e links       | Lookup único confundia texto, existência e resolução                            | Modos compartilhados; matrizes individuais de readlink/realpath e propriedades de idempotência |
| Mkdir -p conserva efeitos parciais; modos especiais dependem de umask/herança | Handler simplificado e aplicação genérica de modo                               | Parser de modos compartilhado e criação por componente; 387 casos                              |
| Rmdir não segue o último symlink; barras e ancestrais mudam erros             | Remoção genérica não distinguia estas situações                                 | Operação VFS específica; 341 casos, inclusive permissões e diretórios não vazios               |
| Ln distingue destino diretório/symlink, -n/-T/-t, ordem de opções e backup    | Handler sem a matriz GNU; validação acontecia em outra ordem                    | Estado por operand e parser com validação ordenada; 591 casos                                  |
| Backup numérico aceita versões decimais longas e ignora zeros iniciais        | Conversão inteira limitava versões; nomes no limite não eram truncados como GNU | Incremento decimal sem limite inteiro e fallback de nome; casos de 254/255 bytes               |
| Diretório pode abrir como stdin; a leitura falha após o prompt de ln -i       | VFS rejeitava a abertura antecipadamente                                        | Descritor somente leitura e erro EISDIR na leitura; regressão GNU e teste de limpeza           |

A regressão também identificou que tail dependia da antiga rejeição de diretórios durante open. O leitor de operands foi ajustado para preservar seus diagnósticos, headers, continuação e limpeza; os 306 goldens de tail foram reexecutados sem alterar expectativas.

**Escopo e limites.** Locale C; nomes UTF-8; caminho virtual até 4096 bytes, componentes armazenados até 255 e texto de symlink até 4095. Truncamento de backup preserva fronteiras UTF-8. Traversal canônico tem orçamento de 10000 componentes e 64 KiB de expansão pendente. Ciclos crescentes excederam os recursos do GNU sandbox; o runtime mantém execução limitada. A montagem de isolamento GNU torna a raiz /home/kali ocupada, enquanto no VFS ela é um diretório comum: essa diferença de ambiente não foi convertida em expectativa falsa. Observações e limites estão no [registro do oracle](../coreutils/evidence/m1c9-oracle-boundaries.json) e nas intentionalDeviations de cada comando.

**Evidência.** Expected vem exclusivamente de execução GNU 9.7 locked, com captura dupla e verify independente. Os onze goldens anteriores (2226 casos) estão byte a byte preservados; mudanças no harness foram revalidadas com recibos separados. Os 581 casos iniciais de ln permaneceram idênticos ao ampliar o corpus para 591. Comparações incluem stdout/stderr, status, estado, relações de inode, link count e mutações inesperadas. Fingerprints distinguem handlers específicos de alterações compartilhadas; o modo focused não declara evidência global e inclui os provedores GNU necessários a TTY/sinais. Nenhum comando fora da wave foi implementado.

| Validação final                                                                | Resultado                                                               |
| ------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| Rust completo                                                                  | 298 PASS; 14 opcionais ignorados                                        |
| Frontend                                                                       | 243 PASS em 40 arquivos                                                 |
| Tooling / legacy                                                               | 63 / 76 PASS                                                            |
| Declarativa global                                                             | 4381 PASS                                                               |
| Normal e strict dos 16 certificados                                            | PASS                                                                    |
| Strict global                                                                  | FAIL esperado: 1504 entradas de dívida conhecida; zero erros adicionais |
| rustfmt, Clippy -D warnings, Prettier, ESLint, Stylelint, TypeScript, conteúdo | PASS                                                                    |
| Web, Windows release, NSIS/MSI e Host Guard                                    | PASS                                                                    |

Build 0.4.2; hashes do executável e instaladores no estado final. O pacote contém somente o recurso de licenças e nenhum externalBin. GNU, WSL e capturadores continuam exclusivos de DEV/CI.

- NEXT QUEUE ITEM: `coreutils/stat`.
- CLASSIFICATION: `SMALL_SHARED_EXTENSION`.
- COMPLEXITY: 2/5.
- ACTION: `NEEDS_IMPLEMENTATION_AND_EVIDENCE`.
- KNOWN GAPS: Filesystem mode, printf, complete formats and GNU default layout incomplete.
- DEPENDENCIES: SHELL.CONTEXT, SHELL.EXPANSION, SHELL.GLOBBING, SHELL.LISTS, SHELL.PARSING, SHELL.PIPELINES, SHELL.REDIRECTION, VFS.DIRECTORIES, VFS.HARDLINKS, VFS.INODES, VFS.METADATA, VFS.PATHS, VFS.PERMISSIONS, VFS.REGULAR_FILES, VFS.SPECIAL_PERMISSIONS, VFS.SYMLINKS, VFS.TIMESTAMPS.
- WAVE_READINESS: **READY**.
- PROPOSED_WAVE: `stat`, `touch`, `chmod`, `chown`.

A proposta agrupa metadados, timestamps, modos e ownership no mesmo modelo de inodes/permissões. As dependências declaradas estão prontas e a complexidade é até 3/5; discovery ainda precisa delimitar timestamps virtuais/statfs e traversal recursivo. READY indica preparação para essa auditoria, sem certificação antecipada. A próxima wave não foi implementada.
