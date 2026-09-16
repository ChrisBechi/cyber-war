# CYBER WAR — Package Management System

Implementação em 16/09/2026, sobre o projeto 0.4.2.

Este documento registra o sistema implementado. Substitui o rascunho de especificação anterior, após o pedido de implementação. Os comandos abaixo são executados **no terminal do jogo**.

## Implementado

APT, dpkg e o instalador gráfico compartilham o mesmo estado Rust por campanha. A instalação escreve arquivos no VFS, registra propriedade, resolve dependências no APT e disponibiliza executáveis virtuais, manuais e lançadores. Remoção, purge, atualização, reparo, cache, permissões, confirmação, cancelamento e saves usam esse estado.

Nenhum comando de pacote do jogador inicia subprocessos ou acessa repositórios reais. Executáveis instalados são descritores vinculados a handlers Rust permitidos.

## Arquitetura

| Módulo em `src-tauri/src/packages/` | Responsabilidade                                                               |
| ----------------------------------- | ------------------------------------------------------------------------------ |
| `model.rs`                          | Definições, dependências, instalação, ownership, índices, eventos e transações |
| `deb.rs`                            | Envelope ar, metadados, integridade e limites dos pacotes virtuais             |
| `version.rs`                        | Comparação Debian: epoch, revisão, tilde e segmentos numéricos                 |
| `resolver.rs`                       | Candidatos, Depends/alternativas, Provides, conflitos, ciclos, held e órfãos   |
| `repository.rs`                     | Catálogo local, releases e leitura das sources no VFS                          |
| `state.rs`                          | Baseline, migração, validação e projeções legíveis no VFS                      |
| `service.rs`                        | Plano, espaço, cache, instalação, configuração, remoção e rollback             |
| `cli.rs`                            | Sintaxe, stdout/stderr, permissões, confirmação e execução                     |
| `executables.rs`                    | PATH, permissões de execução, handlers e manuais                               |
| `ipc.rs`                            | Inspeção/instalação gráfica, downloads e validação de lançadores               |
| `tests.rs`                          | Segurança, integração, persistência e desempenho                               |

`WorldState.packages` é a autoridade. `/var/lib/dpkg/status`, `/var/lib/dpkg/info/*.list`, `/var/lib/apt/lists/virtual-index` e `/var/lib/apt/extended_states` são projeções de leitura. O jogador modifica `/etc/apt/sources.list` e os arquivos `.list` em `sources.list.d` pelo VFS, inclusive com editor/terminal.

A transação trabalha sobre uma cópia do mundo. Só publica o resultado consistente; `GameService::mutate` grava blobs, snapshot e checkpoints numa transação SQLite antes de substituir o mundo ativo. Falhas SQL preservam a memória e o save anterior.

Os jobs reutilizam a infraestrutura dos archives: processo virtual, progresso com duração limitada, cancelamento cooperativo e execução sob o mutex do GameService. O lock de pacotes impede mutações concorrentes. Um plano deixa de ser válido quando os arquivos ou o estado de pacotes mudam enquanto aguarda confirmação.

O histórico registra as fases percorridas: PLANNED, DOWNLOADING, VERIFYING, UNPACKING, CONFIGURING, COMMITTING e COMPLETED; erros e interrupções terminam em FAILED ou CANCELLED. Reutilização de cache não registra um download inexistente. A configuração incompleta deliberada do dpkg registra falha e mantém o estado reparável.

## Comandos suportados

| Comando                | Subconjunto funcional                                                                                                                             |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `apt`, `apt-get`       | `update`, `search TEXTO...`, `show PACOTE`, `list [FILTRO] [--installed\|--upgradable]`, `policy [PACOTE]`                                        |
| `apt`, `apt-get`       | `install [-y] PACOTE[=VERSÃO]...`, `reinstall [-y] PACOTE...`, `remove [-y] PACOTE...`, `purge [-y] PACOTE...`, `upgrade [-y]`, `autoremove [-y]` |
| `apt`, `apt-get`       | `--fix-broken install [-y]`, `install --reinstall [-y] PACOTE...`, `clean`, `autoclean`                                                           |
| `apt-cache`            | `search TEXTO...`, `show PACOTE`, `policy [PACOTE]`                                                                                               |
| `apt-mark`             | `manual`, `auto`, `hold`, `unhold` com nomes; `showmanual`, `showauto`, `showhold`                                                                |
| `dpkg`                 | `-i ARQUIVO.deb...`, `-r PACOTE...`, `-P PACOTE...`, `--configure -a`                                                                             |
| `dpkg`, `dpkg-query`   | `-l [TRECHO]`, `-s PACOTE`, `-L PACOTE`, `-S CAMINHO_OU_NOME`                                                                                     |
| Todos os seis comandos | `--help`, `-h`, `--version`, `-v`                                                                                                                 |

Aliases de flags APT: `--assume-yes`/`-y`, `--fix-broken`/`-f`, `--installed`/`-i`, `--upgradable`/`-u`, `--reinstall`/`-r`, nos contextos indicados. A listagem dpkg usa filtro literal por trecho, não glob Debian. Consultas `show`, `-s`, `-L` e `-S` têm um operando documentado.

Mutações exigem root virtual (`sudo`). Consultas funcionam offline. Códigos: sucesso 0; erro APT 100; erro de pacote/parser dpkg 2; configuração dpkg incompleta 1; interrupção 130; recusa da confirmação 1. A execução de ferramenta ausente usa o erro do terminal. Pipes e redirecionamentos usam os mecanismos existentes; comandos de instalação em scripts/pipes precisam de `-y`.

A integração também abrange `which`, `whereis`, `type`, `command -v`, `man`, `file` e autocomplete. `tee` foi integrado ao VFS para editar sources em pipelines. O manual interno descreve o subconjunto disponível.

## Pacotes e repositórios

Baseline: base-files, bash, coreutils, curl, iproute2, nano, nmap, openssh-client, procps, sudo, systemd e wget. Pacotes essenciais são protegidos de remoção. Ferramentas anteriores fora desse conjunto continuam no catálogo existente; a implantação não converte automaticamente os 303 lançadores em novos pacotes.

Pacotes adicionais: cyber-core, libpacket2, netscan, wireless-utils e iot-discovery. Há 17 nomes distintos e versões adicionais de netscan. A versão 2.5.0 é liberada pelo estado de release 2 do repositório; a release inicial oferece 2.4.1 e 2.3.0.

| Repositório virtual | Source                                                            |
| ------------------- | ----------------------------------------------------------------- |
| Oficial             | `deb https://mirror.kali.game/kali rolling main contrib non-free` |
| Blackwire           | `deb https://repo.blackwire.net/tools stable main`                |

`apt update` processa sources ativas, combina duplicatas, verifica suite/componentes/disponibilidade/confiança e substitui os índices obtidos. Falhas parciais preservam os índices anteriores daquele repositório e retornam erro; repositórios válidos ainda são atualizados. Remover uma source e atualizar retira seus candidatos sem desinstalar programas existentes. URLs desconhecidas falham dentro da simulação.

Os navegadores internos apresentam um índice clicável de pacotes e páginas de download. Os arquivos usam caminhos únicos com complemento numérico quando já existe o nome. `wget`/downloads virtuais usam o mesmo artefato.

## DEB e segurança

O envelope usa ar com `debian-binary` 2.0, `control.tar.*` e `data.tar.*`. TAR/GZIP/BZIP2/XZ são produzidos/lidos pelo ArchiveService existente. O controle inclui campos Debian, `conffiles` quando necessário e o manifesto `cyber-war.json`. Esse perfil define payload, tamanhos lógicos, handlers permitidos e ações declarativas.

A leitura aceita control TAR/GZIP/XZ e data TAR/GZIP/XZ/BZIP2. São verificadas estrutura ar, contagem de membros, arquitetura amd64/all, nomes, versões, metadados, conteúdo/mode/link dos arquivos e SHA-256 do artefato. MIME é detectado por bytes na importação, inclusive após renomear o arquivo.

Limites: DEB físico de 32 MiB; até 4.096 entradas por payload; conteúdo lógico até 16 GiB; limites adicionais de profundidade, nomes, expansão e texto do ArchiveService/VFS. Relações e ações também têm limites. Caminhos de payload só podem ficar em `/usr`, `/etc`, `/opt` e `/var/lib`, excluindo os próprios bancos/sources de pacotes. Traversal, caminhos de host, symlinks escapando e scripts desconhecidos são recusados.

Ações permitidas: criar diretório e criar arquivo de dados inicial, restritos a `/var/lib/NOME_DO_PACOTE/`. Não são shell scripts. Nenhum binário extraído é executado pelo sistema operacional do host.

## Estados, dependências e arquivos

Estados persistidos: `unpacked`, `halfConfigured`, `installed` e `configFiles`. Ausência no registro representa não instalado. Problemas detalhados e eventos representam situações quebradas; não há um booleano que esconda a falha.

APT instala Depends transitivas, alternativas e providers compatíveis, com operadores `=`, `<<`, `<=`, `>=`, `>>`. Prefere a versão mais nova disponível para a arquitetura e preserva pacotes held. Ciclos, restrições incompatíveis e conflitos produzem erro legível. O algoritmo é determinístico e limitado a 128 níveis e 10.000 tentativas; não é um solver SAT universal.

`dpkg -i` instala o payload local e tenta configurar. Dependências ausentes deixam o pacote desempacotado; falha numa ação deixa `halfConfigured`. `apt --fix-broken install` busca dependências faltantes; `dpkg --configure -a` tenta configurar novamente. APT não publica uma instalação que termina com dependências inválidas.

Ownership registra arquivos e diretórios criados. Colisões entre pacotes ou arquivos pessoais impedem instalação; `Replaces` explícito permite transferência de propriedade. Dois payloads novos não podem reivindicar o mesmo arquivo na mesma operação. Diretórios compartilhados mantêm os proprietários e só são removidos quando vazios.

`remove` preserva conffiles, inclusive alterados pelo jogador. `purge` remove os conffiles registrados e as cópias de novos padrões `.dpkg-dist`. Upgrade mantém a configuração local e disponibiliza o novo padrão separadamente. Exclusões locais de conffiles são preservadas. Arquivos comuns alterados são preservados na remoção e bloqueiam substituição numa atualização.

Dependências instaladas pelo APT recebem marca automática; instalação explícita promove para manual. `autoremove` percorre dependências dos pacotes manuais/essenciais/held ainda ativos; configurações residuais não mantêm dependências desnecessárias.

Tamanho lógico controla o disco virtual. Conteúdo físico é pequeno e deduplicado pelo armazenamento de blobs existente. O plano considera cache, payload e espaço temporário conservador. Cache válido pode permitir reinstalação offline; conteúdo corrompido é recusado ou baixado novamente se o repositório estiver disponível.

## Integrações

- **File Manager:** associação DEB, inspeção, metadados, plano, confirmação root virtual, progresso, cancelamento e logs.
- **Terminal:** mesmo plano e serviço; estado de saída, confirmação interativa e encerramento de terminal liberando/cancelando trabalho.
- **Launcher:** lê desktop entries instalados; backend valida ownership, conteúdo e executável. O NetScan abre um terminal com o handler correspondente.
- **Man/PATH:** arquivos reais do VFS e permissões, com desaparecimento após remoção.
- **Save:** pacotes, ownership, versões, marcas, sources, índices e cache pertencem ao snapshot do slot. Locks, prompts e jobs em execução não são persistidos.
- **Migração:** saves sem `packages` recebem baseline e importam `aptInstalled`, `aptManual`, `aptHeld` uma vez. Não foi necessária migration SQL adicional; o esquema de blobs/snapshots existente é reutilizado.
- **MissionEngine:** condições `packageInstalled` e `packageEvent`; efeito `repository` para disponibilidade, confiança e release. Pacotes e seus arquivos não são desfeitos ao abandonar uma tentativa de missão.

Eventos: PACKAGE_INSTALLED, PACKAGE_REMOVED, PACKAGE_PURGED, PACKAGE_UPGRADED, PACKAGE_BROKEN, PACKAGE_REPAIRED, APT_UPDATED, REPOSITORY_ADDED e REPOSITORY_REMOVED. O histórico retém 512 eventos e 128 transações por campanha.

## Como validar no jogo

```text
sudo apt update
apt search netscan
apt show netscan
sudo apt install netscan
# Responda y
which netscan
netscan --version
man netscan
dpkg -s netscan
dpkg -L netscan
dpkg -S /usr/bin/netscan
```

Salvar, sair e carregar deve manter instalação e arquivos. Depois:

```text
sudo apt remove -y netscan
cat /etc/netscan/netscan.conf
sudo apt autoremove -y
sudo apt purge -y netscan
```

Descoberta de repositório:

```text
echo 'deb https://repo.blackwire.net/tools stable main' | sudo tee /etc/apt/sources.list.d/blackwire.list
sudo apt update
apt show iot-discovery
sudo apt install -y iot-discovery
```

Para GUI: abra `https://mirror.kali.game/kali` no navegador do jogo, escolha wireless-utils, baixe o DEB e abra-o em Downloads. Revise o plano e confirme. O terminal deve refletir a instalação imediatamente.

## Testes e desempenho

A suíte Rust cobre versões, archives reais, dependências/ciclos/alternativas/providers, conflitos e Replaces, conffiles, reparo, held/manual/auto, cache offline, sources, disco cheio, cancelamento, lock, plano obsoleto, permissões, pipes, migração, cinco slots, checkpoint, falha SQL, missões e validação de payloads maliciosos.

Frontend: inspeção antes de instalar, confirmação, cancelamento, liberação do plano ao fechar, erro de arquivo inválido e associação por extensão/MIME. A regressão cobre também o navegador, desktop, terminais, editor, saves e demais interfaces existentes.

E2E em Tauri/WebView2 com SQLite em memória e IPC Rust real: 13 verificações passaram, incluindo File Manager → instalador → terminal → save/reload → remove/purge, APT/dependências, launcher e browser/download. Evidência: `artifacts/package-installer-native.png`; roteiro `.tools/package-qa.js`. Não utiliza mock de backend. O helper de captura Windows falhou; a imagem foi obtida pelo protocolo de depuração da própria janela de QA.

Benchmark separado, build Rust de desenvolvimento, 21 amostras por tamanho de catálogo; busca pelo comando APT completo, resolução de uma instalação simples e consulta BTreeMap. Não mede download físico nem grafos patológicos.

| Entradas | Consulta P50 |  Busca P50 |  Busca P95 | Resolver P50 |
| -------: | -----------: | ---------: | ---------: | -----------: |
|      100 |     0,002 ms |   2,439 ms |   3,085 ms |     0,611 ms |
|    1.000 |     0,004 ms |  13,662 ms |  16,188 ms |     5,525 ms |
|    5.000 |     0,006 ms |  61,810 ms |  72,109 ms |    29,537 ms |
|   10.000 |     0,006 ms | 122,678 ms | 138,285 ms |    56,225 ms |

Medição final em 16/09/2026, após o término da compilação de produção, com a tabela de consulta do mesmo tamanho do catálogo. Todas as metas passaram: consulta P50 abaixo de 50 ms, busca P50 abaixo de 300 ms/P95 abaixo de 1 s e resolução P50 abaixo de 100 ms. Os tempos dependem da máquina e da carga; o cenário do resolver é uma instalação simples, não um grafo complexo de 10 mil dependências.

## Dependências adicionadas

Nenhuma. Foram reutilizados serde, SHA-256, UUID, SQLite, Tauri e os codecs do ArchiveService já presentes. Nenhum gerenciador de pacotes do host foi usado pelo runtime.

## Limitações intencionais

- Perfil DEB próprio do jogo: arquivos Debian arbitrários sem `cyber-war.json`, binários nativos, maintainer scripts, GPG real, multiarch, triggers, diversions e alternativas do sistema Debian não são suportados.
- Recommends/Suggests são metadados; a instalação automática resolve Depends.
- Repositórios são definições locais; confiança e releases são estado do mundo, sem HTTP externo.
- O resolver procura alternativas de dependências com limites explícitos e rejeita planos incompatíveis; não promete resolver todo problema combinatório aceito pelo APT real.
- Os novos desktop entries abrem ferramentas no terminal virtual. O catálogo gráfico anterior permanece para aplicações existentes.
- O pacote não fornece gerenciamento remoto de software via SSH; erros são explícitos.
- Progresso temporal usa os jobs existentes e o tamanho lógico. Commit e configuração são atômicos para o observador externo.

## Problemas encontrados e correções

O APT anterior mantinha listas em settings e respondia sem instalar arquivos. Foi substituído pelo serviço compartilhado. Também foram corrigidos confirmação em contextos aninhados, ausência de tee, nomes canônicos do browser, promoção manual, órfãos após remove, colisões de payload, falhas de configuração, bloqueios ao fechar terminal, logs de jobs e detecção MIME de DEB.

A formatação global encontrou 23 arquivos antigos do jogo embarcado fora do padrão; foram formatados sem alterar a lógica. Os testes desse jogo referiam uma pasta `dist` inexistente; passaram a importar os módulos realmente embarcados em `public/games/vigilia`. Seus 22 testes passaram.

O build web emite avisos existentes de comentários de anotação em Zod e chunk JavaScript acima de 500 kB. São avisos de empacotamento; não foram ocultados.

## Validação final

| Verificação                                | Resultado                                              |
| ------------------------------------------ | ------------------------------------------------------ |
| Prettier global                            | Passou                                                 |
| ESLint / Stylelint                         | Passaram sem erros                                     |
| TypeScript                                 | Passou                                                 |
| Conteúdo                                   | 12 missões, 11 threads, 8 hosts e 303 entradas válidos |
| Cargo fmt / Clippy com warnings como erros | Passaram                                               |
| Testes Rust                                | 129 passaram; benchmark separado da regressão          |
| Benchmark de pacotes                       | Passou com 100, 1.000, 5.000 e 10.000 entradas         |
| Vitest                                     | 201 passaram, em 33 arquivos                           |
| SECTOR IX                                  | 22 passaram                                            |
| E2E nativo                                 | 13 cenários passaram                                   |
| Build web                                  | Passou, com os avisos de empacotamento descritos acima |
| Build Windows release com assets embutidos | Passou; executável de 88.657.920 bytes                 |

Comandos de reprodução na raiz do projeto (ferramentas de desenvolvimento, fora do terminal virtual):

```powershell
pnpm format:check
pnpm lint
pnpm typecheck
pnpm content:check
pnpm test
pnpm build:web
.\.tools\run-rust.ps1 fmt --manifest-path src-tauri/Cargo.toml --all --check
& .\.tools\run-rust.ps1 @('clippy','--manifest-path','src-tauri/Cargo.toml','--all-targets','--all-features','--','-D','warnings')
.\.tools\run-rust.ps1 test --manifest-path src-tauri/Cargo.toml
& .\.tools\run-rust.ps1 @('test','--manifest-path','src-tauri/Cargo.toml','performance_catalog_sizes','--','--ignored','--nocapture')
.\.tools\run-rust.ps1 build --release --features tauri/custom-protocol --manifest-path src-tauri/Cargo.toml
node --test games/vigilia/tests/engine.test.js games/vigilia/tests/animation.test.js
```

O executável de produção é `src-tauri/target/release/game-hacker.exe`, compilado em 16/09/2026 às 01:11, com os assets web embutidos. MSI/NSIS não foram regenerados nesta implementação.

## Principais arquivos alterados

`src-tauri/src/packages/*`; integrações em `world.rs`, `service.rs`, `terminal.rs`, `terminal_sessions.rs`, `vfs.rs`, `binary.rs`, `completion.rs`, `software.rs`, `browser.rs`, `mission.rs`, `mission_runtime.rs`, `archive/{mod,jobs,ipc,downloads,qa}.rs` e registro IPC em `lib.rs`.

Frontend: `src/lib/packages.ts`, `api.ts`, `window-store.ts`, `file-associations.ts`, `file-open.ts`, `src/features/files/PackageInstaller.tsx`, `packages.css`, testes, e integrações em Desktop, KaliMenu, AppIcon, BrowserContent e TerminalWindow. Validação de conteúdo em `scripts/content-schema.mjs`.
