# Coreutils — arquitetura

## Fonte de verdade

O discovery M0 determina quais executáveis pertencem a Coreutils. `content/cli-compatibility/coreutils.json` descreve cada subset, flags, contratos obrigatórios, implementação e lacunas. O manifest geral continua sendo a fonte da versão GNU. Não existe campo de promoção manual no manifest Coreutils.

`coreutils/foundation.rs` trata nomes de caminhos e ambiente. `coreutils/bytes.rs` trata leitura, recortes, duplicação, Base64 e SHA-256. `terminal_text.rs` conserva wc/sort/uniq; os handlers de consulta, cópia e remoção existentes continuam usando M1B. `coreutils/legacy.rs` rejeita opções desconhecidas antes que parsers antigos as descartem e produz ajuda a partir dos contratos. O dispatch continua validando pacote, PATH e permissão de execução antes do handler.

## Resolução e pacotes

Os 37 nomes descobertos foram vinculados ao pacote virtual Coreutils, inclusive os que anteriormente só funcionavam por dispatch nominal. Saves anteriores recebem bindings faltantes apenas quando o pacote baseline está instalado e não contém a definição antiga. Arquivos removidos de uma definição já atualizada não são recriados. `true`, `false`, `pwd`, `echo` e `printf` não são certificados acidentalmente como Coreutils: a classificação do inventário prevalece.

## Bytes e apresentação

Os canais cooperativos do shell transportam `Vec<u8>` em blocos de 4 KiB, com capacidade de 64 KiB. EOF e backpressure continuam distintos. `yes` e `head` por linhas continuam incrementais; fechar o consumidor encerra o produtor. Caminhos explícitos como `/usr/bin/head` participam do mesmo scheduler.

`Output.byte_ordered` conserva ordem e bytes de stdout/stderr; `binary` agrega stdout sem conversão. A conversão tolerante de UTF-8 acontece apenas ao produzir texto para o terminal visual. Os resultados de captura incluem `stdoutHex` e `stderrHex`; arquivos e comandos seguintes recebem bytes originais. Comandos ainda textuais rejeitam stdin binário inválido em vez de substituí-lo silenciosamente.

Handles M1B leem e escrevem conteúdo UTF-8 ou BlobRef. A cache de blobs continua sendo virtual e imutável. Escritas por handle preservam inode, offset, permissões, aliases e unlink. Append não exige leitura do arquivo pelo ator. Um fragmento que divide um caractere UTF-8 pode ser armazenado como blob e reconstruído por escrita posterior. O schema persistente permanece VFS v2.

O adaptador textual sem cache mantém seu limite de 1 MiB e rejeita sobreposição que dividiria UTF-8 antes de alterar o inode. IO binário exige a API com cache persistente. Escritas de zero bytes não estendem arquivos nem alteram timestamps.

Limites: adapters e saída acumulada de 4 MiB; payload binário de arquivo de 32 MiB; cache e saves mantêm limites próprios. Os adapters finitos de tail/tee/base64/hash e várias opções de cat/head ainda acumulam input antes do handler. Isso está nos blockers; não foi implementado streaming ilimitado para todas as ferramentas.

## Verificação

Casos estruturados passam pelo executable resolver usando `invocation: /usr/bin/NAME`. Testes Rust cobrem handlers e fluxos shell. Fixtures aceitam texto e bytes hexadecimais; estado reutiliza relações entre inodes e snapshots M1B. Diferenças binárias informam offset e trechos curtos em hexadecimal.

Três gates adicionais impedem certificação superficial: `COMMAND_CONTRACTS` exige todos os contratos e flags declarados; `GNU_REFERENCE` exige execução reproduzível da versão pinada, locale C e correspondência dos bytes/status; `KNOWN_GAPS` impede promoção enquanto houver lacunas conhecidas. Expectativas escritas a partir de documentação não satisfazem o gate GNU. Os gates M0, dependências e guard do host permanecem obrigatórios.

O fingerprint inclui shell/VFS/parser e o handler relevante. Casos diretos excluem handlers independentes; scripts de integração mantêm invalidação conservadora. Capturas antigas com fingerprint global continuam aceitas somente quando inteiramente atuais. Uma referência alterada ou versão divergente bloqueia promoção.

## Captura DEV

`python scripts/cli/coreutils-environment.py prepare` prepara o ambiente fixado no lock. O modo `run --capture --command NAME` executa o harness existente em Linux, bubblewrap e GNU da versão exata do manifest. Cada caso estruturado roda duas vezes em fixture temporária, sem rede, repositório ou home do desenvolvedor. Apenas executáveis/bibliotecas ficam visíveis em leitura. O harness registra versão/hash dos binários, locale, argv, stdin, stdout/stderr, status e snapshots. Scripts/setup do jogo são explicitamente excluídos de execução host. Veja [reference-environment.md](reference-environment.md).

Uma captura existente exige `--replace`. `cli:verify` lê evidência sem executar GNU; `coreutils-environment.py run --verify` reproduz GNU e compara sem atualizar golden. O contrato abstrato M0 `ReferenceEnvironment` continua sem fallback automático. A implementação de captura fica exclusivamente em tooling DEV e suas saídas são consumidas pelo pipeline. O jogador Windows não instala GNU, WSL ou bubblewrap.

As relações de inode são normalizadas dentro de cada snapshot. Os redirecionamentos Foundation comparam conteúdo e metadados relevantes; operações sem escrita exigem ausência de mutação. Capturas por executável ficam em `tests/cli/gnu/coreutils/<versão>/`. Digest de entrada, ambiente e harness impede reaproveitar uma referência desatualizada.
