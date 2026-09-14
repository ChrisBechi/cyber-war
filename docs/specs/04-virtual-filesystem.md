# Spec 04 — Virtual File System

## Objetivo

Simular filesystem Linux persistente e compartilhado entre terminal e GUI.

## Estrutura inicial

```text
/
├── bin
├── boot
├── dev
├── etc
├── home
│   └── kali
│       ├── Desktop
│       ├── Documents
│       ├── Downloads
│       ├── Music
│       ├── Pictures
│       ├── Videos
│       ├── projects
│       └── tools
├── opt
├── root
├── tmp
├── usr
└── var
```

## VfsNode

```text
id
parent_id
name
kind
content
blob? { hash, size, mime }
owner
group
mode
created_at
modified_at
metadata
```

## Regras

- paths absolutos/relativos;
- `.` e `..`;
- `~`;
- permissões Linux simuladas;
- owner/group;
- symlinks futuros;
- blobs/assets;
- mounts de missão;
- nada mapeia arbitrariamente ao host;
- estado entra no save.

## Armazenamento implementado — 0.4.2

Texto permanece inline, limitado a 1 MiB. Binários usam referência opcional e bytes imutáveis em `vfs_blobs` no SQLite; cache compartilhado transitório, não serializado. Limites: 32 MiB/arquivo, 128 MiB de referências únicas/snapshot, 512 MiB retidos/banco. SHA-256 deduplica e verifica integridade no carregamento. Referências em journals também são preservadas.

Importação exige escolha explícita no gerenciador, nome virtual válido, permissão e criação exclusiva ou versão esperada para substituir. Nunca é leitura por caminho do host enviado ao backend. Cópia/movimento/lixeira preservam blob; escrita textual explícita substitui-o; append/leitura textual de binário falham. Players usam IPC de bytes e URLs Blob temporárias. Ainda não há coleta de órfãos, streaming, integração de todos os utilitários CLI ou suporte universal a codecs. Ver [contrato detalhado](../APROFUNDAMENTO-0.4.2.md).
