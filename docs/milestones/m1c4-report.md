# M1C.4 — VFS.EVENTS/WATCH e tail: certificação concluída

**PUBLICATION_STATUS: `PENDING_PUSH`.** A certificação técnica passou: o
pipeline derivou tail VERIFIED e VFS.EVENTS/WATCH READY. Os seis executáveis
anteriores permanecem VERIFIED. A publicação foi retomada após a recusa externa
anterior: o `git add` e a revisão do diff preparado passaram, sem alterar a
evidência técnica. M1C.5 não foi iniciado.

A recusa anterior foi `Automatic approval review failed: You've hit your usage
limit`, antes do `git add`. A retomada autorizada confirmou a integridade do
fingerprint e preparou somente os arquivos M1C.4. Resta criar o commit e fazer
push sem força. Os status abaixo continuam sendo resultados derivados, sem
promoção ou rebaixamento manual para representar publicação.

A entrega parte de `bbb1b925840dbb416b841cd31f178615d76f9a14`. A evidência foi executada em
2026-09-19T02:11:18.112Z. [Snapshot derivado](../coreutils/evidence/m1c4-final-state.json),
[arquitetura](../coreutils/MILESTONE-1C.4.md) e
[inventário gerado](../generated/coreutils-inventory.json) registram os detalhes.

## Resultados finais

| Verificação                        | Resultado                                                                                     |
| ---------------------------------- | --------------------------------------------------------------------------------------------- |
| Rust completo                      | 228 PASS, 0 FAIL, 7 testes DEV/benchmarks ignorados pela suíte padrão                         |
| Legacy                             | 76 casos exatos PASS, incluídos no Rust e reexecutados no fluxo global                        |
| Frontend                           | 214 PASS em 36 arquivos                                                                       |
| Tooling                            | 41 PASS, 0 FAIL; protocolo, provenance, hashes, fingerprints e fronteira de build             |
| GNU tail                           | 306 casos; captura com dois runs idênticos e verify independente                              |
| GNU Foundation                     | 142 casos: basename 48, dirname 27, printenv 36, whoami 31                                    |
| GNU cat / head                     | 91 / 215 casos                                                                                |
| GNU total atual                    | 754 referências; 754 matches obrigatórios                                                     |
| Compatibilidade declarativa global | 881 PASS, 0 FAIL, 0 SKIPPED                                                                   |
| Verificação normal / regressão     | PASS, sem regressão não aprovada                                                              |
| Strict dos sete comandos           | PASS individual: basename, dirname, printenv, whoami, cat, head, tail                         |
| Strict global                      | FAIL esperado pela dívida preservada: 1513 entradas (107 PARTIAL, 1406 UNVERIFIED)            |
| Prettier / ESLint / Stylelint      | PASS                                                                                          |
| TypeScript / conteúdo              | PASS; 12 missões, 11 threads, 8 hosts, 303 entradas Kali                                      |
| rustfmt / Clippy                   | PASS; Clippy all-targets/all-features com -D warnings                                         |
| Web                                | PASS; avisos de dependência Zod e chunk acima de 500 kB permanecem informativos               |
| Windows standalone                 | PASS; release com tauri/custom-protocol, assets de produção embutidos                         |
| Host Guard                         | PASS; nenhuma exceção nova                                                                    |
| Exclusão de tooling DEV            | PASS no teste negativo de build; sem marcadores do harness nos JS de produção e no executável |

O fluxo global também executou os capturadores DEV de registro, casos e benchmarks
VFS/shell/Coreutils, que são ignorados pelo cargo test padrão. Não foram necessários
novos testes visuais nem execução do próximo Coreutil.

## Certificação por executável

| Executável | BEFORE   | AFTER    | GNU | Matches | Contratos | Gates required |
| ---------- | -------- | -------- | --- | ------- | --------- | -------------- |
| basename   | VERIFIED | VERIFIED | 48  | 48      | 5/5       | 19/19          |
| dirname    | VERIFIED | VERIFIED | 27  | 27      | 5/5       | 19/19          |
| printenv   | VERIFIED | VERIFIED | 36  | 36      | 5/5       | 19/19          |
| whoami     | VERIFIED | VERIFIED | 31  | 31      | 5/5       | 19/19          |
| cat        | VERIFIED | VERIFIED | 91  | 91      | 9/9       | 26/26          |
| head       | VERIFIED | VERIFIED | 215 | 215     | 9/9       | 26/26          |
| tail       | PARTIAL  | VERIFIED | 306 | 306     | 11/11     | 26/26          |

Cada referência está em `tests/cli/gnu/coreutils/9.7/<comando>.json`, com hash de
binário, ambiente, harness, requests e bytes capturados. O snapshot final registra
os fingerprints individuais de execução de todos os casos e os hashes das
referências. Nenhum hash de captura antiga foi alterado manualmente.

A baseline continua Alpine 3.22.0/Coreutils 9.7-r1, locale C, x86_64 e rede
desativada. Foi acrescentado ao lock o vínculo de tail com o mesmo binário
multicall já usado pelos demais comandos. Isso mudou o hash do arquivo de lock;
por isso todos os 754 casos foram novamente capturados e verificados, sem
alterar os pacotes ou a versão da baseline.

## Tail

A matriz final tem 306 IDs únicos: 268 do gerador base, 27 do gerador follow e
11 direcionados. Todos passaram no schema. Expectativas foram adotadas pelo
gerador a partir da captura canônica GNU, seguida de verify independente.
Os oito casos antigos de tail foram substituídos pela matriz ampliada; seus
comportamentos continuam cobertos.

Os 11 contratos required possuem vínculos reais:

- **invocation:** 25 casos, PASS.
- **options:** 111 casos, PASS.
- **errors:** 138 casos, PASS.
- **help-version:** 13 casos, PASS.
- **integration:** 58 casos, PASS.
- **bytes:** 22 casos, PASS.
- **headers:** 26 casos, PASS.
- **tty:** 8 casos, PASS.
- **signals:** 42 casos, PASS.
- **follow:** 39 casos, PASS.
- **scheduler:** 39 casos, PASS.

Todas as 19 flags declaradas possuem evidência. Resultado final:
306 matches, zero diferenças obrigatórias remanescentes,
zero known required gaps e 26/26 gates PASS.
A comparação inclui stdout/stderr binários, status/sinal, término, observações
intermediárias, existência de caminhos, conteúdo, modos e relações de inode.

### Divergências realmente observadas nesta retomada

| Situação                                                                      | GNU 9.7                                                                                                | Cyber War antes / causa                                                                              | Correção e regressão                                                                                                                   |
| ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Follow por bytes, +N além do EOF inicial                                      | Com arquivo ab e -f -c+8, informa truncamento e emite ab antes dos appends seguintes                   | Limitava o seek ao tamanho inicial, perdendo os bytes e o diagnóstico                                | Preserva o offset solicitado no follow; directed-start-bytes e directed-start-binary, nova confirmação GNU e regressão Rust            |
| Truncate + append maior que o offset anterior, antes de executar o consumidor | Após old-data, -c2 e lote truncate(0)+9 bytes, emite somente o byte além do offset anterior, sem aviso | Usava o menor tamanho do evento coalescido para zerar o offset, emitindo nove bytes e aviso indevido | Compara tamanho atual com offset; evento VFS preserva o histórico para outros consumidores; directed-truncate-batch-9 e regressão Rust |

O primeiro strict revelou que o manifesto exigia os subsistemas gerais PROCESS, SIGNALS e TTY. Foram vinculadas as capacidades existentes PROCESS.LIFETIME, SIGNALS.STREAMS e TTY.CANONICAL_IO aos testes GNU de writers, sinais e TTY, preservando todos os requisitos VFS da família. Os subsistemas gerais permanecem PARTIAL, e um teste impede que os novos vínculos eliminem a dívida dos outros comandos. A captura global foi refeita após esse ajuste.

Também foram corrigidos problemas comprovados de infraestrutura: dispatch do
manual ainda retornava o help genérico, testes antigos esperavam rejeição de
follow, IDs com maiúscula violavam o schema, o vínculo de headers tinha typo e
o lock não continha tail. Nenhum desses ajustes mudou bytes GNU para fazer o
Cyber War passar. A suíte final confirma as correções.

## VFS.EVENTS / VFS.WATCH

- **VFS.EVENTS:** PARTIAL → READY; 39 requiredTests reais do protocolo follow.
- **VFS.WATCH:** PARTIAL → READY; 39 requiredTests reais do protocolo follow.

As declarações permanecem PARTIAL com política GNU_DIFFERENTIAL. O pipeline
calcula READY a partir dos testes atuais, comparação GNU, contratos, flags,
ausência de gaps required e Host Guard. Não há promoção manual de estado.
Os contratos/gates de integração e follow são os de tail, listados acima;
estas capacidades não inventam um conjunto separado de gates.

Além dos casos declarativos, os testes Rust atuais comprovam:

- Assinaturas por inode/caminho, hardlinks, writes sem relação, revisão e prevenção de perda
  de wakeup, substituição, duração de descritores e truncamento/append.
- Coalescing de 1000 mutações, determinismo, limites de watchers/timers,
  dependências de symlinks, componentes ausentes e permissões.
- Timers virtuais, isolamento entre mundos, integração de duas sessões reais,
  contexto de terminais e escrita concorrente com tail ativo.
- Cancelamento INT/TERM, falha de commit SQL, save/load e proteção contra publicar
  estado antigo depois de uma troca de mundo.
- Zero watchers, handles, timers, waits, sinais pendentes e processos tail órfãos
  nos cenários de limpeza aplicáveis; invariantes VFS conferidas na captura.

Limites: 1024 watchers por VFS, 1024 timers, retenção de sufixo de 4 MiB,
captura de 4 MiB (1024 bytes reservados ao diagnóstico) e arquivo binário de
32 MiB. Os limites de arquivo/retenção/captura foram exercitados em -1, exato e
+1. Saves não serializam watchers, handles, waits, timers ou sinais pendentes;
um mundo carregado recebe nova identidade. SHELL.JOBS e VFS.DEVICES continuam
PARTIAL. Follow e os timers deste contrato não dependem de GNU, relógios ou
watchers do host.

## Contagens e fingerprint

| Escopo             | BEFORE | AFTER |
| ------------------ | ------ | ----- |
| Coreutils VERIFIED | 6      | 7     |
| Coreutils PARTIAL  | 31     | 30    |
| Global VERIFIED    | 6      | 7     |
| Global PARTIAL     | 114    | 113   |
| Global UNVERIFIED  | 1406   | 1406  |

Fingerprint global da execução final:

`faa2b719ecde2a4585a4df05eced82f50e65dd39ae4916fb6fc00dce49f39096`

Inventário, dashboard, grafo de dependências, next-work, fila, contratos, gates
e contagens foram regenerados pelo pipeline real. O strict global continua
falhando pela dívida dos demais comandos; os requisitos não foram reduzidos.

## Próximo item calculado — não implementado

- **NEXT QUEUE ITEM:** coreutils/base64.
- **CLASSIFICATION:** SMALL_SHARED_EXTENSION.
- **COMPLEXITY:** 2/5.
- **DEPENDENCIES / blockers:** SHELL.CONTEXT; SHELL.EXPANSION; SHELL.GLOBBING; SHELL.LISTS; SHELL.PARSING; SHELL.PIPELINES; SHELL.REDIRECTION; VFS.DIRECTORIES; VFS.HARDLINKS; VFS.INODES; VFS.METADATA; VFS.PATHS; VFS.PERMISSIONS; VFS.REGULAR_FILES; VFS.SPECIAL_PERMISSIONS; VFS.SYMLINKS; VFS.TIMESTAMPS.
- **KNOWN GAPS:** Decode partial-output behavior and invalid-input diagnostics need full GNU audit.
- **ACTION:** NEEDS_IMPLEMENTATION_AND_EVIDENCE.

A classificação é produzida por waveDecision sobre a fila derivada, sem usar a
ordem textual do catálogo. O limite explícito desta entrega determina parar
após tail mesmo se o tooling permitir continuar a onda.

## Reprodutibilidade e publicação

As execuções usam o wrapper Rust local e o ambiente CyberWar-GNU97/bubblewrap.
Os comandos canônicos equivalentes estão nos scripts package.json e no CI:
GNU capture/verify, cli:test, cli:compat e cli:verify com strict por comando.
Logs locais ficam em artifacts/m1c4-* e não fazem parte da entrega. Probes
exploratórios, target, node_modules e o sandbox GNU também não entram no commit.

O diff, o stage e a formatação final passaram. O commit do milestone está pronto
para push sem força; a confirmação de publicação será registrada pelo histórico
Git após o remoto aceitar o commit. Não reexecutar suítes já aprovadas sem
alteração ou outro motivo concreto.

**PUBLICATION_STATUS: `PENDING_PUSH`.** Nenhum item posterior foi iniciado.
