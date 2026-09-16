# ADR 0002 — Archives no VFS

Data: 2026-09-15. Status: implementado.

## Decisão

`ArchiveService` interpreta bytes e modifica somente o VFS selecionado. Terminal,
File Manager, Archive Viewer, downloads e anexos utilizam o mesmo serviço Rust.
Os blobs existentes continuam persistidos no SQLite, referenciados por SHA-256;
clones de estado compartilham os bytes por `Arc`. Nenhum comando virtual executa
ZIP, TAR, shell, processo ou operação de extração do sistema hospedeiro.

## Bibliotecas avaliadas

| Dependência fixada no lockfile    | Decisão e licença                                                                                         |
| --------------------------------- | --------------------------------------------------------------------------------------------------------- |
| zip 8.6.0                         | Leitura do índice, Deflate, CRC e ZIP AES-256. MIT. Features limitadas a `deflate-flate2` e `aes-crypto`. |
| tar 0.4.46                        | Headers TAR e leitores incrementais; MIT/Apache-2.0. Sem `unpack` ou preservação de permissões do host.   |
| flate2 1.1.10                     | Gzip com backend Rust; MIT/Apache-2.0. Reutiliza dependência já existente.                                |
| bzip2 0.6.1                       | Backend Rust libbz2-rs-sys 0.2.5; wrapper MIT/Apache-2.0, backend bzip2-1.0.6.                            |
| liblzma 0.4.8 / liblzma-sys 0.4.9 | XZ estático, wrapper MIT/Apache-2.0 e liblzma 0BSD. Escolhido pela API de limite de memória do decoder.   |
| embed-resource 3.0.11             | MIT; manifesto Common Controls v6 em executáveis e harness de testes Windows.                             |

As versões publicadas, features, licenças e fontes foram conferidas em docs.rs e
nos pacotes baixados. O lockfile contém 26 pacotes novos, incluindo RustCrypto.
As versões recentes demonstram atividade, mas não constituem garantia de suporte
futuro. Mudanças de versão devem repetir os testes de compatibilidade e segurança.

Fontes: [ZIP](https://docs.rs/crate/zip/8.6.0),
[TAR](https://docs.rs/tar/0.4.46/tar/),
[flate2](https://docs.rs/flate2/1.1.10/flate2/),
[bzip2](https://docs.rs/bzip2/0.6.1/bzip2/),
[limite de memória XZ](https://docs.rs/liblzma/0.4.8/liblzma/stream/struct.Stream.html#method.new_stream_decoder),
[licenças XZ](https://tukaani.org/xz/).

## Segurança e compatibilidade

TAR 0.4.46 inclui a correção para a interpretação incorreta de tamanhos PAX
descrita em [RUSTSEC-2026-0068](https://rustsec.org/advisories/RUSTSEC-2026-0068.html).
Outros avisos históricos sobre extração para disco não tornam `unpack` aceitável:
esse projeto não utiliza essa API. Não se afirma ausência universal de vulnerabilidades.

O ZIP é interoperável para conteúdo materializado, incluindo AES-256; leitores sem
suporte AES podem não abrir os ZIPs protegidos. O TAR aceita nomes usuais iniciados
por `./`. ZIP64, volumes divididos, hardlinks, dispositivos, caminhos absolutos,
traversal e links com `..` são rejeitados. RAR e 7z permanecem explicitamente não
suportados. Os parsers das bibliotecas ficam encapsulados em `archive/codec.rs`.

Limites: 4096 entradas, 32 MiB de conteúdo materializado, 16 GiB lógicos, razão de
expansão 2000, 32 níveis de caminho, componente de até 255 bytes, 8 camadas de archive.
O decoder XZ recebe limite de 64 MiB. O ZIP verifica o número de entradas no EOCD
antes de construir o índice. Extração rejeita duplicatas e ancestrais que sejam
arquivos ou symlinks. Links virtuais são armazenados, nunca seguidos ao extrair.

## Conteúdo físico e virtual

Conteúdo materializado usa os codecs reais. Bytes virtuais adicionais usam perfis
de compressibilidade e um manifesto reservado `.cyber-war-vfs-v1.json` que preserva
tamanho lógico, timestamps, proprietário, grupo, MIME e referências de mídia local.
Gzip/bzip2/xz de arquivos esparsos carregam um pequeno envelope `CYBERWAR-SPARSE-V1`.
Ferramentas externas enxergam esse envelope ou o conteúdo materializado, não os GB
fictícios. ZIP criptografa também o manifesto; senhas não entram no save ou histórico.

O VFS fica pronto somente após validação completa. Falha de permissão, disco cheio,
senha, integridade ou cancelamento preserva os arquivos anteriores. Transações de
`GameService` persistem o resultado antes de publicá-lo. `ls`, `du` e `df` consultam
tamanhos lógicos; armazenamento real tem limites independentes.

## UI, jobs e eventos

Abrir ZIP lê o índice; abrir TAR percorre headers e descarta os corpos. TAR comprimido
precisa decodificar o stream para alcançar os headers. Não são criados nós VFS ao abrir.
`PARTIAL` significa conteúdo ainda não verificado; o botão de integridade lê e testa
todas as entradas. Erros de formato são apresentados ao usuário.

Operações grandes usam processos virtuais e duração proporcional ao tamanho/CPU.
O trabalho de codec roda em `spawn_blocking`; o cancelamento é atômico e checado antes
de publicar o VFS. Gzip/bzip2/xz usam buffers limitados, mas um bloco de codec pode
terminar antes de observar Ctrl+C. Jobs explícitos em background não aceitam prompts.
Pipes implementam stdout textual para `grep`; bytes comprimidos usam redirecionamento
para arquivo virtual. Man pages documentam somente o subconjunto implementado.

Eventos estruturados possuem paths virtuais, formato, destino e entradas. Missões
podem observar `archiveEvent` ou, preferencialmente, o arquivo resultante por
`fileContains`, preservando equivalência entre GUI e terminal.

## Windows, Tauri e tamanho

Validado com Rust 1.98.1 `x86_64-pc-windows-gnu`, LLVM-MinGW e WebView2. XZ e SQLite
são ligados estaticamente; WebView2 mantém a dependência normal do Tauri.
O manifesto também deve estar no harness: sem Common Controls v6, Windows retorna
`STATUS_ENTRYPOINT_NOT_FOUND` para `TaskDialogIndirect` antes de executar os testes.

Features opcionais dos codecs foram desativadas para evitar formatos e backends extras.
Não há build anterior equivalente disponível para medir o aumento isolado do binário;
tamanho total, testes e limitações de QA são registrados em `docs/ARCHIVES-QA.md`.
