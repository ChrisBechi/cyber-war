# Cyber War 0.4.2 — relatório de correções

Checkpoint de consolidação em 2026-09-14. **A consolidação completa ainda está em andamento.** Este documento separa alterações já implementadas de etapas pendentes; não certifica instaladores nem equivalência com um Linux real.

## 1. Resumo executivo

Implementados o bloqueio de missão simultânea, o encerramento de sessão por IPC, o descarte seletivo de tentativas e contextos independentes de terminal. O fórum foi ampliado com categorias, criação de tópicos, respostas, citações, perfis, pesquisa, acompanhamento, leitura e denúncias locais. Também foram corrigidos logs de boot, restrições do nano, associações de arquivos e menu contextual do desktop. O checkpoint seguinte consolidou oito comandos de arquivos/navegação e a separação entre saída, erro e status, sem modificar o visual do terminal.

O checkpoint atual aprofunda ls/grep/find/permissões, serviços/processos, validação de pacotes, parser/retornos do shell e edição interativa do nano. Implementa armazenamento binário transacional, importação explícita e leitura de bytes nos players. Contratos, testes e limitações detalhados em [APROFUNDAMENTO-0.4.2.md](APROFUNDAMENTO-0.4.2.md).

A orientação mais recente do jogador substitui a concorrência mencionada no pedido inicial: **só uma missão pode estar ativa**. Ordens alternativas continuam possíveis, mas sequencialmente. A compatibilidade com saves antigos que contêm duas tentativas serve apenas para normalizá-los, não para permitir novas missões simultâneas.

## 2. Baseline e problemas encontrados

O relatório de estado atual foi lido integralmente antes das alterações desta consolidação. O ponto inicial está em [BASELINE-CONSOLIDACAO-0.4.2.md](BASELINE-CONSOLIDACAO-0.4.2.md).

Inicialmente: TypeScript falhava em cinco pontos do fórum; ESLint em dois mocks; Stylelint em 236 itens de formatação; 76 testes frontend passavam; Rust compilava, mas um dos 36 testes contradizia o suporte existente a `bash -c`. Faltava registrar `end_session`; o rollback integral apagava alterações pessoais; load e restore recuperavam tentativas; terminais compartilhavam usuário, pasta e SSH.

## 3. Correções realizadas

- Corrigidos os gates do fórum sem redesenhar sua aparência.
- Mantidas as correções E0716 de `man` e `killall/pkill` por variáveis intermediárias com vida útil explícita.
- `end_session` registrado no Tauri e reutilizado pelo fechamento nativo e `quit_game`.
- Navegação de logout aguarda gravação e atualização do estado. Falhas mantêm a tela e as janelas; há possibilidade de repetir a operação.
- Missão única imposta pelo motor, inclusive para solicitações diretas ao backend, não apenas por botão desabilitado.
- `missionBaseline` global substituído por diário de recursos por tentativa.
- Normalização no autosave carregado, save manual e restauração de checkpoint, antes de publicar o estado.
- Contextos de terminal independentes; VFS e mundo continuam compartilhados.
- Entrada seca das linhas de boot, conservando sequência, temporização e deslocamento vertical existente.
- `rm` não usa mais a lixeira; `-f` não ignora erro de permissão nem remove diretório sem recursão.
- `daemon-reload` e opções de shell/nano não implementadas deixam de simular sucesso silencioso nos casos corrigidos.

## 4. Arquitetura de estado e transações

`GameService` continua sendo a autoridade. Uma alteração é aplicada a uma cópia candidata, observada pelo motor, validada e gravada em transação SQLite; só então o estado vivo é publicado. O clone transacional não é usado como baseline de rollback de missão.

`MissionRuntime` contém tentativas, contadores e um diário de recursos. Entradas guardam valores anteriores e efeitos atribuídos, não uma cópia serializada do mundo inteiro. Recursos são identificados por tipo e chave: arquivo/caminho/host, flag, decisão, mensagem, inventário, serviço do cenário ou contador narrativo.

Configurações pessoais, fórum, pacotes e perfis de rede não são incluídos genericamente no rollback. Recursos das missões existentes são declarados a partir das condições/efeitos e do mapeamento explícito das mecânicas de laboratório.

## 5. Missões e tentativas

`MissionEngine::start` recusa qualquer segunda missão enquanto existir uma ativa. A interface mantém outras missões liberadas visíveis, mas informa que é necessário concluir ou abandonar a atual. Uma requisição recusada não grava um novo checkpoint nem modifica progresso.

Abandonar remove etapas e efeitos temporários e devolve a missão à disponibilidade. A próxima tentativa recebe novo identificador e contador incrementado. Gatilhos automáticos não reiniciam imediatamente uma tentativa já abandonada.

Arquivos de missão recebem proveniência em metadados. Cópia, movimentação e operações de lixeira preservam essa identificação; o diário acompanha seus novos caminhos. Ao concluir, a identificação temporária é removida e os resultados ficam duráveis. Arquivos pessoais comuns não recebem essa marca.

O histórico técnico de eventos conserva o registro de tentativa descartada; isso não reativa objetivos nem recompensas.

## 6. Migração de saves

Novos snapshots usam `schemaVersion: 2`. Saves da versão 1 continuam sendo reconhecidos. Para tentativas antigas, o migrador consulta o checkpoint de início ou a referência antiga disponível e extrai apenas os recursos atribuíveis à missão.

Não se restaura o WorldState inteiro. Missões concluídas e alterações pessoais do snapshot selecionado são preservadas. Tentativas antigas simultâneas são descartadas durante a normalização. Se não houver referência suficiente para migrar uma tentativa antiga, o carregamento falha com explicação e não sobrescreve o save original.

O save manual antigo não é apagado só porque o autosave foi normalizado. Ao carregá-lo explicitamente, ele passa pela mesma política. Carregar um snapshot mais antigo continua sendo uma restauração daquele instante, não uma mesclagem de todas as alterações posteriores do usuário.

## 7. Persistência e recuperação

Cobertura adicionada para arquivo e pasta pessoal, ordenação, wallpaper, serviços habilitados, estado de pacotes e acesso Wi-Fi durante tentativa abandonada. Efeitos temporários, mensagens da tentativa e recompensas intermediárias são retirados; resultados de conclusão são promovidos ao save manual.

Save manual durante missão é permitido, mas seu carregamento descarta a tentativa. Uma nova instância de `GameService` sobre o mesmo banco, sem executar logout, simula recuperação após interrupção do processo. Ela também descarta a tentativa antes de disponibilizar o mundo.

Falha SQLite durante saída ou normalização impede publicação parcial. O encerramento é idempotente. Um autosave atrasado após o encerramento não deve reiniciar a campanha.

## 8. Sessão, login e autostart

Login reconstrói o processo-base e os processos dos serviços explicitamente habilitados. Serviços locais anteriormente iniciados manualmente são colocados em inativo. `networking` habilitado depende da conexão virtual.

Janelas são limpas após sucesso da transição. Só a lista permitida de autostart reabre aplicações. Terminais, SSH, buffers nano, histórico e ambiente são transitórios. Bloquear/desbloquear a tela não encerra nem recria a sessão.

A criação de nova campanha não executa um logout artificial antes da primeira entrada, preservando a tentativa inicial de First Boot.

## 9. Terminais independentes

`terminal_open` cria um contexto por `sessionId`; `terminal_close` o remove sem avaliar missões ou salvar buffers. Comando, autocomplete e operações nano recebem o mesmo identificador da janela.

Cada contexto possui cwd, usuário, host SSH, ambiente, histórico e editor/foreground. O adaptador seleciona o contexto sob o mutex do serviço, executa no mundo virtual compartilhado e restaura o contexto padrão. Não há shell ou SSH real.

Testes usam três terminais para verificar diretórios distintos, SSH somente em um deles, privilégio temporário de sudo, variáveis independentes e dois nanos diferentes. Escritas de arquivos continuam visíveis às outras janelas. Conflitos de salvamento preservam a versão mais recente.

Limite atual: 32 contextos e 500 comandos de histórico por contexto. Não há jobs assíncronos ou scheduler de processos equivalente ao Linux.

## 10. Comandos Linux

Contrato implementado e testado para `pwd`, `cd`, `cat`, `head`, `tail`, `cp`, `mv` e `rm`. Flags curtas/longas aceitas, operandos após `--`, efeitos no VFS, saídas e limitações estão em [CONTRATOS-COMANDOS-0.4.2.md](CONTRATOS-COMANDOS-0.4.2.md). Opções fora desse contrato retornam erro explícito.

Navegação respeita HOME e atualiza PWD/OLDPWD por sessão; `cd -` volta e imprime o diretório anterior. Cat mantém numeração entre arquivos, respeita a precedência de `-b`, marcadores de fim/tabs e linhas vazias. Head/tail preservam CRLF e linhas incompletas, aceitam seleção de linhas/bytes, contagens negativas ou origem `+N` conforme o comando, headers e precedência de opções. Recortes de bytes que quebrariam UTF-8 são recusados, não corrompidos silenciosamente.

Cp/mv substituem arquivos regulares com as permissões do VFS; `-n` preserva destinos existentes. Diretórios exigem recursão no cp e mesclagem continua sem suporte. Rm remove permanentemente do VFS, sem lixeira; `-f` ignora somente ausência, não erros de travessia, permissão ou uso de arquivo como diretório. Cp/mv/rm continuam atômicos no jogo, sem reproduzir todos os efeitos parciais do GNU.

O adaptador conserva stdout válido e stderr separado em falhas parciais de cat/head/tail, com status não zero. `sudo false` não vira sucesso. Redirecionamentos vinculam destino ao cwd/host original e preparam a escrita antes do comando; append não exige permissão de leitura. Teste pela fronteira real do serviço confirma que conteúdo redirecionado de uma leitura parcial continua salvo após logout/load. `man` e `--help` dos oito comandos apresentam o mesmo subconjunto auditado; outros manuais se identificam como resumos legados.

Ambiente de `export`/`env`/`printenv` passou para o terminal selecionado. `systemctl daemon-reload` informa ausência de suporte em vez de retornar sucesso vazio.

`ls`, `grep`, `find`, `chmod` e `chown` agora possuem contratos e testes próprios em `terminal_query`. Serviços e processos receberam validação de flags, status e integração entre execução, processos e eventos; apt/apt-get recusam flags fora do subconjunto e operações remotas que alterariam inventário local. SSH/SCP, rede, demais utilitários e semântica completa de pacotes ainda exigem auditoria. Stdin, pipes, mensagens/códigos GNU exatos e falhas/redirecionamento completos não estão implementados. O VFS agora admite blobs, mas os leitores de texto continuam textuais, limitados a 4 MiB de saída, e recusam binários. Foram consultados manuais GNU, mas não executado Linux real para comparação.

## 11. Shell

Executor sequencial virtual, limitado a 512 linhas e oito níveis de inclusão. `bash -c` usa o nome e argumentos posicionais fornecidos. Opções desconhecidas do interpretador são recusadas, e argumentos do script não são filtrados como se fossem flags do interpretador.

O parser novo conserva aspas, escapes, argumentos vazios, `$@`, parâmetros posicionais e variáveis sem reinterpretar os valores como operadores. Atribuições/exportação, `$?`, `exit`/`return` numéricos, saída/erro/status final e isolamento do shell filho têm regressões; source compartilha contexto. `set`, pipelines, operadores lógicos, loops, funções, substituição de comandos e globbing geral permanecem sem suporte. O contrato está no documento de aprofundamento; não equivale a Bash.

Arquivos `.zsh` abrem como texto: a extensão não promete um interpretador Zsh.

## 12. Nano

Editor compatível em parte, não GNU nano real. `--version` e ajuda identificam essa condição. Várias opções antes somente armazenadas agora produzem erro explícito antes de abrir o editor. Múltiplos arquivos são recusados.

Corrigidos: Tab chegava bloqueado à edição; `--nonewlines` removia novas linhas já existentes; restrição de leitura/escrita não correspondia ao arquivo original; `--operatingdir` não protegia o caminho de gravação; backups em diretório não eram numerados. Testes verificam edição, desfazer/refazer, Tab, nova linha final, somente leitura, repetição de save após erro, conflito e restrições.

Regras de nova linha, arquivo original em modo restrito e backups numerados foram conferidas no [manual GNU nano](https://www.nano-editor.org/dist/latest/nano.html#Command_002dline-options). O manual atual não foi usado para afirmar equivalência de toda a interface.

| Situação no editor virtual                             | Opções do parser                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Suporte concreto, com limites do VFS e editor          | `-A/--smarthome`, `-B/--backup`, `-C/--backupdir`, `-E/--tabstospaces`, `-L/--nonewlines`, `-R/--restricted`, `-i/--autoindent`, `-k/--cutfromcursor`, `-l/--linenumbers`, `-n/--noread`, `-o/--operatingdir`, `-v/--view`, `-h/--help`, `-V/--version`                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Parcial; não certificado pixel a pixel/atalho a atalho | `-S/--softwrap`, `-T/--tabsize` (inserção), `-c/--constantshow`, `-w/--nowrap`, `-x/--nohelp`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Aceito como condição já aplicada por padrão            | `-I/--ignorercfiles`: o simulador não lê nanorc                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| Reconhecido, mas recusado explicitamente               | `-D/--boldtext`, `-F/--newbuffer/--multibuffer`, `-G/--locking`, `-H/--historylog`, `-J/--guidestripe`, `-K/--rawsequences`, `-M/--trimblanks`, `-N/--noconvert`, `-O/--bookstyle`, `-P/--positionlog`, `-Q/--quotestr`, `-U/--quickblank`, `-W/--wordbounds`, `-X/--wordchars`, `-Y/--syntax`, `-Z/--zap`, `-z/--listsyntaxes`, `-a/--atblanks`, `-b/--breaklonglines`, `-d/--rebinddelete`, `-e/--emptyline`, `-f/--rcfile`, `-g/--showcursor`, `-j/--jumpyscrolling`, `-m/--mouse`, `-p/--preserve`, `-q/--indicator`, `-r/--fill`, `-s/--speller`, `-u/--unix`, `-y/--afterends`, `-!/--magic`, `-@/--colonparsing`, `-%/--stateflags`, `-_/--minibar`, `-0/--zero`, `-1/--solosidescroll`, `-//--modernbindings` |

Qualquer outra opção não reconhecida retorna erro. Novas regressões cobrem busca com retorno ao início, Unicode, mensagem de não encontrado, substituição confirmada Y/N/A e limitada à seleção, corte/cópia/colagem multilinha, desfazer/refazer M-U/M-E e salvar ao sair com falha/repetição. Não é cobertura exaustiva de atalhos/reflow. Permanecem pendentes buffer sem nome completo, múltiplos buffers, nanorc, regex, corretor externo, confirmação completa de sobrescrita em salvar como e preservação geral de formatos DOS/Mac.

## 13. VFS e associações

Menu contextual do desktop implementado conforme a referência enviada: popup claro, seleção roxa, ícones pequenos, separadores e submenus. Inclui criação de lançador de aplicativo do catálogo, link de URL, pasta, arquivo vazio e script `.sh`; colagem de arquivos do clipboard virtual; terminal na pasta Desktop; gerenciador de arquivos como root; nova janela; organização por nome e configurações da área de trabalho; aplicativos por categoria. Sobre ícones, acrescenta abrir, copiar e recortar. Esc, setas, Shift+F10, clique fora e ajuste aos limites da tela são tratados.

Correção posterior: fechamento externo unificado no desktop, lançador e popups da barra. Apenas o popup e seu botão contam como interior, não a barra inteira. Pointerdown/click em captura permitem fechar mesmo se outra janela interromper propagação, preservando a ação clicada e controles internos. Oito novos testes verificam todos os popups, submenu, volume, alternância e clique sem pointerdown; 152 testes frontend em 27 arquivos aprovados. Nenhum estilo foi alterado.

Criação ocorre em uma única transação Rust: rejeita nomes inválidos, travessia de diretórios, alvos não catalogados e sobrescrita. Atalhos URL abrem exclusivamente no navegador virtual; não há abertura de URLs no host. Lançadores são descritores do simulador, não implementação geral de `Exec`/Desktop Entry do Linux. Documentos oferecem dois modelos fixos; descoberta de modelos em `Templates` não está implementada. A colagem recusa conflitos sem sobrescrever o destino, preservando o clipboard para nova tentativa; diálogo de renomeação automática ainda não existe.

Ordenação fica em `desktopSort` e é salva; clipboard é transitório e limpo na transição de sessão. Janelas de arquivos e editores administrativos têm contexto próprio. `asRoot` é apenas o ator do VFS do jogo, identificado na janela; nunca solicita privilégios do Windows. Terminais novos recebem pasta e usuário próprios; abrir o lançador root não modifica o contexto compartilhado. Desktop e gerenciador usam a mesma abertura por associação, inclusive mídias, scripts e atalhos.

Associações diferenciam texto editável, shell parcial, prévia dependente de conteúdo/codec e formato sem suporte. PDF e arquivos compactados mostram uma mensagem e são preservados, sem abertura automática como texto. A escolha por extensão não garante capacidade de decodificação.

Metadados de mídia não aceitam URLs externas, caminhos do host ou travessia `..`. Recursos demonstrativos locais permanecem; arquivos importados usam bytes do VFS e URLs Blob transitórias, nunca salvas. O MIME do blob prevalece sobre extensão renomeada.

Armazenamento binário implementado com referências SHA-256, tabela `vfs_blobs`, cache compartilhado e gravação na mesma transação do snapshot. Migração SQLite 3; schema do mundo continua 2, com campo opcional aditivo. Importação explícita pelo gerenciador; IPC de leitura e escrita de bytes, validação de tamanho/permissões/conflito. Saves antigos textuais, checkpoints, lixeira e diário de missão têm regressões. Limites: 32 MiB/arquivo, 128 MiB de blobs únicos/snapshot e 512 MiB retidos/banco. Coleta de blobs órfãos, streaming e integração dos utilitários binários legados/SCP ainda estão pendentes.

## 14. Áudio, vídeo, legendas e imagens

Players integrados à leitura binária autorizada, com descarte de respostas atrasadas e revogação de URLs. Testes de componentes cobrem play/pause sem autoplay, seek/saltos, mute/loop/velocidade, volume do player combinado com global, SRT/VTT e modo da track após carregar, erros de decoder/fullscreen e ajuste/rotação de imagens. CSP permite mídia Blob sem fontes externas; SVG/HTML binários não são publicados como documentos ativos. A decodificação efetiva, sincronização, playlists e CSP/fullscreen no WebView nativo ainda não foram certificados.

Não houve alteração da cinematic de abertura. Os testes usam APIs de reprodução simuladas no DOM; não demonstram que todos os codecs funcionam no Windows.

## 15. Serviços, rede e configurações

Inicialização de processos foi corrigida, e estados de execução são separados da habilitação no login. A política de serviços ainda usa as estruturas anteriores de settings; não há unidades arbitrárias, daemon-reload real, cron completo nem PostgreSQL real.

Rede permanece inteiramente virtual: hosts, portas, DNS, SSH, Wi-Fi e arquivos remotos do cenário. Não foram introduzidos sockets, captura real ou comandos externos.

Start/stop/restart e sinais TERM/KILL agora sincronizam processos e estado dos serviços. Journalctl mostra operações registradas, não mensagens de saúde fabricadas. A tipagem progressiva das configurações, dependências, logs mais completos e um gerenciador completo de perfis Wi-Fi ainda precisam ser consolidados.

## 16. Fórum e catálogo

Fórum: 11 tópicos de conteúdo, incluindo os dois narrativos preservados; nove categorias em cinco grupos; membros fictícios; páginas home/categoria/tópico/perfil/diretório/regras/pesquisa/composição; respostas e citações validadas pelo backend; tópicos bloqueados, gates narrativos, estado offline e limites de conteúdo. Estado do usuário permanece por campanha em SQLite via snapshot.

Denúncias são registros locais, não mensagens enviadas a moderadores reais. Não há comunidade online/multiplayer. O catálogo permanece com 303 entradas; não foram acrescentadas ferramentas. A classificação individual A/B/C/D exigida pelo pedido amplo ainda está pendente, e 303 entradas não significam 303 implementações completas.

## 17. Arquivos principais alterados

Núcleo: `world.rs`, `mission.rs`, novos `mission_runtime.rs`, `mission_persistence_tests.rs`, `terminal_sessions.rs`, além de `service.rs`, `save.rs`, `commands.rs`, `lib.rs`, `terminal.rs`, `nano.rs`, testes da campanha e de investigação.

Comandos de arquivos/navegação: novos `src-tauri/src/terminal_io.rs` e `src-tauri/src/terminal_io_tests.rs`, integração em `terminal.rs`/`lib.rs` e validação de existência/preparação de redirecionamento em `vfs.rs`. Documentação de contratos, terminal e API atualizada; nenhum componente ou CSS do terminal alterado nesta etapa.

Aprofundamento: novos `terminal_query.rs`, `terminal_query_tests.rs`, `shell.rs`, `shell_tests.rs`, `binary.rs`, `binary_tests.rs`; atualizações em VFS, world, db, save, commands, Cargo e testes de missão. Frontend: `nano.ts`, `use-media-source.ts`, `MediaPlayer.tsx`, `ImageViewer.tsx`, importação no gerenciador, contratos/associações/audio-manager, CSP e testes. Contratos consolidados em `APROFUNDAMENTO-0.4.2.md` e specs 03/04/07/08/12.

Interface: `App.tsx`, `App.session.test.tsx`, `lib/session.ts`, `lib/game-store.ts`, `TerminalWindow.tsx`, `nano.ts`, novos testes do nano e de missões, `Desktop.tsx`, `Saves.tsx`, `Missions.tsx`, `FileManager.tsx`, associações e `styles/boot.css`.

Menu do desktop: `DesktopContextMenu.tsx`, `DesktopCreateDialog.tsx`, `desktop-context.css`, `Desktop.context.test.tsx`, `lib/vfs-clipboard.ts`, `lib/file-open.ts`, `lib/window-store.ts`, `Editor.tsx`, `Browser.tsx`, `AppWindow.tsx` e `src-tauri/src/desktop.rs`. IPC e testes atualizados para criação, atores virtuais e contexto inicial dos terminais.

Fórum: componentes/modelo/CSS/testes em `src/features/forum/`, adaptador desktop, `src-tauri/src/forum.rs`, conteúdo e validadores em `content/forums/` e `scripts/`. A prévia isolada em `artifacts/forum-preview.*` não é um backend substituto de produção.

## 18. Testes e gates

Os resultados finais deste checkpoint são atualizados abaixo após a última execução; contagens anteriores não substituem regressão final.

- Baseline: 76 frontend aprovados; Rust 35/36, conforme documento separado.
- Primeira regressão crítica: 101 frontend aprovados e 45 Rust aprovados.
- Regressão intermediária: 49 Rust aprovados; Clippy com `-D warnings` aprovado; TypeScript e lint aprovados.
- O teste novo de Tab falhou, expôs o descarte do caractere no frontend e levou à correção da implementação. O teste não foi removido nem teve a expectativa enfraquecida.
- Checkpoint do desktop: 130 testes frontend em 25 arquivos e 53 testes Rust aprovados. Inclui menu contextual, nomes de scripts com aspas e reabertura de atalhos URL após navegar para outra página.
- Regressão final do checkpoint de comandos: 130 testes frontend em 25 arquivos e 70 testes Rust aprovados, incluindo 17 novos testes de contratos, permissões, flags, saídas exatas do subconjunto, erros parciais, limite de saída, SSH e persistência. TypeScript, ESLint, Stylelint, Prettier, Cargo fmt e Clippy com `-D warnings` aprovados. Conteúdo validado: 12 missões, 11 tópicos, 2 hosts e 303 entradas padrão do Kali.
- Regressão do aprofundamento: 144 testes frontend em 27 arquivos e 88 testes Rust aprovados. Inclui contratos de busca/permissões, scripts/retornos/exportação, proteção contra reinterpretação de variáveis, serviços/processos, binários/deduplicação/transações/corrupção, restauração de bytes no journal de missão, nano e mídia/importação.
- Gates finais do aprofundamento aprovados: TypeScript (também reexecutado pelo build), ESLint/Stylelint, Prettier, validação de conteúdo (12 missões, 11 tópicos, 2 hosts, 303 entradas), Cargo fmt e Clippy all-targets/all-features com `-D warnings`. As suites completas foram repetidas após os ajustes; build web atualizado com 227 módulos. Não houve QA visual/nativo ou novo instalador.

Comandos reproduzíveis: `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm content:check`, `pnpm test`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, `cargo clippy --offline --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --offline --manifest-path src-tauri/Cargo.toml`.

## 19. Builds e instaladores

Build web reexecutado e aprovado no aprofundamento (`pnpm build:web`, TypeScript + Vite, 227 módulos). O aviso de bundle JavaScript acima de 500 kB e os avisos de anotações da dependência Zod são conhecidos e não foram tratados como erros de compilação. Os arquivos em `dist/` correspondem a este build web, não a um instalador nativo. As alterações Rust foram verificadas por compilação, testes e Clippy, mas ainda não incorporadas a um novo instalador.

Nenhum executável/NSIS/MSI anterior deve ser tratado como build desta consolidação. A geração de novos instaladores de distribuição permanece pendente até fechar as etapas restantes e gates. Não houve teste em máquina Windows limpa. Android/iOS têm recursos de ícone, mas não APK/AAB/IPA produzidos neste checkpoint.

## 20. Limitações conhecidas e validação visual

A autorização para abrir a prévia do fórum falhou por limite de uso da revisão automática. Não houve contorno por outra ferramenta. Assim, testes de componentes não são apresentados como inspeção visual ou ensaio nativo.

Além das pendências das seções anteriores, a política de recursos temporários precisa acompanhar futuras mecânicas: novos efeitos não podem gravar progresso fora do escopo declarado. Saves antigos incompletos sem referência de início podem exigir recuperação manual. Compatibilidade integral com Bash, GNU coreutils, GNU nano, systemd e formatos de mídia não foi alcançada.

## 21. Próximos checkpoints

1. Auditar utilitários/rede/SSH/SCP e completar integração binária de stat/file/checksums, sem ampliar o catálogo.
2. Fechar lacunas do nano: salvar como, formatos de linha, Unicode/grafemas e layout de tabulação/reflow; ampliar shell somente conforme necessidade do jogo.
3. Validar codecs/legendas/fullscreen no WebView e projetar coleta segura de blobs considerando todos os snapshots e journals.
4. Tipar configurações/serviços, completar perfis de rede e classificar as 303 ferramentas.
5. Fazer auditoria final de branding/ícones, QA visual/nativo, regressão e builds novos, seguida de teste dos instaladores em Windows limpo.

Sem novas missões, expansão de campanha ou trabalho adicional na cinematic de abertura nesta consolidação.
