# Milestone 1C.3 — Coreutils: leitura e seleção incremental

## Resultado da wave

- Executáveis processados verticalmente: **1**, `head`.
- Resultado: **1 VERIFIED**, **0 PARTIAL** entre os processados.
- Coreutils: **6 VERIFIED / 31 PARTIAL**; a família continua PARTIAL.
- Parada: **ARCHITECTURAL_BOUNDARY**, antes do limite de 12 executáveis.
- Próximo item exato calculado: **coreutils/tail**. Foi inspecionado para a
  decisão de parada; não foi processado verticalmente nem promovido.

`head` passou com **215/215 casos GNU, 26/26 gates e 9/9 contratos**.
Foundation e `cat` continuam certificados. Não existe promoção da wave inteira.

## Estado inicial verificado

- Commit: `ed9d30483e68e072fa7aa9410dcda2393f91b071` (`main`, M1C.2 publicado).
- Fingerprint: `c5d9fd0f96eedc7f4af658dcc226e3cfa0fa81fd76f238599e5aa80fc5fb4caf`.
- Coreutils: 5 VERIFIED, 32 PARTIAL; família PARTIAL.
- `basename`, `dirname`, `printenv`, `whoami` e `cat` certificados.
- `head`: primeiro da fila, 8 casos de projeto, 0 casos GNU, 14/26 gates.
- Global: 5 VERIFIED, 115 PARTIAL, 1406 UNVERIFIED; 376 casos declarativos PASS.
- Shell: 7 capacidades READY; VFS: 11 READY. JOBS e DEVICES permaneciam PARTIAL.

## Implementação de head

`src-tauri/src/coreutils/head.rs` implementa seleção por linhas e bytes,
contagens positivas e negativas, `+N`, multiplicadores, saturação de overflow,
sintaxe histórica, NUL como delimitador e headers. O scheduler compartilhado
cuida de processos, pipes, redirecionamento, backpressure e sinais.

O parser e os quoters existentes foram estendidos. A opção GNU
`--zero-terminated` foi acrescentada ao contrato. Help e version reproduzem as
mensagens capturadas, incluindo autoria e licença; ver `src-tauri/src/coreutils/messages/NOTICE.md`.

As leituras limitadas preservam os bytes restantes no TTY e na pipe. Em arquivo
regular, o cursor é reposicionado após leitura antecipada ou retenção do sufixo.
O fechamento de handles foi compartilhado entre os motores de `cat` e `head`.
A implementação antiga de `head` foi removida do adaptador finito de bytes.

### Diferenças encontradas e correções

| Observação GNU                                                          | Comportamento anterior/defeito encontrado          | Correção e evidência                                                                       |
| ----------------------------------------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Contagens com multiplicadores e delimitador NUL são aceitas             | Subset decimal, sem `-z`                           | Parser + transformação; matrizes `suffix-*`, `lines-suffix-*`, `binary-*`                  |
| Contagens acima de 64 bits saturam; sufixo inválido continua sendo erro | Conversão de tamanho podia rejeitar overflow       | Acúmulo saturado sem alocação proporcional ao número declarado; `count-*`, `suffix-zero-*` |
| `-n 0` verifica existência/permissões e pode emitir headers             | Leitura finita não distinguia abrir e ler          | Abertura separada da seleção; `file-3`, `file-11`, `directory-permission-zero`             |
| Headers usam bytes do nome; erros usam outro estilo de quoting          | Diagnósticos do subset divergiam                   | Quoter compartilhado com variante sempre citada; `header-*`, `missing-*`                   |
| Stdin regular pode recuperar bytes lidos além do prefixo                | Fast path antigo era só um contador de linhas      | `stdin-file-n`, `stdin-file-c`, testes de cursor e cleanup                                 |
| Prefixo positivo em TTY termina sem EOF                                 | Modo de bytes dependia de entrada finita           | `tty-lines`, `tty-bytes`, teste Rust sem EOF e produtor controlado                         |
| GNU exporta `POSIXLY_CORRECT` recebido no ambiente                      | O bridge nativo criava variável não exportada      | Exportação em invocações nativas; scripts preservam variáveis locais; caso `posix`         |
| Sinais descartam dados ainda retidos na seleção negativa                | Primeiro protótipo emitia os dados cedo demais     | Capturas de 6, 2048 e 8192 bytes; `negative-*-signal-*`                                    |
| Leitura antecipada de pipe é observável em `- -`                        | Fast path lia blocos diferentes do ambiente fixado | Limite de leitura observado no GNU/musl; `pipe-read-ahead`                                 |
| Diagnóstico histórico inválido usa o primeiro byte                      | Primeiro protótipo usava o caractere Unicode       | Diagnóstico binário; `legacy-utf8`                                                         |

Também foi corrigida a anexação ao próprio arquivo com contagem negativa:
`head -n -1 log >> log` e `head -c -2 log >> log` usam o tamanho inicial do
arquivo regular, evitando reler indefinidamente os bytes recém-anexados.
Os casos `self-negative-0/1` com 6000 bytes produziram os mesmos 11998 bytes
finais do GNU. No total, este relatório registra **11 classes de diferenças
corrigidas**; isso não representa uma contagem deduplicada de todos os casos
que falharam durante o desenvolvimento. Restam **zero diferenças obrigatórias**
na matriz final de `head`.

As capturas são a referência de comportamento. A análise também consultou o
[código oficial de head 9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/head.c)
para entender a retenção de leitura. O código do jogo não invoca esse binário.

### Cobertura

215 casos GNU independentes, com duas execuções idênticas por captura e nova
reprodução via `--verify`. Incluem arquivos, stdin, erros parciais, permissões,
symlinks/hardlinks, dados binários, headers, precedência, formas históricas,
abreviações, POSIXLY_CORRECT, sinais e descritores.

Os testes de propriedades variam chunks de 1, 2, 3, 7, 31, 255 e 4096 bytes.
Um produtor controlado exercita o scheduler sem depender de certificação de
`yes`. A captura exige ausência de handles abertos e registros de processos
após cada caso. O teste de fuzz existente encontrou um panic Unicode no
protótipo; a versão corrigida passou.

## Tooling reutilizável

- `scripts/cli/common-contracts.mjs`: pedidos comuns de help, version, opções
  inválidas, `--`, stdin, erros, permissões, bytes, pipe, redirecionamento e sinais.
- `scripts/cli/head-cases.mjs`: matrizes parametrizadas; não contém resultados
  esperados inventados.
- `scripts/cli-head-generate.mjs`: gera requests ou adota bytes GNU com verificação
  de proveniência e digest do pedido. `--link` atualiza os vínculos declarativos.
- `scripts/cli/coreutils-wave.mjs`: classifica o próximo item exato da fila,
  respeita o limite e identifica dependências arquiteturais.
- `scripts/cli-coreutils-wave.mjs`: exige fingerprint global atual antes de
  produzir a decisão da wave. Não edita código nem promove status.
- Captura Rust grava um snapshot por vez em arquivo temporário e só publica o
  JSON completo ao terminar. Isso elimina a retenção de centenas de árvores VFS.
- Execuções direcionadas preservam Foundation, `cat` e `head` com evidência atual.
  Fingerprints incluem parser, streams, VFS e os handlers realmente utilizados.

Exemplo de geração e captura (DEV, no ambiente GNU fixado):

```text
node scripts/cli-head-generate.mjs --requests artifacts/head-requests.json
python3 scripts/cli/coreutils-reference.py --capture --command head --probe artifacts/head-requests.json --output-dir artifacts/head-probe
node scripts/cli-head-generate.mjs --capture artifacts/head-probe/head.json --output tests/cli/compat/pilot/coreutils-head.json --link
python3 scripts/cli/coreutils-reference.py --capture --command head --replace
python3 scripts/cli/coreutils-reference.py --verify --command head
node scripts/cli-compat.mjs
node scripts/cli-verify.mjs --strict --command head
node scripts/cli-coreutils-wave.mjs --processed head
```

`GNU_PROBE` serve para investigação e geração de expectativas. Somente a captura
canônica `GNU_REFERENCE`, a execução virtual atual e os gates satisfazem a
certificação. Não foram criados waivers.

## Limites de interpretação

O contrato é para os argumentos/bytes representáveis pelo VFS e pelo shell
virtual, opções documentadas de GNU 9.7 e locale C no ambiente Linux/musl fixado.
Não certifica outros locales, dispositivos Linux completos nem opções internas
de teste do upstream. Permanecem os limites globais de recursos do jogo: canais
de 64 KiB, chunks de saída de 4 KiB e limites existentes de VFS/output. A retenção
negativa cresce com o sufixo efetivamente necessário, sem reservar de antemão
uma contagem arbitrariamente grande.

## Validação final e fila

| Executável | Casos GNU | Matches | Gates | Contratos | Resultado |
| ---------- | --------- | ------- | ----- | --------- | --------- |
| `basename` | 48        | 48      | 19/19 | 5/5       | VERIFIED  |
| `printenv` | 36        | 36      | 19/19 | 5/5       | VERIFIED  |
| `whoami`   | 31        | 31      | 19/19 | 5/5       | VERIFIED  |
| `dirname`  | 27        | 27      | 19/19 | 5/5       | VERIFIED  |
| `head`     | 215       | 215     | 26/26 | 9/9       | VERIFIED  |
| `cat`      | 91        | 91      | 26/26 | 9/9       | VERIFIED  |

Total canônico: **448 casos GNU** nos seis executáveis certificados. O probe
de boundary não entra nessa soma. A mudança do harness invalidou a proveniência
antiga: Foundation e `cat` foram recapturados e reproduzidos; seus bytes de
caso permaneceram iguais. Foram renovados metadados de captura, hash do harness
e hash do lock, sem transcrever resultados do simulador para GNU.

| Verificação                                | Resultado                                                    |
| ------------------------------------------ | ------------------------------------------------------------ |
| GNU `--verify`: Foundation, cat e head     | PASS; duas execuções idênticas por caso                      |
| Declarativos globais                       | 583 PASS, 0 FAIL, 0 SKIPPED (antes: 376)                     |
| Suíte legada de compatibilidade            | 76/76 PASS                                                   |
| Rust completo                              | 204 PASS, 0 FAIL; 7 helpers DEV ignorados por padrão         |
| Frontend                                   | 214 PASS em 36 arquivos                                      |
| Tooling                                    | 35 PASS                                                      |
| Strict head / cat / Foundation             | PASS individualmente                                         |
| Verificação global normal e regressões     | PASS                                                         |
| Strict global                              | FAIL esperado: 1514 dívidas de compatibilidade (antes: 1515) |
| Host Guard                                 | PASS                                                         |
| Shell / VFS                                | 7 / 11 capacidades READY; JOBS / DEVICES continuam PARTIAL   |
| Rustfmt / Clippy com warnings como erros   | PASS                                                         |
| TypeScript / ESLint / Stylelint / conteúdo | PASS                                                         |
| Prettier                                   | PASS                                                         |
| Build Web                                  | PASS; aviso existente de chunk JavaScript acima de 500 kB    |
| Build Windows standalone                   | PASS; release com assets embutidos (`tauri/custom-protocol`) |

A suíte global passa sem promover o catálogo inteiro. O strict global contém
somente rejeições `Strict: ...` de executáveis ainda não certificados; o modo
normal não reporta regressões, evidências stale ou falhas de isolamento.
O estado global mudou de 5/115/1406 para **6 VERIFIED / 114 PARTIAL /
1406 UNVERIFIED**. A família Coreutils permanece PARTIAL.

Build Windows concluído em 7m42s. Executável local:
`src-tauri/target/release/game-hacker.exe`, 89698304 bytes,
SHA-256 `dc889eed1d615be68fdf101b8663f02cdbde09f11520edae3dda300233e55606`. O binário é um artefato de build ignorado pelo Git;
os fontes, referências canônicas e instruções de reprodução são versionados.

### Identidade da evidência

- Fingerprint global inicial: `c5d9fd0f96eedc7f4af658dcc226e3cfa0fa81fd76f238599e5aa80fc5fb4caf`.
- Fingerprint global final: `c2e7b0abc80294e2025bdaf60f5fb4976e99665b1f684dd8278dfd8d19faeb58`.
- Execução final: `2026-09-17T23:27:41.263Z`.
- Ambiente: `alpine-3.22.0-coreutils-9.7-r1-x86_64`, locale C.
- Binário multicall GNU (SHA-256): `8c3d5024b3ea24ef924e50b3b1647bf1623fccacd56cca1e45a4395b6b3bc600`.
- [Snapshot derivado: estado, escopos estritos, fila e hashes](evidence/m1c3-final-state.json).
- Referência canônica: `tests/cli/gnu/coreutils/9.7/head.json` (schema 3).
- Casos declarativos: `tests/cli/compat/pilot/coreutils-head.json`.
- Inventário e relatórios gerados: `docs/generated/coreutils-inventory.json`,
  `coreutils-compatibility.md`, `cli-verification.md`, `cli-dashboard.md`,
  `shell-compatibility.md`, `vfs-compatibility.md` e fila/dependências existentes.

Não houve atualização manual de hashes para conservar um PASS antigo. A última
alteração de tooling foi seguida de nova execução global; a decisão da wave
recusaria um fingerprint global desatualizado.

### Medições úteis

Amostras locais do harness em debug, uma execução por caso, sem comparação de
velocidade com GNU e sem promessa de latência de produção:

| Caso                    | Entrada / saída                                        | Tempo observado |
| ----------------------- | ------------------------------------------------------ | --------------- |
| `head -c8193 large`     | Arquivo de 131072 bytes; saída de 8193 bytes           | 7,19 ms         |
| `head -qn1 - -` em pipe | Read-ahead e consumo entre operandos; saída de 4 bytes | 4,48 ms         |
| `head -c3` em TTY       | Prefixo de 3 bytes, sem exigir EOF                     | 4,48 ms         |

O produtor controlado confirma fechamento do consumidor antes do EOF; testes
confirmam a capacidade máxima de pipe de 65536 bytes e liberação de handles.
A captura DEV grava cada snapshot antes de processar o próximo caso, evitando
reter a matriz inteira de árvores VFS na memória do processo Rust.

### Reprodução da validação

```text
node --test scripts/cli/tooling.test.mjs scripts/cli/foundation.test.mjs scripts/cli/head.test.mjs
node scripts/cli-compat.mjs
node scripts/cli-verify.mjs --strict --command head
node scripts/cli-coreutils-wave.mjs --processed head
node scripts/cli-verify.mjs --strict --command cat
node scripts/cli-verify.mjs --strict --wave foundation
node scripts/cli-verify.mjs
node scripts/cli-verify.mjs --strict
cargo test --offline --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --offline --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
pnpm format:check
pnpm lint
pnpm typecheck
pnpm content:check
pnpm test
pnpm build:web
cargo build --offline --manifest-path src-tauri/Cargo.toml --release --features tauri/custom-protocol
```

O último strict global deve falhar enquanto a dívida registrada existir. No
Windows local, `.tools/run-rust.ps1` configura o toolchain usado nesses comandos.
A CI agora também reproduz GNU `head` e executa seu strict individual.

## Parada arquitetural: tail

A decisão foi obtida por `node scripts/cli-coreutils-wave.mjs --processed head`
logo após o strict de `head`. O primeiro item é `tail`, com 8 casos de projeto,
0 referências canônicas GNU, 14/26 gates e a lacuna explícita
`Follow/retry requires process/filesystem notifications`.

| Aspecto                          | Evidência e decisão                                                                                                                                                                                                                     |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Comportamento requerido          | Manter o processo ativo no EOF; acompanhar acréscimos e substituição do arquivo pelo nome; cancelar por sinal.                                                                                                                          |
| GNU 9.7                          | `tail -n1 --follow=name --retry --sleep-interval=0.01 --max-unchanged-stats=1 log` emite `first\n`, depois `second\n` após append e `third\n` após substituição atômica; permanece ativo em cada barreira e termina por SIGTERM.        |
| Cyber War atual                  | As mesmas opções retornam status 1: `unrecognized option '--follow=name'`. `-f log` também retorna 1: `invalid option -- 'f'`. Stdout vazio e VFS inalterado nos dois probes.                                                           |
| Subsistema ausente               | Assinaturas de eventos do VFS e espera/cancelamento no scheduler, com identidade de inode e acompanhamento do caminho durante escrita, truncamento, remoção e recriação.                                                                |
| Por que não corrigir apenas tail | Uma rotina local de polling duplicaria agendamento e invalidaria garantias compartilhadas de espera, backpressure, lifetime, sinais e cleanup. O pedido proíbe esse atalho para obter VERIFIED.                                         |
| Próximo milestone recomendado    | Eventos/observadores do VFS integrados aos processos virtuais; follow por descritor e por nome; retry; rotação/truncamento; cancelamento e descarte de assinaturas; testes de persistência/recarga; depois fechamento vertical de tail. |

Prova reproduzível:

- [GNU: append, rotação e SIGTERM](evidence/m1c3-tail-boundary.json).
- [Cyber War: rejeições atuais](evidence/m1c3-tail-virtual.json).
- `scripts/cli/coreutils-follow-probe.py`: executa duas vezes em namespaces
  independentes, sem rede, com fixture temporária e barreiras de saída. A
  substituição é atômica para não depender da corrida entre remover e recriar.
- O probe exige o hash do binário multicall Alpine já fixado e a versão 9.7.
  `--verify` reproduz os resultados sem sobrescrever a captura.
- Esses arquivos têm proveniência de **probe de boundary** e
  `certifiesExecutable: false`; não satisfazem `GNU_REFERENCE` de `tail`.

```text
python3 scripts/cli/coreutils-follow-probe.py --verify --output docs/coreutils/evidence/m1c3-tail-boundary.json
```

O caminho de evolução foi registrado; o novo subsistema não foi iniciado.
Os gaps menores de `tail` (multiplicadores e NUL) não foram usados como motivo
para parar. Nenhum item posterior, como `wc` ou `tee`, foi escolhido para
contornar a fila.

## Isolamento

Gameplay continua **100% virtual**: shell, VFS, processos, TTY, pipes e sinais.
GNU/WSL/Python/bubblewrap são exclusivamente ferramentas DEV/CI. O bridge de
captura é compilado somente em testes e o Host Guard verifica o runtime.
SHELL.JOBS e VFS.DEVICES continuam PARTIAL.

## Próximo item

```text
NEXT QUEUE ITEM: coreutils/tail
CLASSIFICATION: ARCHITECTURAL_BOUNDARY
STOP_REASON: ARCHITECTURAL_BOUNDARY
```
