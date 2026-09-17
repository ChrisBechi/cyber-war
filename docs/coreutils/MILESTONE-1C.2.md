# CYBER WAR — Milestone 1C.2 — Cat

## Resultado

**Cat VERIFIED**, derivado do pipeline: **26/26 gates**, 9/9 contratos e **91 casos GNU 9.7** correspondentes. Zero lacunas obrigatórias e nenhum waiver novo. Foundation conserva os quatro executáveis VERIFIED. A família Coreutils continua PARTIAL, com 5 executáveis VERIFIED e 32 PARTIAL.

A captura global final tem 376 casos PASS, zero FAIL/SKIPPED e fingerprint atual. Fontes, casos, contratos e hashes foram conferidos pelo pipeline; não houve alteração manual de status.

## Auditoria inicial e mudanças

| Item                   | Antes (M1C.1)                    | Implementado em M1C.2                                             |
| ---------------------- | -------------------------------- | ----------------------------------------------------------------- |
| Cat                    | PARTIAL; 13 casos; 18/26 gates   | 91 casos GNU e projeto; 26/26 gates; VERIFIED                     |
| Streaming transformado | Acumulava até EOF                | Estado incremental para todas as opções                           |
| CR/LF entre chunks     | Divergência reproduzida          | CR pendente preservado entre chunks e operandos                   |
| Interação              | Modelo de input/EOF insuficiente | TTY canônico com Pending, Data, EOF e Interrupted                 |
| Sinais                 | Cancelamento geral               | Estado por processo, SIGINT/SIGTERM/SIGPIPE e término explícito   |
| Auto-saída             | Sem evidência por inode          | Casos real GNU de mesmo inode, hardlink e symlink                 |
| Ajuda e versão         | Mensagens de subset              | Bytes GNU 9.7, invocação e precedência verificadas                |
| Foundation             | 4 VERIFIED; 142 casos GNU        | 142 referências recapturadas/reproduzidas; 4 VERIFIED preservados |

A auditoria BEFORE está em `artifacts/m1c2-before.json`; fingerprint inicial:
`d3e1080fbb7fac356ed0a59c462b4eeba260f3c0651948b965133187349da6f7`.

Dois testes reproduziram falhas antes da implementação: cat -E com CR/LF separado entre chunks, e cat -n sem emissão antes de EOF. O log está em `artifacts/m1c2-before-regressions.log`. Ambos passaram após a correção.

## Implementação virtual

`coreutils/cat.rs` conserva número da linha, início de linha, linha vazia anterior e CR pendente. As transformações operam bytes, incluindo NUL e 0–255. Todas as flags documentadas, aliases, opções longas abreviadas, opções inválidas, `--`, POSIXLY_CORRECT, ajuda e versão passam pelo parser compartilhado. -b prevalece sobre -n. O quoter de nomes foi acrescentado ao módulo compartilhado de opções.

O scheduler abre cada operando quando necessário e lê handles VFS em blocos de 4 KiB. O estado continua entre arquivos e stdin. Pipes têm capacidade de 64 KiB; um consumidor fechado produz SIGPIPE. Descritores são fechados tanto no término normal quanto em erro ou sinal. Transformações não acumulam o arquivo inteiro no handler; o armazenamento VFS, cache e resultado agregado continuam sujeitos aos limites do jogo.

O teste de propriedade compara 64 combinações de flags com chunks de 1, 2, 3, 7, 31, 255 e 4096 bytes, incluindo todos os bytes, dados pseudoaleatórios determinísticos e CR/LF. A integração Rust cobre vários cat em pipeline, redireção, contexto de pacote e limpeza de processos/handles.

## TTY

`VirtualTty` distingue ausência de entrada de EOF. Newline libera uma linha; Ctrl+D com texto libera a linha parcial e mantém a entrada aberta; Ctrl+D vazio gera um evento EOF. Outro operando `-` pode ler novamente. A fila é limitada a 64 KiB. Eco e edição permanecem na interface e não entram no stdout de cat. O frontend serializa os envios de texto e EOF; interrupções ignoram essa fila para acordar o comando.

O escopo não inclui termios, modos raw, job control completo, controle de foreground POSIX ou PTY do host dentro do jogo. A captura DEV usa PTY canônico real com echo desligado e stdout/stderr separados. Os casos interativos observam saída antes de EOF, numeração, squeeze, EOF parcial e stdin repetido.

## Sinais e término

Cada PID virtual tem seu próprio `ProcessSignalState`. SIGINT, SIGTERM e SIGPIPE usam a disposição padrão terminante. `Termination::Exit { code }` é distinto de `Termination::Signal { signal }`; o status shell é 128 + sinal (130, 143, 141). A UI converte Ctrl+C em interrupção virtual. Nenhum sinal do host é usado em gameplay.

Entrega por PID não cancela irmãos. Leituras pendentes, pipes e espera por capacidade de saída consultam sinais e acordam. A emissão para a interface reserva um chunk antes de contabilizar seus bytes no resultado. Um teste interrompe cat após entregar exatamente 64 KiB: confirma os bytes entregues, status 143, término SIGTERM e ausência de processos/handles remanescentes.

Estado de execução é transitório. Filas, sinais e handles não são serializados como processos pendentes; o VFS e os efeitos confirmados seguem o mecanismo de save existente. SHELL.JOBS e VFS.DEVICES não receberam promoção.

## Diferenças GNU encontradas e fechadas

- CR/LF dividido entre chunks e operandos: estado explícito de CR pendente.
- Transformações interativas: emissão incremental antes de EOF, comprovada por barreiras de stdout.
- EOF parcial: entrega de texto sem encerrar permanentemente stdin; EOF vazio consumível.
- Mensagens, opções e quoting: ajuda/versão capturadas, precedência e abreviações verificadas, diagnósticos de bytes UTF-8 inválidos preservados.
- Loop de symlink: a baseline Linux/musl retorna `Symbolic link loop`; o VFS interno tinha `Too many levels of symbolic links`. Cat traduz o diagnóstico observável. O caso existente `vfs/symlink/loop` agora exige a mensagem exata correspondente.
- Auto-saída: identidade de inode com offset/tamanho; truncamento ocorre antes da leitura.
- Backpressure interrompido: o resultado não contabiliza o chunk que deixou de ser entregue.

O último comparativo direto (`artifacts/m1c2-final-cat-differences.log`) registrou **91 casos, zero diferenças** em bytes, estado, observações e término. O pipeline completo posterior renovou as capturas e confirmou as mesmas comparações, incluindo o registro byte-exact do diagnóstico de limite de saída.

## Auto-saída e estado VFS

| Cenário GNU e virtual            | Resultado                                                 |
| -------------------------------- | --------------------------------------------------------- |
| `cat a > a`                      | Trunca antes da leitura; saída vazia; status 0            |
| `cat a >> a`, a não vazio        | Diagnóstico `input file is output file`; status 1         |
| Redireção para hardlink do input | Mesmo comportamento por inode                             |
| Redireção para symlink do input  | Mesmo comportamento após resolução                        |
| `cat < a >> a`                   | Mesmo teste de identidade para stdin redirecionado        |
| Erro entre operandos válidos     | Preserva saída válida, continua e retorna status não zero |

As comparações incluem conteúdo binário, tipo, mode, nlink, destino de symlink e relações de identidade. Casos de estado também verificam ausência de processos remanescentes e preservação de usuário/cwd. Timestamps lógicos VFS não são comparados com tempo de parede do GNU.

## Evidência e isolamento

Foi evoluído o harness existente, mantendo o lock Alpine 3.22.0, GNU 9.7-r1, musl e Linux x86_64. Cat usa schema 3, interação versionada e hash do helper; Foundation permanece schema 2. Hash SHA-256 do binário cat:
`8c3d5024b3ea24ef924e50b3b1647bf1623fccacd56cca1e45a4395b6b3bc600`.

Cada captura é executada duas vezes e precisa ser idêntica. Os 91 casos cat e 142 Foundation foram capturados e reproduzidos em modo verify sem atualização dos goldens. Os cenários de sinal sincronizam em stdout/leitura bloqueada; não usam sleeps arbitrários de inicialização. O consumidor fechado é um pipe real apenas dentro do namespace DEV.

GNU, Python, WSL e bubblewrap são usados exclusivamente em DEV/CI, com rede desabilitada e fixtures temporárias. Gameplay usa Rust, VFS e processos virtuais. Host guard PASS na última captura direcionada e nos testes de tooling. Docker/CI remoto não foram executados localmente; estão configurados para reproduzir Foundation e cat.

Os testes de tooling rejeitam helper desatualizado, descritor pipe usado como evidência de PTY, término normal confundido com sinal, observações intermediárias ausentes e digest de requisição alterado.

## Validação final

| Verificação                                          | Resultado                                                                              |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Rust                                                 | 199 PASS; 7 testes DEV ignorados por padrão; teste final de diagnóstico de limite PASS |
| Frontend                                             | 214 PASS em 36 arquivos                                                                |
| Tooling                                              | 32 PASS                                                                                |
| Casos declarativos                                   | 376 PASS; zero FAIL/SKIPPED                                                            |
| Legados                                              | 76 casos exatos PASS, sem normalização                                                 |
| GNU cat                                              | 91 casos; duas execuções idênticas; verify PASS                                        |
| GNU Foundation                                       | 142 casos; duas execuções idênticas; verify PASS                                       |
| Inventory / verify                                   | PASS                                                                                   |
| Strict cat e Foundation                              | PASS                                                                                   |
| Strict global                                        | FAIL esperado: 1515 pendências; sem alegação de sucesso global                         |
| Prettier / ESLint / Stylelint / TypeScript / content | PASS                                                                                   |
| Clippy / rustfmt                                     | PASS; all-targets/all-features com -D warnings                                         |
| Build web                                            | PASS; aviso existente de chunk maior que 500 KiB                                       |
| Build Windows release                                | PASS; standalone com custom-protocol e frontend atualizado; instaladores não gerados   |
| Host guard                                           | PASS                                                                                   |

Fingerprint final:

`c5d9fd0f96eedc7f4af658dcc226e3cfa0fa81fd76f238599e5aa80fc5fb4caf`

Captura: 2026-09-17T22:01:33.334Z. Logs detalhados ficam em `artifacts/m1c2-*` (locais, ignorados pelo Git); fixtures e referências GNU reproduzíveis ficam em `tests/cli/`. A compilação release não exige GNU/WSL/Docker no computador do jogador.

## Gates finais

| Gate obrigatório       | Resultado |
| ---------------------- | --------- |
| COMBINED_FLAGS         | PASS      |
| COMMAND_CONTRACTS      | PASS      |
| CROSS_TOOL_CONSISTENCY | PASS      |
| DISCOVERY              | PASS      |
| ERRORS                 | PASS      |
| EXIT_CODE              | PASS      |
| GNU_REFERENCE          | PASS      |
| HELP                   | PASS      |
| HOST_ISOLATION         | PASS      |
| KNOWN_GAPS             | PASS      |
| LONG_FLAGS             | PASS      |
| PARSER                 | PASS      |
| PERMISSIONS            | PASS      |
| PIPE                   | PASS      |
| POSITIONAL_ARGS        | PASS      |
| REDIRECTION            | PASS      |
| REFERENCE_PINNED       | PASS      |
| SHORT_FLAGS            | PASS      |
| SIDE_EFFECTS           | PASS      |
| SIGNALS                | PASS      |
| STATE_CONSISTENCY      | PASS      |
| STDERR                 | PASS      |
| STDIN                  | PASS      |
| STDOUT                 | PASS      |
| TTY                    | PASS      |
| VERSION                | PASS      |

## Regressão dos subsistemas

Shell conserva **7 capacidades READY** e VFS conserva **11 capacidades READY**. SHELL.JOBS=PARTIAL; VFS.DEVICES=PARTIAL. A infraestrutura mínima de cat não promoveu esses subsistemas inteiros.

| Foundation | Casos GNU | Gates | Status   |
| ---------- | --------- | ----- | -------- |
| basename   | 48        | 19/19 | VERIFIED |
| printenv   | 36        | 19/19 | VERIFIED |
| whoami     | 31        | 19/19 | VERIFIED |
| dirname    | 27        | 19/19 | VERIFIED |

## Desempenho e limites

Medições locais em debug, uma amostra por operação, sem comparação de velocidade com GNU:

- `cat large | wc -c`: 1000000 bytes, 252.43 ms.

O benchmark compartilhado registrou pico de 4096 bytes em pipe, com capacidade de 65536. Os limites de saída agregada (4 MiB com reserva para diagnóstico), input/pipe (64 KiB), arquivos binários (32 MiB), VFS e cache permanecem explícitos. O handler incremental não implica streams ilimitados em todos os comandos nem memória total do mundo constante.

## Próximo trabalho — fila calculada

Resultado exato de `coreutilsReport.next`, também disponível em `docs/generated/coreutils-inventory.json`:

```json
{
  "command": "head",
  "aliases": [],
  "baseline": "9.7",
  "implementation": "src-tauri/src/coreutils/bytes.rs",
  "implementationKind": "NATIVE",
  "capabilities": ["VFS_READ", "VFS_WRITE", "STDIN", "STDOUT", "STDERR"],
  "flags": ["-q", "-v", "-n", "-c", "--quiet", "--silent", "--verbose", "--lines", "--bytes"],
  "contracts": [
    {
      "id": "invocation",
      "applicability": "REQUIRED",
      "description": "Decimal line/byte limits, multiple headers and negative counts",
      "evidence": ["coreutils/head/lines"],
      "result": "PASS"
    },
    {
      "id": "options",
      "applicability": "REQUIRED",
      "description": "Supported options, combinations, --, unknown options and required operands",
      "evidence": [
        "coreutils/head/binary",
        "coreutils/head/negative-or-start",
        "coreutils/head/headers",
        "coreutils/head/unknown-option"
      ],
      "result": "PASS"
    },
    {
      "id": "errors",
      "applicability": "REQUIRED",
      "description": "Separate stderr, exact status, partial success and operand order",
      "evidence": ["coreutils/head/invalid", "coreutils/head/unknown-option"],
      "result": "PASS"
    },
    {
      "id": "help-version",
      "applicability": "REQUIRED",
      "description": "Truthful subset help and manifest baseline; full GNU banners remain a documented deviation",
      "evidence": ["coreutils/head/version"],
      "result": "PASS"
    },
    {
      "id": "integration",
      "applicability": "REQUIRED",
      "description": "Explicit packaged executable, shell argv, streams and host isolation",
      "evidence": [],
      "result": "NO_EVIDENCE"
    },
    {
      "id": "bytes",
      "applicability": "REQUIRED",
      "description": "NUL, invalid UTF-8, empty and bounded large input; no lossy semantic conversion",
      "evidence": ["coreutils/head/binary"],
      "result": "PASS"
    }
  ],
  "wave": 2,
  "area": "reading",
  "complexity": 2,
  "missionUses": [],
  "before": "PARTIAL",
  "after": "PARTIAL",
  "projectCases": 8,
  "referenceCases": 0,
  "referenceState": "SKIPPED",
  "matchedReferenceCases": 0,
  "requiredGates": 26,
  "passedGates": 14,
  "blockers": [
    "GATE:COMMAND_CONTRACTS",
    "GATE:CROSS_TOOL_CONSISTENCY",
    "GATE:GNU_REFERENCE",
    "GATE:HELP",
    "GATE:KNOWN_GAPS",
    "GATE:PERMISSIONS",
    "GATE:PIPE",
    "GATE:REDIRECTION",
    "GATE:SIDE_EFFECTS",
    "GATE:SIGNALS",
    "GATE:STATE_CONSISTENCY",
    "GATE:TTY",
    "GAP:Count multipliers and NUL delimiter unsupported",
    "GAP:bounded byte mode is not incremental"
  ],
  "action": "NEEDS_IMPLEMENTATION_AND_EVIDENCE"
}
```

A fila foi recalculada após a certificação; nenhum comando alternativo foi escolhido manualmente. Outros Coreutils conservaram suas lacunas e gates.
