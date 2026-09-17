# Milestone 1B — auditoria inicial

Base: commit `35d9418`, após Milestone 1A. A fila atual aponta coreutils → `NEEDS_SUBSYSTEM` → `VFS:PARTIAL`; as sete capacidades de fundação do shell possuem evidência aprovada.

## Estado encontrado

- `vfs.rs`: mapa por caminho, conteúdo/metadados duplicados por node; sem inode/hardlink. `stat` não distingue `lstat`; ancestral symlink é rejeitado. `normalize` colapsa `..` antes da travessia e rejeita nomes válidos no Linux.
- Permissões: owner/group/other por nomes, sem grupos suplementares; root ignora todos os bits; criação fixa 0644/0755; `/tmp` 0777; chmod restrito a 0777.
- VFS é compartilhado por terminal, arquivos, downloads, missões, pacotes e archives. Hosts remotos possuem instâncias próprias. Há edição direta de metadados em comandos, instalação, missões e extração.
- Binary payloads usam BlobRef e cache de conteúdo; o save persiste blobs em SQLite através da inspeção dos snapshots, inclusive journals de missões.
- Saves: WorldState versões 1/2; VFS sem versão própria, serializado como nodes/clock/capacity_bytes. Slots e checkpoints independentes; load valida integridade e WorldState.
- Shell: normalização lexical antes de resolver; descritores de saída guardam caminho/offset; `/dev/null` implementado no shell. Globs usam `child_names` ordenado.
- Archives: TAR já cria symlinks, restaura metadados e faz checks de destino; a rejeição implícita de ancestrais symlink precisa virar uma regra explícita de extração.
- Limites: 10.000 entradas, texto de 1 MiB, capacidade lógica de 64 GiB; não equivalem a blocos ext4. Leitura de texto e blobs possuem contratos separados.

## Evolução planejada

Tabela de inodes com conteúdo compartilhado, diretórios com entries, projeção de caminhos compatível com os consumidores; mutações centralizadas. Resolver iterativo com follow/no-follow, erros tipados, permissões e limites. Handles virtuais e dispositivos limitados. Saves versionados com migração determinística e sem duplicar conteúdo de hardlinks. Integração seguida por testes de invariantes, casos declarativos, benchmarks e gates completos.

## Referências

[Linux path_resolution(7)](https://man7.org/linux/man-pages/man7/path_resolution.7.html) fundamenta travessia componente a componente, busca em diretórios, links relativos, follow/no-follow e limite de 40 links. [Linux inode(7)](https://man7.org/linux/man-pages/man7/inode.7.html) fundamenta identidade, tipos e metadados. Expectativas DEV continuarão declaradas; este marco não certifica coreutils completo.
