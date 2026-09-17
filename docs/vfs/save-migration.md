# Migração de saves VFS

## Formato 2

Cada filesystem serializa `formatVersion: 2`, `entries` (caminho → inode), `inodes` (payload/metadados canônicos), `nextInode`, `clock`, `capacityBytes`, `identities` e `readOnly`. Os IDs permanecem estáveis após load e o allocator é monotônico. Handles e umask de sessão não são persistidos.

Hardlinks salvam somente um conteúdo. BlobRefs usam os hashes e a tabela `vfs_blobs` existentes; nenhuma duplicação de bytes binários dentro do JSON. Journals de missões continuam contendo before-images suficientes para desfazer alterações; sua restauração atualiza o inode compartilhado por API controlada.

## Formato legado

O deserializador reconhece `{nodes, clock, capacity_bytes}` sem versão. Ordena entries por profundidade e caminho, atribui IDs determinísticos e cria um inode independente para cada node antigo. Conteúdos iguais NÃO viram hardlinks. Preserva texto, BlobRef, symlink target, owner/group, mode e timestamps antigos; atime/ctime ausentes derivam de modifiedAt.

Identidades conhecidas têm IDs fixos; outros nomes recebem IDs determinísticos na migração. Diretórios reconstruídos determinam entries e nlink. Se `/dev` existe como diretório, os dispositivos null/zero ausentes são acrescentados sem substituir entries do jogador. Permissões legadas, inclusive `/tmp`, são preservadas.

A migração não reinstala arquivos apagados pelo jogador. A definição de pacote baseline pode ganhar somente as novas bindings conhecidas (`ln`, `rmdir` e a migração anterior de `yes`) quando a definição antiga ainda não as conhece; inicializações posteriores são idempotentes.

## Falhas

Versão desconhecida, entry sem inode, identidade inválida, allocator em conflito, excesso de entries, tipo/mode inválido, grafo inconsistente ou nlink divergente falham explicitamente. O carregamento transacional existente preserva o save anterior em falhas. Sem reparos destrutivos silenciosos.

## Evidência

`vfs::fidelity_tests::migration_preserves_legacy_metadata_and_sharing_roundtrip`, testes de corrupção em `vfs::integration_tests`, cinco slots com hardlinks/symlinks, checkpoints existentes, rollback de missões com blobs e casos declarativos `vfs/save/*`. IDs dos casos e resultados atuais são gerados em `docs/generated/vfs-compatibility.md`.
