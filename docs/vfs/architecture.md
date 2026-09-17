# Arquitetura VFS — Milestone 1B

## Estado compartilhado

`VirtualFileSystem` é a autoridade de arquivos de cada host virtual. Terminal, File Manager, editor, navegador, downloads, pacotes, archives e missões usam essa instância. Hosts remotos têm tabelas independentes. Nenhuma operação de jogador usa o filesystem, processos ou rede do computador real.

`vfs/model.rs` mantém:

- `Inode`: identidade monotônica, tipo, conteúdo UTF-8 ou BlobRef, uid/gid, nomes de owner/group, mode, nlink, timestamps e metadata.
- `NodeTable.inodes`: um `Arc<Inode>` canônico por identidade; múltiplas entradas compartilham o mesmo payload.
- `directories`: inode do diretório → nome → inode. `links` mantém o índice reverso.
- `views`: projeção ordenada caminho → node, compatível com consumidores existentes. Metadados dessa projeção não constituem uma segunda autoridade: o guard de edição atualiza o inode canônico e todas as entradas.
- Contadores de subdiretórios e bytes lógicos evitam varredura de todos os arquivos em cada criação. Hardlinks contam conteúdo uma vez. Um inode removido, ainda aberto, continua ocupando capacidade até o último close.

Snapshots de transação usam copy-on-write de `Arc`; a tabela de caminhos ainda é clonada. Renomear uma árvore atualiza seu índice de caminhos, preservando os inodes. Não é uma implementação de blocos ext4.

## Resolução

`normalize` somente torna o caminho absoluto; preserva `.` / `..` / barras repetidas / barra final. `resolve.rs` percorre componentes, verifica search/execute no diretório corrente e expande symlinks iterativamente, até 40 expansões. `..` é aplicado depois de seguir os links anteriores. Links relativos começam no diretório que contém o link; absolutos começam na raiz virtual.

`stat` segue o último symlink; `lstat` não segue; `readlink` devolve o target literal. Ancestrais sempre são resolvidos. Barra final exige diretório. Target ausente pode ser criado por open/write com create; um symlink dangling existe para lstat e O_EXCL.

Nomes são Unicode UTF-8, case-sensitive, sem normalização Unicode. NUL é rejeitado; cada componente tem até 255 bytes, caminho até 4096. `:`, `\\`, `CON`, `NUL` e textos parecidos com caminhos Windows são nomes virtuais. `/` separa componentes. A raiz não permite escape por `..`.

## Permissões e identidade

Identidades são virtuais e persistentes; owner/group usam uid/gid. Grupos suplementares participam da decisão. `r`, `w` e `x` são verificados independentemente. Enumerar nomes exige read; resolver filhos exige search. O root virtual ignora read/write/search, mas arquivo regular precisa de ao menos um bit execute para ser executado.

Criação aplica umask (022 por padrão; runtime de cada terminal). Diretórios setgid transmitem grupo e bit setgid a subdiretórios. Sticky controla unlink/rename; `/tmp` novo é 1777. chmod aceita até 07777, e mudanças de owner/group e escrita não privilegiada limpam bits de privilégio pertinentes. Nenhum setuid/setgid executável eleva a identidade do jogador.

## Operações e ciclo de vida

`operations.rs` implementa unlink/rmdir, rename, copy, touch e verificação de invariantes. Rename mantém identidade; copy cria identidade nova; hardlink cria somente uma entrada para o inode. Diretórios não recebem hardlinks arbitrários. Operações compostas de UI usam candidato transacional. Comandos com múltiplos operandos preservam o contrato anterior de sucessos parciais.

`handles.rs` abre descritores virtuais com read/write/create/exclusive/truncate/append; cada abertura possui offset independente. Writes append usam o final atual do inode. Unlink não invalida um descritor já aberto. Os descritores não sobrevivem a save/load.

`Errno` fornece erros tipados, incluindo ENOENT/EEXIST/ENOTDIR/EISDIR/EACCES/EPERM/ENOTEMPTY/ELOOP/EINVAL/ENAMETOOLONG/EROFS/ENOSPC/EBADF/EXDEV. Mensagens adicionais do domínio continuam explícitas para limites do subset.

## Timestamps

Relógio lógico monotônico, sem dependência do relógio do host. Criação define quatro timestamps. Escrita altera mtime/ctime, metadados alteram ctime, touch altera atime/mtime/ctime; diretório pai recebe mtime/ctime nas mutações de entries. Consultas imutáveis de texto/blob têm política noatime. Leitura por handle e redireção `<` atualizam atime quando consomem bytes. Não se alega epoch POSIX real.

A lixeira guarda o caminho de origem por entry em arquivos virtuais `.trashinfo`; assim, dois hardlinks enviados à lixeira podem ser restaurados independentemente. A listagem projeta os metadados de origem para a UI sem alterar o inode canônico.

## IPC e persistência

Wire VFS v2 usa `entries`, `inodes` e `nextInode`, sem repetir o conteúdo de hardlinks. `src/lib/api.ts` valida e projeta esse wire para nodes somente de leitura usados pela UI. Fixtures antigas de preview ainda aceitam `{ nodes }`. `vfs_stat` usa a resolução do backend; File Manager distingue symlinks e abre o target pelo mesmo backend.

WorldState mantém sua versão existente; VFS tem versão própria. Blobs continuam na camada SQLite existente, referenciados por hash. A inspeção de referências inclui inodes e journals. Missões restauram before-images por API privilegiada do VFS; pacote mantém suas projeções protegidas e recolhe inodes temporários ao concluir a sincronização.

## Invariantes e limites

Raiz diretório, pai válido, identidade única, referências existentes, projeções compartilhadas, nlink exato, ausência de diretórios com vários pais e de órfãos sem handles, allocator sem colisões. Load valida invariantes; WorldState.validate e os testes validam as fronteiras transacionais.

Limites: 10.000 entries por filesystem, texto de 1 MiB, 1024 handles por filesystem, 40 symlinks, blobs de até 32 MiB com limites existentes de snapshot/SQLite, capacidade lógica padrão de 64 GiB. Medições DEV ficam em `artifacts/vfs-performance.json`; não entram no runtime.

## Referências

[Linux path_resolution(7)](https://man7.org/linux/man-pages/man7/path_resolution.7.html) e [Linux inode(7)](https://man7.org/linux/man-pages/man7/inode.7.html). As expectativas são declaradas, sem alegar captura executada em um Linux de referência.
