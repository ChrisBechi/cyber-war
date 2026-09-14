# Contratos verticais do terminal — checkpoint 0.4.2

Implementados nesta etapa: `pwd`, `cd`, `cat`, `head`, `tail`, `cp`, `mv`, `rm`, seus manuais e integração de resultados/redirecionamento. Não é certificação Bash/GNU/Kali completa. Nenhum comando do jogador é executado no Windows, WSL ou rede real.

Este documento registra o checkpoint inicial de oito comandos. O [aprofundamento seguinte](APROFUNDAMENTO-0.4.2.md) acrescenta contratos de ls/grep/find/permissões, serviços/processos, shell, nano e armazenamento binário.

## Levantamento das missões

Fontes verificadas: `content/missions/session_1/vertical-slice.json`, `content/missions/session_2/orion.json`, `src-tauri/src/campaign_tests.rs` e os executores reais. Catálogo e roadmap não comprovam implementação.

| Missão                  | Fluxo efetivamente usado                                                        |
| ----------------------- | ------------------------------------------------------------------------------- |
| first-boot              | `echo` e `>` criam `Documents/notes.txt`; GUI lê o mesmo VFS                    |
| v1                      | `wget https://archive.org` ou navegador; memória/compilação por `lab`           |
| v2                      | Escrita de `profile_01.dat` pelo editor ou CodeLab/`lab`                        |
| wifi                    | `lab wifi/collect/analyze/connect`                                              |
| pendrive                | `cp /dev/usb.img Documents/usb-backup.img` antes de `lab recover`               |
| forum-job               | SSH autorizado, `cat /var/log/web.log`, `sudo edit`, `sudo service web restart` |
| vex-setup               | `sudo edit /srv/www/health.txt OK`, `service web status`                        |
| vex-production          | Backup remoto com `cp`, `lab verify-backup`, decisão de hardening               |
| vex-incident            | `lab contain/restore` no host virtual                                           |
| girl                    | Navegador, mensagens e correlação de pistas; não exige shell                    |
| signal-no-ar / em-claro | Workspaces aircrack-ng/Wireshark consultam sinal e captura do cenário           |

Não há pipelines, `&&`/`||`, background ou substituição de comandos obrigatórios nessas missões. Não foram introduzidos nesta etapa. Navegação, head/tail e gerenciamento de arquivos aprofundam capacidades já solicitadas, sem novas missões.

## Contrato comum

- Caminhos e permissões usam somente o VFS local ou SSH selecionado. Cwd/ambiente são por sessão; arquivos são compartilhados por host.
- `--` encerra opções. Clusters curtos e opções longas listadas abaixo funcionam. Flag desconhecida falha antes da operação; não ativa recursão/força por conter uma letra coincidente.
- `--help` e `man COMANDO` mostram o mesmo contrato dos oito comandos. Demais manuais se identificam como resumos legados ainda não auditados.
- Sucesso: `exitCode: 0`. Erros desse subconjunto: `1` e diagnóstico em stderr; não há equivalência de todas as mensagens/códigos GNU. Comando desconhecido permanece `127`; `false` e `sudo false` retornam `1` sem diagnóstico.
- cat/head/tail continuam pelos arquivos restantes após falha de leitura, preservando stdout válido e retornando status não zero. O IPC mantém duas strings separadas, não a intercalação cronológica de stdout/stderr.
- Stdin interativo, pipelines e fonte `-` não estão implementados: sem arquivos ou com operando `-`, há erro explícito. `./-` pode identificar um arquivo do VFS.
- Leitores têm teto de saída de 4 MiB; o VFS permanece limitado a 1 MiB por conteúdo textual. Falha por limite não finge sucesso com truncamento.
- cp/mv/rm são operações atômicas do jogo: erro desfaz o comando, diferindo dos efeitos parciais possíveis no GNU. O journal de missões e a regra de missão única permanecem ativos.

## Navegação

| Comando | Sintaxe implementada              | Resultado / efeitos                                                                                                                  |
| ------- | --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| pwd     | `pwd [-L\|-P] [--]`               | Cwd absoluto mais nova linha; sem operandos neste subconjunto                                                                        |
| cd      | `cd [-L\|-P] [--] [DIRECTORY\|-]` | Sem argumento usa HOME. Valida travessia; atualiza cwd/PWD/OLDPWD. `cd -` imprime o destino anterior. String vazia não altera estado |

Sem links simbólicos, -L e -P convergem. CDPATH não existe. Falhas não mudam contexto. `sudo cd` é recusado para não alterar o shell chamador sob identidade diferente. Na mudança de host, PWD acompanha cwd e OLDPWD do host anterior é removido; uma pilha completa de shells SSH permanece pendente.

HOME/PWD/OLDPWD e `cd -` foram conferidos no [manual de builtins do Bash](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html).

## Leitura de arquivos

| Comando | Opções implementadas                                                                                             | Saída                                                                                                                                                 |
| ------- | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| cat     | `-n/--number`, `-b/--number-nonblank`, `-s/--squeeze-blank`, `-E/--show-ends`, `-T/--show-tabs`, `-u`            | Concatenação; numeração contínua entre arquivos; -b prevalece sobre -n. CR/LF/tabs preservados salvo transformação explícita. -u não altera buffering |
| head    | `-n N`, `-c N`, contagem negativa, `--lines=N`, `--bytes=N`, `-q/--quiet/--silent`, `-v/--verbose`, `-N` inicial | Dez primeiras linhas por padrão; contagem negativa exclui últimas N unidades; preserva linha incompleta                                               |
| tail    | Mesmas flags de seleção/headers; `+N` para origem                                                                | Dez últimas linhas por padrão; +N começa na unidade N com origem 1; preserva CR/LF e linha incompleta                                                 |

Valores anexados ou separados funcionam, e a última seleção -n/-c prevalece. Headers usam o nome fornecido pelo jogador; última opção -q/-v vence. Contagens são decimais, sem sufixos K/M. Não há `-f/--follow`, delimitador NUL nem outras flags não listadas. Cat não implementa -v/-e/-t/-A.

Recortes em bytes só são entregues se continuarem UTF-8 válidos. Cortar um caractere multibyte retorna erro explícito: strings do IPC não transportam esses bytes brutos sem alteração. Esses leitores não aceitam os blobs binários introduzidos no checkpoint seguinte.

Referências: [cat](https://www.gnu.org/software/coreutils/manual/html_node/cat-invocation.html), [head](https://www.gnu.org/software/coreutils/manual/html_node/head-invocation.html), [tail](https://www.gnu.org/software/coreutils/manual/html_node/tail-invocation.html). As fixtures validam o simulador; não houve execução de GNU/Linux real neste checkpoint.

## Cópia, movimento e remoção

| Comando | Opções / argumentos                                                        | Efeitos e limites                                                                                                                        |
| ------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| cp      | `-r/-R/--recursive`, `-n/--no-clobber`, `-v/--verbose`; SOURCE DESTINATION | Diretório exige recursão. Arquivo regular gravável pode ser sobrescrito, mantendo owner/mode do destino. -n preserva destino existente   |
| mv      | `-n/--no-clobber`, `-v/--verbose`; SOURCE DESTINATION                      | Move/renomeia arquivo ou diretório; substitui arquivo regular. -n não remove origem em colisão                                           |
| rm      | `-r/-R/--recursive`, `-f/--force`, `-v/--verbose`; PATH...                 | Remoção permanente. Diretórios exigem recursão. -f ignora ausência, não permissões. Proteção da raiz e infraestrutura da lixeira virtual |

Não há merge de diretórios existentes, prompts interativos, hardlinks, umask completo, preservação POSIX integral ou cp -a/-p/-f. Origem igual ao destino e cópia para descendente são recusadas. Permissões são as do VFS: mv ainda depende das verificações de leitura/travessia do transfer virtual, mais restritas que um rename POSIX. A GUI conserva sua política de colagem sem sobrescrita; substituição foi acrescentada no terminal.

## Redirecionamento e sudo

`>` abre/trunca antes da execução; `>>` valida escrita sem exigir leitura. Destino é vinculado ao cwd/host de origem, mesmo se o comando mudar o SSH. Só stdout é gravado; stderr permanece separado. Redireção simples deve estar no final da linha; `2>` e múltiplos redirecionamentos não foram implementados.

Leitores auditados podem retornar saída e erro juntos, inclusive redirecionados. Erros estruturais/legados em `GameError` mantêm o rollback anterior, inclusive da preparação do destino. Portanto, ainda não existe equivalência completa às falhas de shell real. Sudo preserva o status dos comandos auditados e usa apenas root virtual; sudoers/PAM e shells administrativos completos não existem.

## Evidência e pendências

Implementação: `terminal_io.rs`, integração em `terminal.rs`, preparação do destino em `vfs.rs`. Regressão: `terminal_io_tests.rs`, além dos testes de campanha, sessões, missão e VFS. Nenhum CSS ou componente visual do terminal foi alterado.

Os oito contratos não certificam outros comandos. Ls/grep/find/chmod/chown, serviços/processos, scripts, nano e binários/mídia foram aprofundados no documento complementar. Rede, SSH/SCP, demais utilitários/pacotes, classificação do catálogo e instaladores ainda exigem trabalho.
