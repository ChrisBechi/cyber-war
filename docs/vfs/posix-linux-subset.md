# Subset suportado e limites intencionais

## Matriz

| Área                              | Estado      | Escopo                                                                           |
| --------------------------------- | ----------- | -------------------------------------------------------------------------------- |
| Paths, entries e inodes           | SUPPORTED   | Resolução virtual limitada, identidade estável e nomes Unicode                   |
| Symlinks e hardlinks              | SUPPORTED   | Travessia/compartilhamento/unlink/save; sem hardlink de diretórios               |
| Permissões                        | SUPPORTED   | Uid/gid/grupos, owner/group/other, read/search separados                         |
| Bits especiais                    | PARTIAL     | Sticky e herança setgid; setuid/setgid executáveis não mudam identidade          |
| Timestamps                        | PARTIAL     | Ticks determinísticos; noatime em consultas imutáveis, atime em reads de handles |
| Handles                           | PARTIAL     | Offsets/append/truncate/unlink aberto para texto UTF-8; blobs via API separada   |
| Dispositivos                      | PARTIAL     | Null e zero limitado; random e tty ausentes                                      |
| Mounts, xattrs, ACLs, locks POSIX | UNSUPPORTED | Não necessários ao subset atual                                                  |
| Save/load                         | SUPPORTED   | Wire 2 e migração determinística de nodes legados                                |

## Suportado

- Arquivos de texto, payloads binários por BlobRef, diretórios, symlinks e hardlinks persistentes.
- Resolução componente a componente, case-sensitive, links absolutos/relativos/dangling, limite de loops, stat/lstat/readlink.
- Owner/group/other, search em diretórios, grupos suplementares, sticky, umask e herança setgid de diretórios.
- Handles virtuais com offsets, append/truncate/create/exclusive, inode aberto após unlink.
- `/dev/null` e leitura limitada por handle de `/dev/zero`; identidade do dispositivo pertence ao inode, inclusive por hardlink.
- `ln [-s]`, `rmdir`, `umask` octal; `stat [-L] [-c FORMAT]` com campos de inode, links, mode, uid/gid, tamanho e timestamps. A primitive stat segue symlink; o comando stat segue a convenção GNU de lstat por padrão e dereference com `-L`.
- chmod octal e classes explícitas u/g/o/a com r/w/x/X/s/t; ls exibe nlink e bits especiais.
- Terminal, UI e persistência observam os mesmos inodes; renomear cwd atualiza sessões por identidade. Cwd removido recua a HOME ou raiz virtual, política deliberadamente simplificada.

## Parcial / fora deste marco

- Conteúdo arbitrário binário continua no BlobRef API; handles de texto exigem UTF-8. Não há edição byte a byte de blobs, sparse files, extents, block accounting físico ou arquivos de tamanho ilimitado.
- Sem mount namespace, múltiplos dispositivos montados, ACL, capabilities Linux, sockets/FIFO, locks POSIX, mmap, procfs completo ou emulação ext4. EXDEV existe no vocabulário; não há operação que receba um inode de outro host.
- Setuid/setgid de executáveis são metadados e não alteram a identidade de execução. `chmod` com classes implícitas ou cópia de classe permanece fora do subset.
- `/dev/random`, `/dev/urandom` e `/dev/tty` não são simulados. Leitura indefinida de `/dev/zero` por cat/redireção é recusada; chamadas limitadas por handle funcionam.
- Consultas imutáveis usam noatime; timestamps são ticks determinísticos e não segundos de relógio real. Arquivos legados preservam os valores previamente salvos.
- `cd -L/-P` ainda usa resolução física; sem todos os casos de cwd deletado de Unix.
- Tar suporta symlinks, porém entradas hardlink do formato TAR continuam rejeitadas explicitamente; criação de archive materializa arquivos ligados como entries de conteúdo independentes. ZIP mantém seu contrato de metadados próprio. A política anti-traversal da extração é mais restritiva que o resolver POSIX normal.
- A interface distingue symlinks e abre destinos; não expõe editor de uid/gid ou gerenciador de inodes.
- Compatibilidade completa GNU Coreutils 9.7 permanece outro marco. Não há promoção automática a VERIFIED por possuir um VFS melhor.

## Segurança

O VFS é um modelo fechado de dados em memória. Nomes que se parecem com caminhos Windows continuam dentro dele. Imports explícitos passam pela camada binária controlada. Packages usam validação própria de caminhos de instalação; archives recusam atravessar symlinks no destino e recusam entries inseguras. Não existe fallback para comandos, leitura de paths ou sockets do host.
