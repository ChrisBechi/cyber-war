# CLI Compatibility — Milestone 0

Este sistema mede a compatibilidade do terminal virtual. Um nome registrado, um
launcher, uma implementação parcial e um programa verificado são coisas distintas.
Os relatórios são calculados; não edite os status gerados.

## Executar

Requisitos: Node 24, pnpm do projeto e toolchain Rust/Tauri. As dependências Rust
devem estar disponíveis localmente (`cargo fetch --manifest-path src-tauri/Cargo.toml`
na preparação do ambiente). Os runners usam Cargo offline. No Windows, reconhecem
automaticamente a toolchain existente em `.tools`, quando presente.

```sh
pnpm cli:inventory
pnpm cli:test
pnpm cli:compat
pnpm cli:verify
# Ou toda a sequência:
pnpm cli:check
# Exige todos os executáveis REAL_COMPAT obrigatórios verificados:
pnpm cli:verify --strict
# Compara com outro baseline de dívida:
pnpm cli:verify --baseline docs/cli-compatibility/debt-baseline.json
```

O modo normal valida integridade, isolamento e regressões. Dívida existente continua
PARTIAL/UNVERIFIED. O modo estrito falha enquanto essa dívida existir. Falhas nos
casos executados fazem `cli:compat` falhar; não são convertidas em sucesso.

## Arquitetura e fontes

```text
Rust: COMMANDS + SHELL_ONLY + software catalog + repository payload bindings
  → command_registry (runtime; índices, disponibilidade, autocomplete)
  → cli_tooling_bridge (somente cfg(test); exportação e execução de fixtures)
  → discovery + manifest Zod + baseline + capabilities/subsystems
  → casos versionados + captura real + comparação de streams/estado
  → gates → effectiveStatus → métricas, dashboard, fila e diff
```

- `src-tauri/src/command_registry.rs`: nomes descobertos no runtime. Inclui nomes
  dos provedores de pacotes; não consulta certificação durante o jogo.
- `src-tauri/src/software.rs`: índices de software/comando mantêm a precedência
  anterior do catálogo. `completion.rs` consome o registro canônico.
- `content/cli-compatibility/manifest.json`: decisões explícitas de agrupamento,
  classificação, versão, capacidades, gates e colisões de registros. Novas
  colisões, metadados órfãos e versões inconsistentes são erros.
- `baseline.json`: perfil virtual amd64, C.UTF-8, bash 5.2.37 e TTY 80×24.
  **Não há snapshot oficial Kali fixado.** Versões dos pacotes existentes foram
  preservadas; isso não comprova equivalência com esses programas reais.
- `subsystems.json`: capacidades dependem de subsistemas identificados. READY
  exige testes obrigatórios atuais com PASS; referências a testes antigos são
  contexto da auditoria, não certificação automática.
- `scripts/cli/schema.mjs`: schemas versionados e estritos. Campos e capacidades
  desconhecidos são rejeitados.
- `scripts/cli/reference-environment.mjs`: interface DEV para uma futura referência
  controlada. Não implementa execução de comandos do host nem fallback.

O JSON completo contém vínculos command → software → package, executablePath,
packageBindings, disponibilidade BASE_SYSTEM/INSTALLABLE/MISSION_PROVIDED/FICTIONAL,
alias → canonical, metadados man/reference, gates, evidências e bloqueios.
Pacotes `.deb` personalizados podem registrar nomes específicos de um save; o
inventário global enumera os provedores e payloads distribuídos com o projeto.

## Reutilização do mundo virtual

| Responsabilidade      | Serviço existente                                      | Limite relevante                                   |
| --------------------- | ------------------------------------------------------ | -------------------------------------------------- |
| Arquivos e permissões | `WorldState.vfs` / `vfs.rs`                            | Inodes, links e permissões Linux incompletos       |
| Usuários e relógio    | `world.rs` / contexto do terminal                      | Identidades fixas e tempo virtual                  |
| Processos e sinais    | `task_manager.rs`, processos virtuais, jobs de archive | Não há lifecycle genérico de todos os processos    |
| Rede/DNS/HTTP         | `network.rs`, `browser.rs`                             | Hosts e rotas virtuais; sockets genéricos ausentes |
| Wi-Fi/packets         | Metadados de access points                             | Sem engine de frames/captura de pacotes            |
| Pacotes               | `packages/model.rs`, `service.rs`, `executables.rs`    | Contratos CLI ainda parciais                       |
| Sessões remotas       | `terminal_sessions.rs`, contexto SSH                   | Sessões limitadas ao modelo atual                  |
| Eventos/missões       | World events e journal de recursos                     | Sem EventBus genérico paralelo                     |
| Persistência          | Save de `WorldState`                                   | Manifestos e certificação não entram no save       |

Novas engines devem consumir esses serviços diretamente. Não criamos serviços
fictícios de sockets, packets ou exploitation para declarar dependências prontas.

## Contrato de execução

`terminal::CommandResult` continua sendo o contrato IPC com stdout, stderr,
exitCode e chunks ordenados. Handlers `GameResult<String>` continuam atrás do
adaptador existente e aparecem no relatório de dívida. Migre cada handler para
o resultado estruturado junto com os respectivos casos de compatibilidade.

`cli_contract.rs` prepara `Input::{Stdin,Eof,Cancel}`, eventos de stdout/stderr,
espera de entrada e término, `Lifecycle`, TTY e `VirtualInteractiveEngine`.
É um contrato de extensão, sem runner de host. Jobs de archive e nano preservam
seus ciclos atuais. O transporte de testes aplica columns e descritores TTY;
rows/ANSI/interactive são metadados preparados para engines futuras. Isso não
certifica streaming ou interatividade existentes.

## VERIFIED e aplicabilidade

Cada gate tem `REQUIRED`, `OPTIONAL` ou `NOT_APPLICABLE`, justificativa e testIds.
O resultado é PASS, FAIL ou SKIPPED. Somente gates REQUIRED com PASS satisfazem
o contrato obrigatório; SKIPPED bloqueia. Falhas opcionais ficam visíveis e não
bloqueiam sozinhas. N/A não entra no denominador de gates obrigatórios.

VERIFIED exige implementação funcional, referência fixada para REAL_COMPAT,
todos os gates obrigatórios PASS e dependências READY. CATALOG_ONLY e LAUNCHER
nunca viram VERIFIED. Uma declaração manual de VERIFIED sem esses requisitos
faz a validação falhar. FICTIONAL_NATIVE não precisa de upstream real, mas mantém
gates e dependências virtuais; não entra no percentual REAL_COMPAT.

Uma família só fica VERIFIED quando todos os executáveis REQUIRED estiverem
verificados, seus aliases tiverem resolução comprovada e as dependências forem
verificadas. OPTIONAL/DEPRECATED permanecem visíveis, fora desse conjunto.
Aliases não duplicam o denominador de executáveis. IDs/display names do catálogo
que não são comandos são LAUNCHER; entradas do catálogo também têm um inventário
separado de launchers, sem somá-los ao progresso CLI.

## Casos e referências

Os casos em `tests/cli/compat/pilot/` usam schemaVersion 2. Cada caso declara
software, versão, command/argv, stdin, env, cwd, TTY, fixture e resultado esperado.
`script` usa o parser virtual; argv usa a entrada tokenizada real, sem shell do
host. Cada caso recebe um `WorldState::new` independente. Setup também usa
comandos virtuais. A captura inclui estado antes/depois de VFS, processos, rede,
pacotes, cwd e usuário.

- EXACT preserva bytes, espaços, quebras de linha e números.
- REGEX exige expressão explícita; STRUCTURED compara objetos JSON.
- NORMALIZED_DYNAMIC exige campo delimitado, tipo e justificativa. Há regras
  específicas de PID, timestamp, latency, transfer rate e identifier; nada
  remove globalmente espaços, números ou datas.
- Asserts de estado usam JSON Pointer, matcher, ausência ou igualdade antes/depois.

Os pilotos usam expectativas declaradas e metadados de referência; não são
capturas certificadas de um Kali externo. O formato já associa cada referência
à versão. Uma futura coleta diferencial deve registrar ambiente/digest e revisão,
mantendo o executor de referência fora de `src` e `src-tauri/src` de produção.
Os 76 golden cases legados continuam executados, sem promover status por contagem.

Evidências expiram quando fontes, manifestos, casos ou tooling mudam. SHA-256
identifica o conjunto de entrada e a captura; uma captura antiga vira SKIPPED.
Esses hashes dão rastreabilidade local, não assinatura contra um mantenedor malicioso.

## Segurança e CI

O bridge está protegido por `#[cfg(test)]`. O guard procura APIs de processo,
filesystem, rede e imports DEV no código de produção, inclusive código depois de
itens de teste. A única exceção de filesystem é a criação do diretório de dados
do próprio Tauri. `cli:test` inclui tentativas de violar essa fronteira. Os
comandos do jogador continuam no mundo virtual; o runner DEV invoca apenas
Cargo com argumentos fixos, nunca transforma o comando de um caso em comando do host.

O guard verifica a fronteira de fontes/build; não é uma prova formal de sandbox
do sistema operacional. O CI Windows executa `cli:check` depois dos testes Rust e
publica os JSONs e Markdown como artifacts. Não são executados containers ou
ferramentas de segurança reais para gerar o inventário.

O baseline de dívida é um snapshot revisável. Perder VERIFIED, remover um comando
ou trocar implementação por CATALOG_ONLY falha sem override. Overrides precisam
de ID, motivo com contexto, referência e expiração futura. Nunca servem para
transformar status ou testes falhos em PASS.

## Fluxo de trabalho

O plugin `cliRuntimeBoundary` rejeita módulos de `scripts/cli/` durante o build
Vite; há teste que tenta empacotar a referência e exige falha. O bridge Rust
contém `compile_error!` sob `cfg(not(test))`, além do guard de registro do módulo.

1. Registre o software e fixe upstream/versão/referências.
2. Registre executáveis no runtime e vincule package/aliases/capabilities.
3. Revise gates e requisitos de subsistemas; documente N/A e limitações.
4. Implemente usando o mundo virtual e os contratos de resultado existentes.
5. Adicione casos com referência versionada, erros, streams e efeitos de estado.
6. Execute `pnpm cli:check`; examine FAIL/SKIPPED, dashboard e diff.
7. Corrija a implementação ou amplie evidências. A promoção é calculada.

Para software CATALOG_ONLY, comece pelo grupo, versão e dependências, não por
respostas textuais estáticas. Nmap, Netcat, Aircrack-ng e Metasploit permanecem
fora do escopo de implementação deste milestone.

## Saídas

- [Dashboard](../generated/cli-dashboard.md), [inventário](../generated/cli-inventory.md),
  [famílias](../generated/cli-software-families.md) e [fila](../generated/cli-next-work.md).
- [Evidências](../generated/cli-verification.md), [parciais](../generated/partial-cli.md),
  [verificados](../generated/verified-cli.md), [sem versão](../generated/unpinned-software.md),
  [somente catálogo](../generated/catalog-only-software.md) e [retornos legados](../generated/cli-legacy-results.md).
- `artifacts/cli-inventory.json`, `cli-implementation-queue.json`, `cli-verification.json`,
  `cli-case-actual.json`, `cli-evidence.json`, `cli-diff.json`, `cli-verify-result.json`.
- `artifacts/cli-performance.json`: geração do inventário e medições de lookup.
  Logs DEV e snapshots ficam em artifacts; não são enviados ao stdout do jogador.
- `docs/cli/inventory.json`: índice compacto para documentação e teste de cobertura
  dos registros. Não é fonte de status independente.

O próximo trabalho é determinado por wave, prioridade, dependências prontas,
referências e uso em missões. `queue.next` mostra o software e os bloqueios a
resolver; uma entrada NEEDS_SUBSYSTEM pede trabalho na infraestrutura primeiro.
