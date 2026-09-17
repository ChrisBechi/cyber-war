# CYBER WAR — Milestone 1C.1

## Resultado

Fechamento vertical da Foundation Wave contra GNU Coreutils **9.7** real, sem alterar a baseline nem reduzir gates. Os quatro comandos alcançaram **VERIFIED** por derivação do pipeline. Coreutils como família permanece **PARTIAL**, com 33 outros executáveis PARTIAL e o bloqueio familiar de `SHELL.JOBS` associado a `env`.

## Estado inicial confirmado e resultado final

| Comando  | Antes → depois     | Casos projeto antes → depois | GNU antes → depois | Gates antes → depois |
| -------- | ------------------ | ---------------------------: | -----------------: | -------------------: |
| basename | PARTIAL → VERIFIED |                      12 → 48 |             0 → 48 |        13/19 → 19/19 |
| dirname  | PARTIAL → VERIFIED |                       6 → 27 |             0 → 27 |        13/19 → 19/19 |
| printenv | PARTIAL → VERIFIED |                       7 → 36 |             0 → 36 |        13/19 → 19/19 |
| whoami   | PARTIAL → VERIFIED |                       5 → 31 |             0 → 31 |        14/19 → 19/19 |

Fingerprint inicial: `59b4fb6df1b833adf7a81055b64e512b6ff56b83ae8e325fb2fab3a0b0bf3413`.
Fingerprint final: `d3e1080fbb7fac356ed0a59c462b4eeba260f3c0651948b965133187349da6f7`.

A fila inicial indicava `basename`. Após a primeira execução direcionada, a fila recalculada indicou `dirname`; em seguida foram fechados `printenv` e `whoami`. As execuções seguintes também repetiram os comandos Foundation já capturados e a evidência obrigatória de shell/VFS. Os estados intermediários ficaram nos artefatos locais `m1c1-after-*.json`.

## Ambiente GNU e reprodução

- Linux x86_64 dedicado em WSL2, distribuição `CyberWar-GNU97`, Alpine 3.22.0 e pacote GNU Coreutils **9.7-r1**, obtidos do repositório oficial Alpine.
- Rootfs, 34 APKs e os quatro binários GNU têm SHA-256 fixados no lock. A versão emitida por cada executável deve coincidir exatamente com o manifest. Os metadados dos pacotes e hashes reais estão nas capturas.
- `LC_ALL=C`; ambiente de processo explícito e ordenado; UID e passwd controlados. Os casos não dependem de timezone.
- Bubblewrap com namespaces isolados e rede desativada. Cada caso só escreve em uma fixture temporária; repositório e home do desenvolvedor não são montados no namespace do caso.
- Limites de CPU, memória, tempo, arquivos e tamanho de saída. Cada caso roda duas vezes e só é aceito se as duas execuções forem idênticas.
- As 142 capturas foram reproduzidas com `--verify`, que não escreve goldens. A chamada nativa no Windows foi rejeitada, conforme a proteção esperada.
- O caminho Docker/CI usa os mesmos arquivos fixados e cache validado por hash. O job Linux foi configurado; a execução local comprovada nesta entrega foi WSL2. A execução hospedada de CI ainda depende do push e não é declarada aprovada aqui.

Procedimento completo: [reference-environment.md](reference-environment.md). A obtenção de GNU é exclusiva de DEV/CI; o jogo Windows continua usando handlers Rust virtuais.

## Diferenças observadas e correções

| Área / caso                                          | GNU observado                                              | Diferença anterior e correção                                                                                                                                             |
| ---------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| basename/help, version, plain-help                   | Texto completo e nome de invocação em ajuda/diagnósticos   | O banner era reduzido e a invocação era descartada. Catálogo C-locale capturado e preservação de argv[0].                                                                 |
| basename/help, version no Windows                    | LF nos bytes da saída                                      | Arquivos de mensagens com CRLF divergiam. As mensagens estáticas usam LF, sem normalizar saída capturada.                                                                 |
| basename/permutation e posix-permutation             | Para no primeiro operando                                  | Parser anterior continuava reconhecendo opções. Regra específica de basename corrigida.                                                                                   |
| dirname/post-operand-option e posix-option           | Permuta opções, exceto com POSIXLY_CORRECT                 | A primeira implementação compartilhada parava cedo. Permutação corrigida e variável considerada somente quando exportada ao processo.                                     |
| basename/utf8-invalid-short e equivalentes           | stderr pode conter um byte isolado da sequência UTF-8      | O parser/transportador textual perderia esse byte. Diagnósticos e transporte mantêm stderr bruto; a conversão tolerante é só visual.                                      |
| Foundation/empty-long-name                           | Lista as opções possíveis no diagnóstico de ambiguidade    | O parser retornava somente opção desconhecida. Diagnóstico e ordem das alternativas corrigidos após probe GNU final.                                                      |
| basename/quote-controls, extra-quoted                | Escapes C para controles e bytes fora de ASCII             | Quoting e diagnóstico corrigidos conforme captura.                                                                                                                        |
| printenv/ordered-environment, ordered-null           | Mantém ordem do ambiente recebido                          | A implementação usava BTreeMap ordenado por chave. Agora há ordem do processo separada da consulta de valores; export, unset e prefixos temporários preservam essa ordem. |
| printenv/ignore-invalid-cluster                      | Retorna usage/status 2 ao processar -i                     | O parser intermediário tentava diagnosticar a opção seguinte. Precedência corrigida.                                                                                      |
| whoami/effective-name, different-uid, lookup-failure | Consulta nome pelo UID efetivo; status 1 se não encontrado | Antes devolvia o nome do ator. Agora consulta a base virtual de usuários/`/etc/passwd` e reproduz o erro de lookup.                                                       |

Os casos também cobrem caminhos vazios, slash/double-slash, pontos, Windows como texto lexical, Unicode, espaços/controles, sufixos, operandos com hífen, abreviações de opções, erros, NUL, mil operandos, valor de ambiente de 64 KiB, stdin ignorado, pipes e redirecionamentos.

## Lacunas da Foundation encerradas

- **basename:** lacuna inicial: Full GNU help/version banners and POSIXLY_CORRECT parsing remain unverified. Lacunas fechadas pelos contratos e casos `basename` listados abaixo. Nenhum blocker obrigatório permanece na baseline declarada.
- **dirname:** lacuna inicial: Platform-specific double-slash roots excluded by Linux baseline. Comportamento de slash/double-slash confirmado na baseline Linux; outras plataformas continuam fora do escopo original, sem alegação de certificação. Nenhum blocker obrigatório permanece na baseline declarada.
- **printenv:** lacuna inicial: Environment listing uses deterministic key order; GNU insertion order is not modeled. Lacunas fechadas pelos contratos e casos `printenv` listados abaixo. Nenhum blocker obrigatório permanece na baseline declarada.
- **whoami:** lacuna inicial: User database lookup failures are not modeled. Lacunas fechadas pelos contratos e casos `whoami` listados abaixo. Nenhum blocker obrigatório permanece na baseline declarada.

O escopo continua Linux x86_64/musl, locale C, argv UTF-8 e recursos virtuais limitados. NUL em argv não é um argumento de processo válido. Não há certificação de outros locales, NSS do host ou plataformas/libcs não capturadas. Essas restrições são explícitas; nenhum gate obrigatório foi convertido em opcional para obter promoção.

## Infraestrutura e integridade

- Reutilizado `scripts/cli/coreutils-reference.py`, com prepare/run em `coreutils-environment.py`, captura por comando/wave, probe separado e substituição explícita.
- Comparação independente dos bytes stdout/stderr, status, digest da requisição e efeitos de filesystem. Alterar só a expectativa declarada não aprova uma divergência com GNU.
- Gates recusam baseline, provenance, hash de harness/ambiente/binário, requisição ou captura desatualizados. Casos de script/TTY não recebem evidência estruturada indevida.
- O runner registra fingerprints antes de executar e recusa alterações nos fontes durante a captura. Execução direcionada só conserva casos anteriores ainda válidos; não renova performance antiga.
- Fingerprints de casos diretos excluem handlers e mensagens de outros comandos; alterações compartilhadas em shell, VFS e parser invalidam seus dependentes. Contratos, flags e fixtures do comando continuam vinculados.
- `stderrHex` é comparado de ponta a ponta. Um teste de redireção comprova preservação de stderr inválido em UTF-8 no VFS.
- As antigas integrações `export SAMPLE=value; /usr/bin/printenv SAMPLE` e `sudo whoami` permanecem em testes Rust, com os mesmos resultados exatos. Os casos GNU usam contexto de processo estruturado; o harness não executa scripts arbitrários.
- O teste de `env` que invoca `basename --version` foi atualizado para o banner GNU completo. `env` permanece PARTIAL e não recebeu certificação indireta.

## Validação final

| Verificação                                          | Resultado                                                                                     |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Rust                                                 | 192 testes PASS; 7 testes DEV ignorados por padrão e acionados quando aplicáveis pelo tooling |
| Frontend                                             | 214 testes PASS em 36 arquivos                                                                |
| Tooling                                              | 31 testes PASS                                                                                |
| Compatibilidade declarativa                          | 298 PASS; zero FAIL/SKIPPED                                                                   |
| Legados                                              | 76 casos exatos PASS, sem normalização de saída                                               |
| GNU                                                  | 142 casos; duas execuções idênticas por captura; reprodução verify PASS                       |
| Inventory / verify                                   | PASS; sem erro de integridade                                                                 |
| Strict Foundation                                    | PASS para os quatro comandos                                                                  |
| Strict global                                        | FAIL esperado: 1516 pendências; não foi tratado como sucesso global                           |
| Prettier / ESLint / Stylelint / TypeScript / content | PASS                                                                                          |
| rustfmt / Clippy -D warnings                         | PASS                                                                                          |
| Build web                                            | PASS; aviso existente de chunk maior que 500 KiB                                              |
| Build Windows release                                | PASS; executável standalone; sem gerar instaladores MSI/NSIS nesta validação                  |
| Host guard                                           | PASS; nenhum executor GNU/WSL/Docker no gameplay                                              |

Shell mantém **7 capacidades READY**; VFS mantém **11 capacidades READY**. `SHELL.JOBS` e `VFS.DEVICES` continuam PARTIAL.

Executável: `src-tauri/target/release/game-hacker.exe`; 9,339,904 bytes; SHA-256 `3699254fe2c62af861e6ecfd6f54f34c54ceec31fe8358bcd1f83c1cce265ca3`.

Logs e capturas de execução do projeto estão em `artifacts/m1c1-*.log` e `artifacts/cli-*.json` (locais, ignorados no Git). Os goldens GNU pequenos, contratos e relatórios gerados são versionados. `.tools`, WSL/rootfs/APKs, target, node_modules e builds não entram no commit.

## Evidências por comando

### basename — VERIFIED

GNU: [captura](../../tests/cli/gnu/coreutils/9.7/basename.json). Implementação: `src-tauri/src/coreutils/foundation.rs`, parser e catálogo de mensagens Foundation.

Gates obrigatórios PASS: `COMBINED_FLAGS`, `COMMAND_CONTRACTS`, `DISCOVERY`, `ERRORS`, `EXIT_CODE`, `GNU_REFERENCE`, `HELP`, `HOST_ISOLATION`, `KNOWN_GAPS`, `LONG_FLAGS`, `PARSER`, `PIPE`, `POSITIONAL_ARGS`, `REDIRECTION`, `REFERENCE_PINNED`, `SHORT_FLAGS`, `STDERR`, `STDOUT`, `VERSION`.

Evidence IDs:

- `coreutils/basename/suffix`
- `coreutils/basename/root-empty`
- `coreutils/basename/lexical`
- `coreutils/basename/suffix-equal`
- `coreutils/basename/multiple-zero`
- `coreutils/basename/long-suffix`
- `coreutils/basename/dash`
- `coreutils/basename/missing`
- `coreutils/basename/extra`
- `coreutils/basename/version`
- `coreutils/basename/unknown-option`
- `coreutils/basename/option-value-help`
- `coreutils/basename/help`
- `coreutils/basename/path-matrix`
- `coreutils/basename/long-multiple-zero`
- `coreutils/basename/suffix-matrix`
- `coreutils/basename/empty-suffix`
- `coreutils/basename/slash-suffix`
- `coreutils/basename/permutation`
- `coreutils/basename/posix-permutation`
- `coreutils/basename/posix-help-operand`
- `coreutils/basename/abbreviated`
- `coreutils/basename/help-value-cluster`
- `coreutils/basename/suffix-value-long-option`
- `coreutils/basename/invalid-before-help`
- `coreutils/basename/help-before-invalid`
- `coreutils/basename/missing-suffix`
- `coreutils/basename/missing-long-suffix`
- `coreutils/basename/invalid-short`
- `coreutils/basename/unexpected-long-value`
- `coreutils/basename/help-argument`
- `coreutils/basename/extra-quoted`
- `coreutils/basename/dash-help`
- `coreutils/basename/stdin-ignored`
- `coreutils/basename/pipe-zero`
- `coreutils/basename/redirect-zero`
- `coreutils/basename/plain-help`
- `coreutils/basename/plain-error`
- `coreutils/basename/utf8-invalid-short`
- `coreutils/basename/utf8-operands`
- `coreutils/basename/quote-controls`
- `coreutils/basename/suffix-utf8`
- `coreutils/basename/large-operands`
- `coreutils/basename/empty-long-name`
- `coreutils/basename/quote-0`
- `coreutils/basename/quote-1`
- `coreutils/basename/quote-2`
- `coreutils/basename/quote-3`

### dirname — VERIFIED

GNU: [captura](../../tests/cli/gnu/coreutils/9.7/dirname.json). Implementação: `src-tauri/src/coreutils/foundation.rs`, parser e catálogo de mensagens Foundation.

Gates obrigatórios PASS: `COMBINED_FLAGS`, `COMMAND_CONTRACTS`, `DISCOVERY`, `ERRORS`, `EXIT_CODE`, `GNU_REFERENCE`, `HELP`, `HOST_ISOLATION`, `KNOWN_GAPS`, `LONG_FLAGS`, `PARSER`, `PIPE`, `POSITIONAL_ARGS`, `REDIRECTION`, `REFERENCE_PINNED`, `SHORT_FLAGS`, `STDERR`, `STDOUT`, `VERSION`.

Evidence IDs:

- `coreutils/dirname/paths`
- `coreutils/dirname/zero`
- `coreutils/dirname/dash`
- `coreutils/dirname/missing`
- `coreutils/dirname/version`
- `coreutils/dirname/unknown-option`
- `coreutils/dirname/help`
- `coreutils/dirname/plain-help`
- `coreutils/dirname/plain-missing`
- `coreutils/dirname/lexical-matrix`
- `coreutils/dirname/long-zero`
- `coreutils/dirname/combined-zero`
- `coreutils/dirname/abbreviations`
- `coreutils/dirname/post-operand-option`
- `coreutils/dirname/posix-option`
- `coreutils/dirname/posix-dash`
- `coreutils/dirname/utf8-invalid-short`
- `coreutils/dirname/invalid-short`
- `coreutils/dirname/help-before-invalid`
- `coreutils/dirname/invalid-before-help`
- `coreutils/dirname/value-not-allowed`
- `coreutils/dirname/help-value`
- `coreutils/dirname/stdin-ignored`
- `coreutils/dirname/pipe-zero`
- `coreutils/dirname/redirect-zero`
- `coreutils/dirname/large-operands`
- `coreutils/dirname/empty-long-name`

### printenv — VERIFIED

GNU: [captura](../../tests/cli/gnu/coreutils/9.7/printenv.json). Implementação: `src-tauri/src/coreutils/foundation.rs`, parser e catálogo de mensagens Foundation.

Gates obrigatórios PASS: `COMBINED_FLAGS`, `COMMAND_CONTRACTS`, `DISCOVERY`, `ERRORS`, `EXIT_CODE`, `GNU_REFERENCE`, `HELP`, `HOST_ISOLATION`, `KNOWN_GAPS`, `LONG_FLAGS`, `PARSER`, `PIPE`, `POSITIONAL_ARGS`, `REDIRECTION`, `REFERENCE_PINNED`, `SHORT_FLAGS`, `STDERR`, `STDOUT`, `VERSION`.

Evidence IDs:

- `coreutils/printenv/missing`
- `coreutils/printenv/multi`
- `coreutils/printenv/null`
- `coreutils/printenv/export`
- `coreutils/printenv/version`
- `coreutils/printenv/unknown-option`
- `coreutils/printenv/stop-at-operand`
- `coreutils/printenv/help`
- `coreutils/printenv/default-environment`
- `coreutils/printenv/plain-help`
- `coreutils/printenv/ordered-environment`
- `coreutils/printenv/ordered-null`
- `coreutils/printenv/empty-and-absent`
- `coreutils/printenv/values`
- `coreutils/printenv/equals-name`
- `coreutils/printenv/end-options`
- `coreutils/printenv/operand-help`
- `coreutils/printenv/posix-options`
- `coreutils/printenv/minimal-environment`
- `coreutils/printenv/duplicate-names`
- `coreutils/printenv/combined-null`
- `coreutils/printenv/invalid-short`
- `coreutils/printenv/invalid-utf8`
- `coreutils/printenv/unsupported-ignore`
- `coreutils/printenv/unsupported-unset`
- `coreutils/printenv/missing-unset`
- `coreutils/printenv/ignore-invalid-cluster`
- `coreutils/printenv/unexpected-value`
- `coreutils/printenv/abbreviations`
- `coreutils/printenv/help-before-invalid`
- `coreutils/printenv/invalid-before-help`
- `coreutils/printenv/stdin-ignored`
- `coreutils/printenv/pipe-null`
- `coreutils/printenv/redirect-null`
- `coreutils/printenv/large-value`
- `coreutils/printenv/empty-long-name`

### whoami — VERIFIED

GNU: [captura](../../tests/cli/gnu/coreutils/9.7/whoami.json). Implementação: `src-tauri/src/coreutils/foundation.rs`, parser e catálogo de mensagens Foundation.

Gates obrigatórios PASS: `COMBINED_FLAGS`, `COMMAND_CONTRACTS`, `DISCOVERY`, `ERRORS`, `EXIT_CODE`, `GNU_REFERENCE`, `HELP`, `HOST_ISOLATION`, `KNOWN_GAPS`, `LONG_FLAGS`, `PARSER`, `PIPE`, `POSITIONAL_ARGS`, `REDIRECTION`, `REFERENCE_PINNED`, `SHORT_FLAGS`, `STDERR`, `STDOUT`, `VERSION`.

Evidence IDs:

- `coreutils/whoami/user`
- `coreutils/whoami/root`
- `coreutils/whoami/extra`
- `coreutils/whoami/version`
- `coreutils/whoami/unknown-option`
- `coreutils/whoami/help`
- `coreutils/whoami/plain-help`
- `coreutils/whoami/effective-name`
- `coreutils/whoami/different-uid`
- `coreutils/whoami/lookup-failure`
- `coreutils/whoami/spoofed-environment`
- `coreutils/whoami/empty-operand`
- `coreutils/whoami/dash-operand`
- `coreutils/whoami/extra-quoted`
- `coreutils/whoami/invalid-short`
- `coreutils/whoami/invalid-utf8`
- `coreutils/whoami/help-before-invalid`
- `coreutils/whoami/invalid-before-help`
- `coreutils/whoami/extra-before-help`
- `coreutils/whoami/posix-extra-before-help`
- `coreutils/whoami/help-argument`
- `coreutils/whoami/abbreviated-help`
- `coreutils/whoami/combined-options`
- `coreutils/whoami/stdin-ignored`
- `coreutils/whoami/pipe`
- `coreutils/whoami/redirect`
- `coreutils/whoami/empty-long-name`
- `coreutils/whoami/quote-0`
- `coreutils/whoami/quote-1`
- `coreutils/whoami/quote-2`
- `coreutils/whoami/quote-3`

## Próximo trabalho calculado

A fila final aponta **coreutils/cat**, ação **NEEDS_IMPLEMENTATION_AND_EVIDENCE**. Blockers: GATE:COMMAND_CONTRACTS; GATE:CROSS_TOOL_CONSISTENCY; GATE:GNU_REFERENCE; GATE:HELP; GATE:KNOWN_GAPS; GATE:SIGNALS; GATE:STATE_CONSISTENCY; GATE:TTY; GAP:Interactive transformed input buffers to EOF; GAP:self-output detection and GNU quoting need evidence.

A recomendação segue o [relatório Coreutils gerado](../generated/coreutils-compatibility.md#executable-queue). O agregado familiar ainda exige `SHELL.JOBS` por causa de `env`; isso não bloqueia executáveis independentes. Nenhuma outra wave foi iniciada neste milestone.
