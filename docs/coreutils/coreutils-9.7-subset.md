# GNU Coreutils 9.7 — subset virtual

A versão vem do manifest. O contrato individual e a lista de flags estão em `content/cli-compatibility/coreutils.json`; o [relatório gerado](../generated/coreutils-compatibility.md) apresenta todos os executáveis, waves, evidências e blockers.

## Trabalhos realizados

- **Fundamentos:** basename lexical, suffix e múltiplos operandos; dirname lexical; printenv com nomes ausentes e NUL; whoami com usuário virtual efetivo; env com ambiente temporário, limpeza/unset e dispatch virtual.
- **Leitura:** cat binário e flags de apresentação; head/tail com limites decimais de linhas ou bytes, cabeçalhos e múltiplos arquivos; tee com append e erros parciais; wc conta bytes/newlines de conteúdo binário; Base64 opera bytes; SHA-256 calcula digest e verifica listas básicas.
- **Integração:** execução por caminho absoluto, disponibilidade do pacote, pipes binários, redireção, append, inode compartilhado e cancelamento de produtor. Comandos de filesystem existentes não receberam certificação completa por herdar as primitives M1B.

## Convenções

Locale determinístico C para comparação; C.UTF-8 permanece disponível nos handlers que já o suportam. NUL e CR/LF não são normalizados. End-of-options segue o parser de cada comando. A Foundation Wave usa os textos observáveis de ajuda e versão capturados do GNU, incluindo o nome de invocação. Os demais comandos conservam seus banners e limitações anteriores. Consulte o [ambiente de referência](reference-environment.md) e o status derivado de cada executável.

Arquivos, identidades, grupos, rede e relógio são virtuais. `df`/`du` conservam semântica lógica incompleta; não são prova de alocação ext4. `date`, `id`, `groups`, `seq`, `cut`, `tr` e as ferramentas avançadas mantêm lacunas individuais no manifest.

## Limites que bloqueiam certificação

Nenhuma flag declarada pode ser certificada sem evidência explícita. Capturas de outra versão não aprovam GNU 9.7. Scripts de integração e contratos com side effects ainda precisam de harness de referência ampliado quando exigirem equivalência GNU.

Full GNU help/version, localização gettext, streams infinitos fora dos engines cooperativos, follow de tail, signals POSIX completos, mounts, dispositivos avançados, byte semantics de sort/uniq/cut/tr e várias matrizes de flags de cp/mv/rm/chmod/chown continuam incompletos. Não foram ocultados como funcionalidades verificadas.

## Fontes de auditoria

- [Manuais Debian da baseline Coreutils 9.7](https://manpages.debian.org/trixie/coreutils/).
- [printenv no tag GNU v9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/printenv.c): status de erros e separação das variáveis ausentes.
- [env no tag GNU v9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/env.c): opções e execução em ambiente modificado.
- [cat 9.7](https://manpages.debian.org/trixie/coreutils/cat.1.en.html) e [base64 9.7](https://manpages.debian.org/trixie/coreutils/base64.1.en.html): contratos de bytes e opções.

As fontes documentam comportamento. A certificação exige as capturas adicionais indicadas pelo pipeline.
