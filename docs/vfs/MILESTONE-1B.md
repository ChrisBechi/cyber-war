# Milestone 1B — VFS / POSIX Filesystem Fidelity

Base: `35d9418` (Milestone 1A). Implementação de um subset virtual de filesystem POSIX/Linux. Resultados abaixo obtidos pelos scripts reais do projeto; sem promover compatibilidade manualmente.

## Architecture Changes

O monólito de VFS foi separado em `model`, `resolve`, `operations`, `handles`, `persistence` e `errors`. Inodes e entries agora são entidades distintas. Terminal, editor, File Manager, pacotes, archives e missões foram adaptados; a projeção de paths mantém o contrato da interface. O schema TypeScript aceita o wire novo e previews legados.

Detalhes: [arquitetura](architecture.md), [subset](posix-linux-subset.md), [migração](save-migration.md).

## Inode Model

IDs u64 monotônicos por filesystem, tabela canônica de `Arc<Inode>`, conteúdo compartilhado, nlink derivado de entries. Copy-on-write conserva a independência dos snapshots transacionais. Inode sem entry permanece vivo enquanto um handle o referencia. Conteúdo é contabilizado uma vez, inclusive após unlink com handle aberto.

## Directory Model

Cada diretório mantém nome → inode; índice reverso localiza paths e projeções. Diretórios têm pai único, nlink 2 + subdiretórios e não admitem hardlinks arbitrários. Rename de árvore reconstrói o índice de paths preservando IDs.

## Path Resolution

Resolver iterativo central: caminhos absolutos e relativos, `.`, `..`, barras repetidas e barra final. Normalização não apaga `..` antes de symlinks. Search é verificado em cada ancestral. 40 expansões de symlink, 4096 bytes por path e 255 por componente. Unicode case-sensitive; sem interpretação de caminhos Windows ou host.

## Symbolic Links

Links relativos e absolutos, targets literais, dangling e ciclos. Primitives stat/lstat/readlink distinguem target e link. Escrita com create através de dangling cria seu target. CLI `stat` usa lstat por padrão e `-L` para seguir. Globbing, PATH e scripts utilizam o resolver do VFS.

## Hard Links

`ln` cria outra entry apontando ao mesmo inode. Escrita/chmod refletem nos aliases; unlink remove somente uma entry. Rename conserva inode; copy cria inode novo. Save/load conserva IDs e compartilhamento sem repetir o payload. A lixeira registra origem por entry, inclusive quando dois hardlinks são removidos e restaurados separadamente.

## Permissions

Uid/gid virtuais, grupos suplementares, owner/group/other, read versus search em diretórios, root execute para arquivos regulares, sticky e umask. Diretórios setgid transmitem grupo. Modos especiais são armazenados e exibidos; setuid/setgid executável não concede privilégios. Projeções do banco de pacotes continuam protegidas.

## Metadata

Ino, tipo, mode, uid/gid, nomes, nlink, tamanho lógico, BlobRef, timestamps e metadata são canônicos. `stat -c` e `ls -l` expõem o subset pertinente. Tamanho lógico não equivale a blocos físicos de disco.

## File Operations

Open/read/write/create/exclusive/append/truncate, seek e close virtuais; offsets por abertura, append no final atual e inode aberto após unlink. Mutações validam antes de publicar. Rename/unlink/mkdir/rmdir/touch são primitives centrais. UI usa transações; comandos com vários operandos mantêm os sucessos parciais já documentados. Handles de texto exigem UTF-8; blobs continuam na API binária.

## Virtual Devices

`/dev/null` é um node do VFS e funciona por alias; `/dev/zero` fornece bytes em reads limitados por handle. Leitura infinita de zero, random e tty não são emulados. A capacidade agregada de dispositivos permanece PARTIAL.

## Save Migration

VFS formatVersion 2 com entries/inodes/nextInode, persistência de identidades e metadados. Nodes legados recebem inodes determinísticos e independentes; symlink targets, conteúdo, mode e BlobRef são preservados. Atime/ctime ausentes derivam de modifiedAt. Migração não deduz hardlinks por conteúdo igual. Validação rejeita wire inconsistente e preserva o save anterior.

## Cross-System Integration

- Shell: resolução central, handles nas redireções, umask de terminal e reparação de cwd por inode.
- File Manager: projeção do wire, links distinguíveis, abertura via backend e mesmas permissões. Sem redesenho.
- Packages: bindings de `ln`/`rmdir`, migração idempotente, projeções e recolhimento de inodes após operações privilegiadas.
- Archives: coleta com lstat, preservação de symlinks e política explícita anti-traversal. TAR hardlink segue fora do subset; ZIP não é tratado como POSIX completo.
- Missions: restore por API do VFS, compartilhamento preservado nos before-images, limpeza de órfãos no descarte e validação transacional.

## Compatibility Evidence

36 casos declarativos novos em `tests/cli/compat/pilot/vfs-foundation.json`, ligados ao pipeline M0 e a manifest de expectativas declaradas. Incluem relações entre inodes, nlink, conteúdo, chmod, caminhos, dangling/loops, null, roundtrip, isolamento, glob e execução por symlink. Nenhuma captura de GNU/Linux de referência foi inventada.

Testes Rust adicionais exercitam 2.000 operações pseudoaleatórias com modelo independente, 1.500 caminhos fuzz, matriz owner/group/other, offsets, limites, zero, capacidade após unlink, corrupção, migração, journals e cinco slots. Casos do shell 1A permanecem no pipeline. Dois pressupostos antigos foram atualizados explicitamente: nomes Windows são virtuais válidos e `/dev/null` é um dispositivo persistente.

## Performance

Medições em build debug, médias de parede; o orçamento de 10.000 entries inclui o filesystem inicial.

| Medida                | 1.000 arquivos | 9.955 arquivos / 10.000 entries |
| --------------------- | -------------- | ------------------------------- |
| Criação total (ms)    | 159.463        | 1509.708                        |
| Enumeração (ms)       | 0.239          | 2.298                           |
| Stat (ms)             | 0.020          | 0.021                           |
| Save (ms)             | 25.997         | 246.973                         |
| Load + validação (ms) | 101.843        | 1013.274                        |

Caminho com profundidade 120: **0.478 ms**. Cadeia de 40 symlinks: **0.257 ms**. Globbing de 9.955 entries: **143.404 ms**. Snapshot maior: **2,434,847 bytes**.

Capturas: `artifacts/vfs-performance.json` e `artifacts/shell-performance.json`; cópia gerada no relatório de compatibilidade. Não são benchmarks de release.

## Security

Guard de produção verifica ausência de APIs de filesystem/processos/rede do host no runtime dos comandos. Casos exercitam nomes Windows e comandos de host inexistentes. Extração e instalação mantêm suas próprias fronteiras de caminho. As ferramentas DEV de capture/benchmark são compiler-gated e não entram no binário do jogo. O guard é evidência de fronteira de código/build, não prova formal de sandbox do sistema operacional.

## Regression

**184 testes Rust aprovados**, sem falhas; 6 testes DEV ignorados na execução normal, com export/capture e ambos os benchmarks executados explicitamente pelo pipeline. **214 testes frontend em 36 arquivos** e **25 testes da infraestrutura CLI** aprovados. Os **76 casos legados exatos** passaram, além de **104 casos declarativos (68 existentes + 36 novos)**. Os 7 grupos READY do shell 1A continuam READY.

Regressões de pacotes, archives, terminal, editor, navegador, save/checkpoints, missões, lixeira e slots passaram. A validação de conteúdo aprovou 12 missões, 11 threads, 8 hosts e 303 entradas do catálogo.

## VFS Readiness

Antes: VFS PARTIAL sem separação suficiente para coreutils. Depois: **11 capacidades READY e VFS.DEVICES PARTIAL**; agregado **PARTIAL**. READY descreve o subset documentado e depende da evidência atual, incluindo os testes Rust executados antes de publicar o fingerprint. [Relatório gerado](../generated/vfs-compatibility.md).

## Coreutils Blockers

Antes: `SUBSYSTEM:VFS:PARTIAL`. Depois: **nenhum bloqueio de subsystem**, estado **READY_TO_IMPLEMENT** calculado pelo grafo. Dependências explícitas de paths, arquivos, diretórios, inodes, symlinks, hardlinks, permissões, bits especiais, metadata e timestamps. Não há promoção manual a VERIFIED.

## Generated Pipeline Results

| Métrica                                    | Resultado        |
| ------------------------------------------ | ---------------- |
| Nomes de comandos                          | 1797             |
| Executáveis únicos                         | 1526             |
| VERIFIED (comandos)                        | 0                |
| PARTIAL (comandos)                         | 120              |
| UNVERIFIED (comandos)                      | 1406             |
| CATALOG_ONLY                               | 1406             |
| Famílias                                   | 279              |
| VERIFIED / PARTIAL / UNVERIFIED (famílias) | 0 / 42 / 237     |
| Casos declarativos                         | 104 PASS; 0 FAIL |
| Gates aprovados                            | 3748             |

Números gerados por `cli:inventory`, `cli:compat` e `cli:verify`. Dashboard, fila, inventário, diff e relatórios foram regenerados. Aumento de três comandos nativos: `ln`, `rmdir` e `umask`. Não houve promoção a VERIFIED.

## Quality Gates

| Gate                                              | Resultado                                                    |
| ------------------------------------------------- | ------------------------------------------------------------ |
| Rust fmt                                          | PASS                                                         |
| Clippy, all targets/features, warnings como erros | PASS                                                         |
| Rust tests / main / doc-tests                     | 184 PASS; 0 falhas                                           |
| TypeScript                                        | PASS                                                         |
| ESLint / Stylelint                                | PASS                                                         |
| Conteúdo                                          | PASS                                                         |
| Frontend                                          | 214 PASS / 36 arquivos                                       |
| Infraestrutura CLI                                | 25 PASS                                                      |
| Casos declarativos                                | 104/104 PASS                                                 |
| Casos legados exatos                              | 76/76 PASS                                                   |
| cli:verify normal                                 | PASS, 0 erros                                                |
| cli:verify strict                                 | FAIL esperado: 1520 comandos reais sem certificação completa |
| Build web                                         | PASS                                                         |
| Build Windows release                             | PASS, otimizado em 7 min 32 s                                |
| Prettier                                          | PASS, incluindo revisão dos arquivos finais                  |
| git diff --check                                  | PASS                                                         |

Strict continua exigindo certificação completa e não foi enfraquecido. O build web mantém o aviso existente de chunk principal acima de 500 kB. A validação é automatizada; não foi realizada inspeção manual do executável Windows.

Logs: `artifacts/m1b-*.log`. Evidência strict preservada em `artifacts/m1b-strict-result.json`.

Executável Windows: `src-tauri/target/release/game-hacker.exe`, **89.101.824 bytes**, compilado em 16/09/2026. SHA-256: `055e1503c4d8f769f1c3de834b422e6b9b933467b1ab782a312b87ffe12ef838`. O build utiliza `tauri/custom-protocol`; não foi gerado um instalador.

## Next Recommended Milestone

A fila calculada indica **GNU Coreutils 9.7 Compatibility**, `READY_TO_IMPLEMENT`, sem bloqueios. A auditoria vertical de flags, stdout/stderr, statuses e side effects é o próximo trabalho. Não foi iniciada uma certificação completa dentro deste milestone.
