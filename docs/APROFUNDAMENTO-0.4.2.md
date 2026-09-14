# Aprofundamento: comandos, shell, nano e mídia — 0.4.2

Checkpoint de 2026-09-14. Implementação verificada por testes automatizados, não certificação de equivalência com GNU/Linux. Complementa os [oito contratos anteriores](CONTRATOS-COMANDOS-0.4.2.md). Mantidos o visual do terminal, a campanha existente e a regra de uma missão ativa.

## Comandos e flags

| Família                          | Contrato implementado nesta etapa                                                                                                             | Limites relevantes                                                                                                                                                                                                                                               |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ls`                             | `-1aAldFhrSt`, `--all`, `--almost-all`, `--directory`, `--classify`, `--human-readable`, `--reverse`, caminhos múltiplos, `--`                | Uma entrada por linha; tamanho binário correto; datas do relógio virtual. Sem links, cores, `-R`, alocação real ou locale GNU. `-h` usa escala K simplificada.                                                                                                   |
| `grep`                           | `-EFGivnclLqHhwx`, `-e PATTERN` repetido, `-m COUNT`, `--`; BRE/ERE limitados ou busca literal                                                | Retorno 0 para correspondência, 1 para ausência, 2 para erro; preserva saída de arquivos legíveis junto com diagnóstico. Sem stdin, recursão, contexto, PCRE, backreferences, lookaround ou busca binária. Regex usa a biblioteca Rust, não a implementação GNU. |
| `find`                           | Raízes múltiplas, `-name`, `-iname`, `-type f/d`, `-mindepth`, `-maxdepth`, `-print`, AND                                                     | Inclui a raiz na profundidade zero. Padrões `*` e `?`; sem classes `[]`, OR/NOT, `-exec`, `-delete` ou links.                                                                                                                                                    |
| `chmod`                          | `-R`, `-v`, octal 000–777, classes explícitas `u/g/o/a` com `+/-/=` e `r/w/x/X`, cláusulas separadas por vírgula                              | Sem bits especiais, umask implícito ou cópia entre classes. Mutação atômica, conforme o VFS do jogo.                                                                                                                                                             |
| `chown`                          | `-R`, `-v`, `OWNER`, `OWNER:GROUP`, `:GROUP`                                                                                                  | Root virtual; usuários/grupos conhecidos. Alterar somente proprietário preserva grupo. Sem IDs arbitrários, `--reference` ou chown POSIX completo.                                                                                                               |
| `service` / `systemctl`          | start/stop/restart/status; is-active/is-enabled; enable/disable e `--now`; list-units/list-unit-files; quiet/no-pager; `service --status-all` | Unidades predefinidas, uma por operação. Status inativo retorna 3, is-enabled desabilitado retorna 1. Sem criação de unidades ou daemon-reload. Habilitação/listagem remota são recusadas.                                                                       |
| `journalctl`                     | `-u UNIT`, `-n N`, quiet/no-pager                                                                                                             | Usa eventos realmente registrados para o host/unidade virtual; não fabrica mensagens de saúde. Sem follow ou filtros de data.                                                                                                                                    |
| `ps`, `kill`, `killall`, `pkill` | ps `-e/-A/-ef/aux`, `-p PID`; sinais 0, 9 e 15; pkill regex e `-x`                                                                            | Processos locais virtuais; permissões e processo-base protegido. Encerrar processo de serviço atualiza o serviço. Sem scheduler, grupos, demais sinais ou métricas completas de `ps aux`.                                                                        |
| `apt` / `apt-get`                | Validação explícita das opções aceitas `-y/-i/-v`, help/version e inventário virtual existente                                                | Flags como `--fix-broken` são recusadas. Operações de pacotes via SSH não alteram inventário local. Não há download, resolução completa de dependências, scripts de instalação ou auditoria integral de apt-cache/dpkg.                                          |

Os comandos de consulta/arquivos usam o VFS do host virtual selecionado. Operações administrativas nunca elevam privilégios do Windows. Limites de saída: 4 MiB nos contratos de leitura/busca e scripts. `man` e `--help` dos cinco novos comandos de consulta/permissão descrevem o subconjunto; outros manuais legados ainda não são documentação GNU integral.

## Shell

O novo parser mantém aspas simples/duplas, escapes, comentários e expansão sem transformar o valor de uma variável novamente em código. Por exemplo, um valor `a b > arquivo` não cria redirecionamento ao ser expandido. NUL/ESC de entradas, escapes ou variáveis são recusados.

- Variáveis `$NAME`/`${NAME}`, `$0`–`$9`, `${10}`, `$@`, `$*`, `$#`, `$?`; `"$@"` conserva fronteiras e argumentos vazios.
- Atribuições isoladas e exportação; valores de atribuição não sofrem separação por espaços. Variáveis não exportadas não passam ao shell filho, exceto ambiente-base do simulador.
- `bash/sh SCRIPT [ARGS...]`, `bash/sh -c COMMANDS [NAME [ARGS...]]`, `source` e `.`.
- Shell filho restaura cwd, usuário, SSH e ambiente do chamador; efeitos no VFS permanecem. Source compartilha o contexto e restaura argumentos posicionais na volta.
- stdout, stderr e status final permanecem independentes. Erros comuns não introduzem `set -e` implicitamente. `exit` e `return` numéricos usam status de 8 bits; return se limita a arquivos incluídos. Exit dentro de source encerra o script pai.
- Limites: 512 linhas por script, oito níveis, 8192 bytes por linha, 4096 argumentos/64 KiB expandidos e 4 MiB de saída. Abertura de editor interativo dentro de script é recusada sem deixar foreground preso.

Ainda não implementados: pipelines, stdin, `&&`/`||`, múltiplos comandos por `;`, jobs/background, command substitution, globbing geral, loops, funções, expansões complexas, aspas multilinha e opções `set`. `>`/`>>` continuam o contrato anterior: uma redireção final de stdout, destino vinculado antes do comando, sem `2>` nem fluxo binário. Shebangs são limitados a sh/bash e env sh/bash. Não é Bash real.

## Nano

Corrigidos comportamentos concretos da tela alternativa existente:

- Busca literal sem diferenciar caixa por padrão; retorno ao começo da mesma linha, posições Unicode por ponto de código, mensagem de não encontrado preservada; M-C alterna caixa, M-B sentido e M-W repete.
- Substituição com confirmação por ocorrência: Y, N, A e cancelar. A operação respeita a seleção e não volta a substituir o próprio texto recém-inserido. Substituir todas é uma etapa no histórico.
- Marcação, corte/cópia de seleção multilinha, M-6 para copiar, cortes consecutivos acumulados e colagem na posição do cursor preservando separadores de linha.
- M-U/M-E para desfazer/refazer, além dos atalhos legados. Inserir texto não apaga automaticamente uma seleção: o modo zap não foi implementado.
- Colagem CRLF não duplica linhas; Enter não acrescenta uma nova linha final fantasma. Autoindent não é aplicado a cada linha colada.
- Salvar ao sair fecha após sucesso; erro de escrita mantém o buffer disponível. Permanecem conflito de versão, backups, readonly e restrições do backend.

Sem alteração de CSS, fonte, cores ou dimensões do terminal. O destaque da seleção e os prompts são funcionais. Continuam parciais: softwrap/reflow, tabulação visual, grafemas compostos, formatos DOS/Mac, buffer sem nome, confirmação completa de sobrescrita em salvar como, multibuffer, regex, nanorc e corretor. A matriz de flags aceitas/recusadas permanece no [relatório](RELATORIO-CORRECOES-0.4.2.md). Não são “todas as opções GNU nano”.

## Arquivos binários e persistência

`VfsNode.blob` é opcional: `{ hash, size, mime }`. Arquivos textuais antigos continuam com `content`. Bytes ficam em `vfs_blobs(hash PRIMARY KEY, data BLOB)` no SQLite; a migração do banco passa a `user_version=3`. O snapshot do mundo permanece schema 2: o campo novo é aditivo e tem default.

Snapshots e journals guardam referências, nunca base64 nem payloads em JSON. O cache transitório usa bytes compartilhados por `Arc`, evitando cópia integral em cada candidato transacional. SHA-256 identifica/deduplica conteúdos; load valida tamanho e hash antes de publicar o mundo. Isso detecta corrupção, não é criptografia.

Importação, referência do arquivo, novos blobs e snapshot pertencem à mesma transação. Erro de SQLite não publica um arquivo sem bytes nem bytes órfãos daquele commit. Cópia, movimento, lixeira, restore e checkpoints preservam referências. Bytes anteriores guardados apenas no diário de missão são hidratados antes de descartar a tentativa; a regressão cobre substituição temporária por texto e recuperação do binário original.

Limites explícitos:

- 32 MiB por arquivo binário; 128 MiB de conteúdos únicos referenciados por snapshot, incluindo journals; 512 MiB de blobs retidos por banco, compartilhados pelos slots.
- Ainda não há coleta de blobs órfãos: apagar um arquivo não libera necessariamente a capacidade do banco, para não invalidar snapshots antigos. O cache também pode reter bytes até recarregar a sessão.
- Importação usa base64 temporariamente no IPC, sem streaming. Arquivos textuais continuam limitados a 1 MiB.
- Leitores textuais e nano recusam blobs; append de texto em binário falha. Sobrescrever explicitamente com texto remove a referência/metadados de mídia.
- `ls` mostra o tamanho binário. Utilitários legados como stat/du/file/sha256sum/strings/base64 e SCP ainda precisam de integração binária e auditoria; não se deve usar seus resultados como validação desses bytes.

## Importação e players

“Importar arquivo…” no gerenciador abre seleção explícita do usuário. Apenas o arquivo selecionado é lido pelo frontend; caminhos recebidos pelo núcleo continuam exclusivamente virtuais. Nome inválido, UTF-8 inválido em texto, tamanho excessivo, conflito e falta de permissão produzem erro. A GUI importa com criação exclusiva, sem sobrescrever silenciosamente. O IPC permite substituição somente com versão esperada.

Texto/scripts e legendas SRT/VTT continuam textuais. Outros formatos conservam bytes exatos. Para binários, o MIME declarado prevalece sobre uma extensão renomeada; um blob `.sh` não vira script executável. MIME não é detecção de conteúdo nem promessa de codec.

Os players leem bytes pelo IPC após validar permissão, criam uma URL Blob transitória e a revogam ao fechar/trocar arquivo. Respostas atrasadas não publicam fontes depois do fechamento. Recursos demonstrativos antigos continuam disponíveis, mas também exigem leitura autorizada do VFS.

Áudio/vídeo: play/pause sem autoplay, seek limitado à duração, saltos ±10s, mute, loop, velocidade, volume do player multiplicado pelo volume global/canal música. Legendas SRT viram VTT e o modo é reaplicado quando a track carrega. Fullscreen trata recusa e a posse do elemento. Imagens têm ajuste que restaura zoom, rotação/teclado e erros de decodificação.

CSP admite `blob:` em mídia/imagens e mantém bloqueio de fontes externas. SVG/HTML importados não são publicados como documentos ativos. Nenhum player lança um executável do host; decodificação depende do WebView2. Codecs, timing de legendas, fullscreen nativo, playlists e sessões longas ainda precisam de QA; PDF/arquivos compactados são armazenáveis, mas não possuem leitor/extrator.

## Evidência e roteiro restante

Novas regressões em `terminal_query_tests.rs`, `shell_tests.rs`, `binary_tests.rs`, `mission_persistence_tests.rs`, `nano.test.ts`, `media.test.tsx`, `import-file.test.ts` e associações. Resultados finais e comandos reproduzíveis ficam no [relatório](RELATORIO-CORRECOES-0.4.2.md#18-testes-e-gates).

QA manual ainda necessário: importar PNG/WAV/WebM reais; fechar/carregar, copiar e restaurar da lixeira; vídeo com SRT/VTT, seek durante reprodução, volumes global/local e fullscreen; nano em duas janelas, acentos/grafemas, seleção, salvar como e teclado real. Os testes de mídia usam APIs de reprodução simuladas no DOM, não decodificadores reais. Não houve execução GNU/Linux como oracle, inspeção visual nova ou geração de instalador Windows deste checkpoint.

Referências primárias usadas para delimitar o subconjunto: [grep](https://www.gnu.org/s/grep/manual/grep.html), [find](https://www.gnu.org/software/findutils/manual/html_node/find_html/Directories.html), [parâmetros Bash](https://www.gnu.org/software/bash/manual/html_node/Special-Parameters.html), [nano](https://www.nano-editor.org/dist/latest/nano.html), [TextTrack/HTMLMediaElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/textTracks). As referências não substituem as limitações explícitas acima.
