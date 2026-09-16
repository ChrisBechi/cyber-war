# Bash 5.2.37 — subconjunto implementado

Referência: [manual versionado do Bash no Debian trixie](https://manpages.debian.org/trixie/bash/bash.1.en.html). Os diagnósticos de limites virtuais identificam o subconjunto do jogo. Não são uma cópia integral dos diagnósticos GNU.

## Disponível

| Recurso                         | Exemplo / comportamento                                                                                                                           |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Aspas simples e duplas          | `echo 'literal $HOME' "$HOME"`; concatenação `pre" duas palavras"post`; argumentos vazios.                                                        |
| Escapes e comentários           | `a\ b`, continuação por backslash-newline, `#` no início de token.                                                                                |
| Variáveis                       | `$NAME`, `${NAME}`, `$0`, `$1`…`${10}`, `$#`, `$@`, `$*`, `$?`, `$$`; `$!` para jobs de arquivos iniciados pelo shell.                            |
| Atribuições                     | `A='a b' B=$A`; prefixo `A=value bash -c 'echo "$A"'` é temporário e exportado para o comando.                                                    |
| Ambiente                        | `export NAME`, `export NAME=value`, `export -p`, `unset NAME`; filhos herdam exportadas e o ambiente virtual padrão.                              |
| Divisão de campos               | IFS padrão e IFS personalizado; valores entre aspas não são divididos.                                                                            |
| Tilde                           | `~` e `~/...` sem aspas usam HOME virtual; `~user` não é expandido.                                                                               |
| Globs                           | `*`, `?`, `[abc]`, `[a-z]`, `[!abc]`; componentes por diretório; ordem determinística; dotfiles exigem ponto inicial; sem match mantém literal.   |
| Substituição                    | `$(...)`, aninhamento, remoção de newlines finais, stderr separado e isolamento de cwd/env.                                                       |
| Listas                          | `;`, newline, `&&`, `\|\|`; expansão tardia e status correto para curto-circuito.                                                                 |
| Pipes                           | `a \| b \| c`; canais limitados, EOF, consumidor fechado, backpressure, status do último estágio.                                                 |
| Redirecionamentos               | `<`, `0<`, `>`, `1>`, `>>`, `1>>`, `2>`, `2>>`, `>&1`, `>&2`, `1>&1`, `1>&2`, `2>&1`, `2>&2`; sem espaços; apenas os descritores 0/1/2 indicados. |
| Comando só com redirecionamento | `>empty`; permissões e truncamento passam pelo VFS.                                                                                               |
| Scripts                         | `bash file args`, `sh file args`, `bash -c text name args`; mesmo parser do terminal.                                                             |
| Source                          | `source file args` ou `. file args`; cwd/env compartilhados; `return` retorna ao chamador.                                                        |
| Status                          | 127 não encontrado; 126 não executável; 2 sintaxe/opção do shell; 130 cancelamento; 141 produtor com consumidor encerrado.                        |
| Entrada interativa              | `cat`/`head` incrementais; Ctrl+D fornece EOF, Ctrl+C cancela; linha vazia com Ctrl+D fecha a janela.                                             |
| Terminal                        | Continuação para aspas, pipes e substituições incompletas; saída incremental e histórico original; visual existente preservado.                   |

## Intencionalmente parcial ou ausente

- Bash como software completo: loops, funções, agrupamentos/subshells `(…)`, `set`, `eval`, `exec`, aliases definidos pelo usuário, aritmética, brace expansion, arrays e process substitution não foram implementados.
- Backticks, operadores `${VAR:-default}` e variantes, `~user`, extglob/globstar, here-documents/here-strings, `|&`, `pipefail`, descritores arbitrários, fechamento `>&-` e duplicação de stdin permanecem fora do subconjunto.
- As palavras reservadas e formas não suportadas são rejeitadas explicitamente. Entrada incompleta em script é erro; no terminal, solicita continuação.
- Background aceita os jobs de arquivos existentes. Não há job control geral, terminal de controle POSIX ou sinais gerais por grupo.
- A entrada de handlers legados continua agregada e limitada. Não há promessa de cancelamento dentro de uma única chamada legada ou de comportamento incremental de todos os programas.
- O VFS é de texto UTF-8, com permissões e symlinks limitados. Pipes binários e append binário continuam rejeitados; saída binária existente exige `> arquivo_virtual`.
- `exit` em scripts encerra a invocação. O comando interativo de saída mantém o comportamento de sessão local/SSH do jogo; Ctrl+D fecha a janela local vazia.
- stderr de substituições é preservado; a saída final continua disponível para consumidores IPC antigos, além dos chunks incrementais.

## Evidência e limites de certificação

Os 50 casos de fundação usam expectativas declaradas e versionadas. Testes de Rust acrescentam entrada incompleta, spans, limites, fuzz determinístico, cancelamento, canais, contexto e persistência. Isso permite atribuir READY a capacidades delimitadas que passam na evidência atual; não certifica Bash nem coreutils completos. Veja o [relatório gerado](../generated/shell-compatibility.md).
