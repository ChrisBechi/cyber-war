# Cyber War — relatório detalhado do estado atual

Data: 14/09/2026. Versão declarada: 0.4.2.

Este relatório considera o código e os recursos presentes no projeto, a validação do conteúdo e os testes executados nesta auditoria. Funcionalidades descritas apenas nas especificações não foram contabilizadas como implementadas. A alteração realizada junto desta auditoria foi a atualização dos ícones Android/iOS; as pendências de programação identificadas abaixo foram registradas para correção.

## 1. Diagnóstico geral

O projeto já possui uma base expressiva de jogo de investigação com computador virtual: desktop, instalador encenado, terminal, arquivos compartilhados entre aplicativos, rede simulada, missões, mensagens e salvamento. A interface e o conteúdo inicial estão mais desenvolvidos do que a fidelidade dos comandos Linux e a integração dos recursos recentes.

Minha avaliação é de um protótipo avançado com um recorte de campanha implementado. Ainda não considero a versão atual pronta para distribuição: a compilação Rust está bloqueada por dois erros, há uma chamada de encerramento de sessão sem implementação na fronteira da interface e existem diferenças importantes entre a persistência desejada e a persistência efetiva.

O computador do jogo é um modelo de dados interpretado pelo próprio aplicativo. Não há kernel Linux, Bash, systemd, GNU nano, Xfce ou os executáveis do catálogo Kali rodando dentro dele. Portanto, ter o nome, o ícone e uma janela de uma ferramenta não significa que todas as funções dessa ferramenta estejam implementadas.

Inventário verificado:

- 93 arquivos TypeScript/TSX na interface.
- 21 arquivos Rust no núcleo, incluindo módulos de testes.
- 19 arquivos de testes da interface, com 70 testes aprovados nesta auditoria.
- 33 funções de teste Rust identificadas; a execução está bloqueada pela compilação.
- 94 nomes na lista principal de comandos do terminal, incluindo aliases e comandos do jogo.
- 303 entradas no catálogo padrão de aplicativos Kali.
- 12 definições de missão, 2 tópicos de fórum e 2 hosts na configuração inicial. A investigação Orion acrescenta conteúdo ao mundo durante a campanha.
- Cinco slots de campanha, com save manual, autosave e checkpoints.

| Sistema                      | Estado atual                                | Principal limite                                           |
| ---------------------------- | ------------------------------------------- | ---------------------------------------------------------- |
| Abertura, menus e instalador | Bastante desenvolvido na interface          | Instalação virtual; efeito do boot ainda diverge do pedido |
| Desktop e janelas            | Funcional na interface                      | Aplicativos e sessões não têm isolamento de um SO real     |
| Arquivos e gerenciador       | Implementação substancial                   | Conteúdo textual; falta armazenamento binário geral        |
| Terminal Linux               | Amplo conjunto de comandos parciais         | Sintaxe, flags e resultados não têm fidelidade completa    |
| Nano                         | Editor interativo implementado parcialmente | Muitas opções são aceitas sem comportamento correspondente |
| Scripts shell                | Executor sequencial básico                  | Não implementa a linguagem Bash completa                   |
| Pacotes e serviços           | Estado virtual persistente                  | Não instala binários nem executa serviços reais            |
| Rede e SSH                   | Rede de cenários implementada               | Hosts e comportamentos previamente definidos               |
| Ferramentas Kali             | Catálogo amplo e laboratórios               | Pouca profundidade específica por ferramenta               |
| Campanha                     | Primeira sessão e início da segunda         | Continuação da história e sistemas mais amplos incompletos |
| Persistência                 | Base transacional implementada              | Encerramento e rollback de missão precisam correção        |
| Áudio, vídeo e imagens       | Controles básicos e intermediários          | Mídia arbitrária, codecs e recursos avançados incompletos  |
| Mobile                       | Ícones atualizados                          | Aplicativos Android/iOS ainda não inicializados/validados  |
| Distribuição                 | Configuração Windows existente              | Fonte atual não compila; binários precisam nova validação  |

## 2. Arquitetura e organização

A interface usa React 19, TypeScript, Zustand, CSS e xterm.js, com Vite para desenvolvimento e empacotamento. O núcleo usa Rust e Tauri 2; SQLite armazena campanhas e preferências. Zod valida respostas na interface, enquanto Serde representa os dados no núcleo.

O WorldState concentra arquivos, hosts, usuário, terminal, processos, mensagens, missões, decisões, evidências, dinheiro, reputação, técnicas aprendidas e configurações. As operações normais passam pelo GameService: ele copia o mundo, aplica a ação, avalia efeitos de missão, valida o resultado e grava a transação antes de publicar o novo estado. Essa organização é uma boa base para impedir que uma falha de gravação deixe o estado em memória diferente do estado salvo.

Os módulos de arquivos, rede, terminal, missões, instalação, investigação, software e persistência estão separados. A interface tem componentes próprios para cada aplicativo e uma camada comum de comunicação com o Rust.

O que ainda está cru: configurações de naturezas diferentes compartilham um mapa de strings; parte dos dados é JSON dentro desse mapa. O terminal concentra muitas responsabilidades em um arquivo extenso. Há nomes internos herdados, como Game Hacker e LifeOS, convivendo com Cyber War. A documentação e os artefatos antigos nem sempre refletem o código atual.

Referências: [WorldState](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/world.rs>), [GameService](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/service.rs>), [API da interface](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/lib/api.ts>).

## 3. Abertura, identidade visual e menus

Estão implementados a apresentação do estúdio, os textos de boot, a abertura cinematográfica com arquivos locais, o título, a tela de início, o menu principal, as configurações, as instruções e a seleção dos cinco slots. Há controles para pular introduções, preferências de áudio, movimento reduzido e alto contraste.

A logo enviada pelo usuário já está referenciada na abertura e na tela de início. O favicon e os recursos Windows também usam a nova identidade no código-fonte. Nesta auditoria foram atualizados os 15 PNGs Android e os 18 PNGs iOS. Os tamanhos foram verificados, o fundo adaptativo Android foi definido como preto e os arquivos iOS estão em RGB sem canal alfa. Os ícones anteriores foram preservados em uma pasta de backup.

Pendência confirmada: a entrada lateral dos textos de boot ainda existe. A regra boot-log-arrive mantém deslocamento horizontal de 28 pixels até zero, com transição de opacidade por 180 ms. Assim, o pedido anterior de aparecimento imediato ainda não está atendido no CSS auditado.

Existem arquivos SVG antigos de branding e um arquivo ICNS legado. Eles não devem ser confundidos com os recursos ativos da versão Windows. A atualização de arquivos-fonte não atualiza um executável já compilado.

Referências: [recursos da abertura](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/features/boot/boot-assets.ts>), [animação pendente](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/styles/boot.css:1007>), [registro dos ícones móveis](<C:/Users/chris/OneDrive/Documentos/Cyber war/artifacts/mobile-icons-2026-09-14/README.md>).

## 4. Instalação e login do sistema virtual

O configurador tem etapas para idioma, localização, teclado, mídia, rede, domínio, hostname, conta, senha, relógio/fuso, discos, particionamento, seleção de desktop, ferramentas e exibição. Há ajuda contextual, validações, opções guiadas e manuais e telas que encenam preparação, formatação e instalação.

As escolhas de rede e partições alimentam o mundo virtual e aparecem em comandos como ip, ifconfig e lsblk. Existem opções de armazenamento simples, LVM, criptografado, RAID e iSCSI na configuração. O núcleo valida o plano de partições e representa dados como pontos de montagem e fstab.

O login apresenta a identidade Kali, exige o usuário/senha configurados e possui tratamento de erro. Preferências de instalação são incorporadas ao save para não depender apenas da conclusão da primeira missão.

O que ainda está cru: selecionar GNOME ou KDE não significa executar esses ambientes; o desktop continua sendo a interface própria. Criptografia, RAID, LVM e instalação de pacotes representam escolhas e efeitos simulados, sem esses subsistemas completos. A senha de login fica no estado salvo e a comparação é feita na interface; trata-se de autenticação encenada, não de isolamento real entre contas. Não há um gerenciador completo de usuários Linux.

## 5. Desktop, painel e gerenciamento de janelas

Existem menu de aplicativos com pesquisa, categorias, favoritos e recentes; painel Kali; quatro áreas de trabalho; relógio/calendário; indicadores de rede, áudio e notificações; bloqueio; logout; wallpapers; atalhos e tela cheia.

As janelas podem abrir, fechar, receber foco, mudar de posição e tamanho, minimizar, maximizar e restaurar. A ordem de sobreposição e a área de trabalho são controladas. É possível abrir mais de uma janela de terminal. Os arquivos em Desktop originam ícones do desktop, com abertura por duplo clique e operações de arrastar relacionadas aos arquivos/lixeira.

A entrada normal no desktop limpa as janelas e reabre somente os aplicativos configurados para iniciar: Terminal, Arquivos, HackPad, Navegador e Mensagens. Essa lista ainda não aceita qualquer ferramenta do catálogo.

O que ainda está cru: janelas são componentes da interface, não processos independentes. Duas janelas de terminal compartilham o contexto de terminal armazenado no mundo, inclusive diretório, usuário e host. A organização manual por coordenadas dos ícones do desktop não tem persistência identificada; o ordenamento persistente implementado é o do gerenciador de arquivos. Indicadores visuais de atividade não constituem um monitor de recursos real.

Não restaurar janelas abertas após novo login está alinhado à preferência mais recente do usuário. Bloquear/desbloquear a sessão tem comportamento diferente de encerrar e iniciar uma nova sessão.

## 6. Ícones Kali, pastas e lixeira

O AppIcon encaminha pastas para folder.svg e a lixeira para user-trash.svg ou user-trash-full.svg, conforme seu conteúdo. Os arquivos existem no diretório de recursos Kali. Terminal, navegador, gerenciador de arquivos, editor e parte dos utilitários também usam recursos desse catálogo. Aplicativos próprios ainda possuem desenhos locais de fallback.

Os avisos de terceiros registram projetos oficiais e commits dos temas, além das licenças. Isso estabelece a origem documentada do conjunto, mas não é uma auditoria completa, ícone a ícone, de equivalência à edição Kali 2026.1. Não considero essa equivalência exata comprovada apenas porque o recurso está na pasta kali.

Referência: [mapeamento de ícones](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/features/desktop/AppIcon.tsx:134>).

## 7. Sistema de arquivos e gerenciador

O VFS representa diretórios e arquivos com caminho, conteúdo, proprietário, grupo, permissões, metadados e informações de modificação. Há estrutura Linux inicial, pastas pessoais, arquivos de configuração, logs, área de projetos e lixeira.

Existem operações de criação, leitura, escrita, cópia, movimentação, remoção, alteração de permissões e propriedade. A escrita do editor pode conferir a versão anterior para impedir sobrescrita silenciosa de conteúdo alterado por outro aplicativo. A normalização de caminhos considera caminhos relativos, absolutos e o diretório pessoal, com validação para manter as operações no mundo virtual.

O gerenciador permite navegar, criar arquivos/pastas, abrir conteúdo, copiar, mover, renomear, enviar para a lixeira, restaurar e excluir definitivamente. A ordenação por nome, modificação ou tipo, a direção crescente/decrescente e a preferência por pastas primeiro são salvas. A ordenação por nome usa comparação natural pt-BR.

Limites atuais: aproximadamente 1 MiB de conteúdo por arquivo e 10 mil nós. O conteúdo é textual; não existe armazenamento geral de bytes para imagens, vídeos, executáveis ou arquivos compactados. Não há links simbólicos reais, montagem de volumes ou implementação completa de inodes, ACLs e todos os detalhes de permissões Linux.

A GUI de arquivos opera no VFS local. O terminal e o nano podem operar no VFS remoto selecionado por SSH. Isso precisa ser considerado ao avaliar se dois aplicativos estão mostrando o mesmo computador.

Referência: [VFS](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/vfs.rs>).

## 8. Extensões e associação de arquivos

Existe uma camada que reconhece extensão e, em alguns casos, metadados MIME para escolher o aplicativo. Arquivos de texto abrem no HackPad; scripts shell abrem no terminal; áudio/vídeo abre no player; imagens abrem no visualizador.

São reconhecidas extensões comuns de código e configuração, áudio, vídeo, imagem, arquivos compactados e PDF. O reconhecimento diferencia maiúsculas/minúsculas de forma normalizada.

O que ainda está cru: a associação é fixa no código. Não há painel completo para escolher aplicativos padrão, registrar associações por usuário ou implementar a infraestrutura MIME/desktop do Linux. Reconhecer .mkv, .avi, .tiff ou .pdf não garante que o conteúdo possa ser reproduzido/renderizado. PDFs e arquivos compactados ainda são encaminhados ao editor, apesar dos rótulos descritivos; não há leitor PDF ou compactador completo.

Referência: [associações de arquivos](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/lib/file-associations.ts>).

## 9. Terminal e comandos Linux

A interface usa xterm.js, com prompt, edição da linha, histórico, navegação pelo cursor, autocomplete de comandos/caminhos, ajuste ao tamanho da janela e suporte ao modo interativo do nano. Os comandos retornam saída, erro, código de saída e o contexto virtual atualizado.

Estes são os 94 nomes da lista principal; a presença nesta tabela significa que há tratamento no código, não conformidade completa com a ferramenta Linux:

| Família                         | Comandos                                                                                                                     |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Sessão, shell e utilidades      | help, pwd, cd, echo, clear, sh, bash, source, true, false, yes, exit, lab                                                    |
| Identidade e ambiente           | whoami, id, groups, uname, hostname, date, env, printenv, export, sudo, su                                                   |
| Arquivos, caminhos e permissões | ls, mkdir, touch, cp, mv, rm, basename, dirname, realpath, readlink, which, whereis, tree, stat, chmod, chown, df, du, lsblk |
| Texto e inspeção                | cat, head, tail, wc, sort, uniq, cut, tr, seq, grep, find, less, edit, nano, file, strings, base64, sha256sum, man           |
| Pacotes                         | apt, apt-get, apt-cache, apt-mark, dpkg, dpkg-query                                                                          |
| Rede                            | ip, ifconfig, ipconfig, ping, route, arp, ss, netstat, dig, nslookup, traceroute, curl, wget, ssh, scp, nmap                 |
| Processos e serviços            | ps, top, kill, killall, pkill, free, uptime, service, systemctl, journalctl                                                  |

Há suporte a aspas e redirecionamento simples para criar ou acrescentar conteúdo em arquivo. Muitas flags comuns alteram o resultado, como seleção de linhas, opções de listagem e ordenação. Arquivos escritos pelo terminal ficam disponíveis nos demais aplicativos.

As limitações são relevantes para o pedido de simular exatamente os comandos:

- Não há pipelines, operadores lógicos, execução em segundo plano ou substituição de comandos. A tokenização rejeita vários operadores do shell.
- Não existe expansão geral de variáveis e curingas equivalente ao Bash interativo.
- grep usa procura simplificada; find não implementa a linguagem completa de expressões/predicados.
- tr trabalha com argumentos próprios para texto, em vez de uma implementação geral baseada em stdin. Outros filtros também têm interfaces limitadas.
- df, free, top, uptime e parte das saídas de hardware usam valores virtuais simplificados. date não implementa todos os formatos e a mesma semântica de um relógio Linux.
- ssh recebe credenciais por uma sintaxe virtual simplificada e mantém um único contexto remoto no mundo.
- O rm local envia arquivos para a lixeira virtual. Isso diverge da remoção comum pelo rm do Linux.
- As flags, mensagens de erro e códigos de saída não foram comparados exaustivamente com os programas de referência.
- man produz manuais resumidos do simulador.

Há ainda dois erros de empréstimo de valores temporários no Rust, nos caminhos de man e killall/pkill, que impedem compilar o núcleo inteiro.

Referência: [implementação do terminal](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/terminal.rs>).

## 10. Scripts .sh, Bash e source

O executor lê scripts do VFS e executa linhas sequencialmente. Reconhece shebang/comentários, argumentos posicionais, substituições simples de variáveis, source, chamada com -c e interrupção por exit/return. Há limite de 512 linhas e controle de profundidade nas inclusões via source.

Isso permite automações simples com os comandos já implementados, como criar pastas, escrever arquivos e executar operações do cenário.

O que ainda está cru: não há linguagem completa de shell com if, case, for, while, funções, arrays, expansões e pipelines. A substituição de variáveis é textual e não reproduz todas as regras de aspas. Opções de set são ignoradas nesse executor. Uma linha que falha interrompe o script, sem reproduzir todas as regras de tratamento de erro e retorno do Bash. O shebang não carrega um interpretador arbitrário, e a extensão .zsh não implica execução de Zsh.

## 11. Nano e HackPad

O nano tem interface interativa dentro do terminal, cabeçalho, área de texto, atalhos, mensagens, cursor e prompts. Estão implementados edição, navegação, salvar, sair com confirmação de alterações, busca/substituição, leitura de outro arquivo, ir para linha, marcação, corte/colagem e desfazer/refazer.

Há comportamentos concretos para opções como somente leitura, numeração de linhas, tamanho de tabulação, tabs para espaços, autoindentação, quebra visual, posição do cursor, backups, restrição de diretório e regras de nova linha. A escrita passa pelo VFS e pode detectar conflito de conteúdo.

O parser aceita uma superfície extensa de opções e anuncia GNU nano 8.7. Entretanto, várias opções só são armazenadas. Não há execução completa de nanorc, corretor ortográfico, integração com ferramentas externas, histórico em disco, locking de arquivos e todos os comportamentos de múltiplos buffers, mouse e sintaxe do nano original. O próprio editor informa que o corretor ortográfico está indisponível.

A classificação correta é editor compatível em parte, com aparência e atalhos familiares. Não é possível dizer que está exatamente igual ao GNU nano, nem que todas as opções funcionam.

O HackPad é um editor separado, com abertura, alteração, salvamento, Ctrl+S e detecção de conflito. A edição utiliza textarea. Não há Monaco, análise de linguagem, debugger, extensões ou IDE completa.

Referências: [parser nano](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/nano.rs>), [editor nano interativo](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/features/terminal/nano.ts>).

## 12. Pacotes, serviços e inicialização automática

apt e comandos relacionados representam catálogo, pacotes instalados, consulta e mudanças de estado. Existem operações de atualização/instalação/remoção e consultas de metadados, com verificações de usuário para operações administrativas. O estado fica salvo no mundo.

O que ainda está cru nos pacotes: instalação não baixa nem instala executáveis Linux. Não existe um resolvedor completo de dependências, scripts de instalação, repositórios, assinatura de pacotes ou execução dos programas reais. A consulta dpkg tem um subconjunto de opções.

Os serviços locais previstos incluem apache2, cron, networking, postgresql, ssh e web. service/systemctl possuem operações como start, stop, restart, status, enable e disable. O estado de execução é separado da lista de serviços habilitados para o próximo login.

No início da sessão, os serviços locais são colocados em inativo; apenas os explicitamente habilitados são reativados. O serviço networking depende também do estado de conexão. A configuração dos aplicativos de autostart usa uma lista permitida de cinco aplicativos.

O que ainda está cru nos serviços: são estados simulados. Criar um arquivo .service não cria um serviço executável; daemon-reload não carrega unidades; cron não é um agendador completo; postgresql não implementa um banco PostgreSQL. O journalctl sintetiza informações, sem o comportamento de um journal real. Serviços remotos têm regras específicas do cenário e não são todos reiniciados pela política da sessão local.

## 13. Rede, Wi-Fi, HTTP e SSH

O mundo contém hosts, endereços, DNS, portas, serviços, credenciais, firewall, disponibilidade e arquivos remotos. Os comandos consultam esses dados e respeitam diversas condições, como rede desconectada, host desconhecido, porta bloqueada e credenciais inválidas.

SSH muda o contexto do terminal para o host virtual. SCP copia conteúdo de um host do cenário. HTTP/HTTPS, curl e wget acessam os serviços previstos no mundo. Existem redes Wi-Fi com SSID, canal, sinal, criptografia e condição de acesso; as escolhas e acessos são serializados com a campanha.

A investigação Orion acrescenta inspeção de sinais, captura representada por dados, filtros e acompanhamento de fluxo. Wireshark e aircrack-ng possuem interfaces específicas para esse conteúdo.

O que ainda está cru: não há pilha TCP/IP, DHCP, captura real, interfaces físicas ou tráfego externo. A rede inicial é pequena e autoral. Não há gerenciador abrangente de perfis Wi-Fi salvos, roteamento dinâmico, múltiplas sessões SSH independentes ou servidor genérico configurável como num Linux. A análise de pacotes lê o formato do cenário, não um decodificador completo de PCAP e protocolos.

Referências: [rede virtual](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/network.rs>), [investigação](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/investigation.rs>).

## 14. Catálogo Kali e laboratórios

As 303 entradas incluem nome, pacote, comandos associados, descrição, categoria, ícone e referência. Pesquisa, favoritos e recentes estão integrados ao menu.

Muitas ferramentas abrem um workspace genérico que inspeciona hosts, web, arquivos, Wi-Fi ou processos. Os relatórios utilizam dados do mundo e podem ser gravados no VFS, com proteção contra sobrescrita de relatórios existentes.

O catálogo é amplo; a implementação individual ainda é rasa. Para diversas ferramentas, a operação útil é --lab e a ajuda explica que as opções originais não são emuladas. Não há 303 ferramentas completas. Ferramentas com marca conhecida podem compartilhar exatamente o mesmo tipo de laboratório por baixo da interface.

O CodeLab possui um exercício específico de scanner de memória: comparar valores, reduzir candidatos, inspecionar uma rotina e construir soluções V1/V2. É uma mecânica narrativa com estados previstos, sem compilador, debugger ou scanner genérico de processos.

## 15. Navegador, fórum, mensagens e aplicativos auxiliares

O navegador apresenta endereços virtuais, navegação e ações narrativas. Há Wipédia local, Archive, B1, FakeBook, Mercado e páginas Blackwire. Ações como download, compra e recuperação alteram arquivos, flags, técnicas e inventário.

O que ainda está cru no navegador: as páginas são implementações específicas, não uma web completa. Não há navegação externa, engine de páginas arbitrárias do jogo, autenticação web geral ou downloads de bytes genéricos.

O mensageiro possui contatos, conversas, mensagens recebidas e enviadas, marcação de leitura e notificações. O histórico fica no mundo salvo. Respostas livres são registradas, mas não constituem um sistema geral de diálogo inteligente; a progressão depende das regras e escolhas escritas para a história.

O fórum mostra dois tópicos com conteúdo e respostas previamente definidos, liberados conforme flags. Ainda não oferece criação geral de tópicos e uma comunidade dinâmica.

A calculadora possui interface e operações aritméticas. Technical Journey lista técnicas e inventário. O monitor de processos mostra PID, nome, usuário e estado; não tem um escalonador nem métricas independentes de consumo por aplicativo. São utilitários funcionais dentro de escopos pequenos.

## 16. Áudio, vídeo, legendas e imagens

O player identificado como Parole tem reprodução/pausa, barra de progresso, busca temporal, avanço/retorno de dez segundos, volume próprio, mute, repetição, velocidade de 0,5× a 2×, tela cheia e atalhos de teclado. Vídeos procuram uma legenda .vtt de mesmo nome e tentam .srt como alternativa; há conversão simples para VTT e controle de exibição.

O visualizador identificado como Ristretto tem ajuste à janela, zoom de 25% a 500%, rotação em passos de 90°, tela cheia e atalhos.

São recursos concretos, porém ainda não constituem reprodutores completos. O conteúdo reproduzível depende de uma fonte de mídia nos metadados ou de arquivos demonstrativos associados a recursos locais do aplicativo. Um arquivo textual criado com nome musica.mp3 não vira áudio. O VFS não possui armazenamento binário geral. Decodificação depende do navegador/WebView, e a extensão reconhecida pode não ter codec disponível.

Ainda faltam playlist/biblioteca, seleção de faixas de áudio, múltiplas legendas selecionáveis, ajustes de sincronização, tratamento amplo de formatos e testes dos controles em execução nativa. As legendas usam URLs data:, cujo comportamento com a política de conteúdo da versão empacotada precisa ser ensaiado. O player não integra seu volume próprio ao mesmo estado persistido do volume do painel. Posição e estado de reprodução são transitórios, coerentes com iniciar uma nova sessão limpa.

O visualizador não oferece edição/salvamento da imagem, recorte, organização de biblioteca ou suporte próprio a todos os formatos reconhecidos.

Referências: [player](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/features/media/MediaPlayer.tsx>), [visualizador](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/features/media/ImageViewer.tsx>).

## 17. Missões, narrativa e progressão

O motor usa definições JSON com requisitos, gatilhos, etapas, condições, escolhas e efeitos. Condições podem consultar arquivos, flags, técnicas, inventário, dinheiro, reputação, decisões e serviços. Efeitos podem criar arquivos, enviar mensagens, conceder recompensas, registrar evidências e mudar estados do mundo.

As doze missões presentes são:

1. First Boot.
2. Só pra relaxar — V1.
3. Patched / V2.
4. Sem Internet.
5. O Pendrive.
6. Voltou.
7. VEX · Do zero.
8. VEX · Produção.
9. Alguma coisa está errada.
10. A Garota.
11. NO AR — início de Orion, sessão 2.
12. EM CLARO — continuação inicial de Orion.

Há pistas, soluções alternativas previstas, progressão por efeitos observados, recompensas, consequências e checkpoints. O teste de campanha existente percorre o recorte principal e cenários alternativos, mas não pôde rodar nesta revisão por causa do erro de compilação.

O que ainda está cru: a campanha completa não está implementada. A sessão 2 tem essas duas missões iniciais; os arcos mais amplos e a sessão 3 permanecem principalmente em documentação. Dinheiro, reputação, inventário, evidências e heat existem como campos/efeitos, sem necessariamente formar sistemas econômicos, investigativos ou de reação completos. Technical Journey apresenta o registro de técnicas, sem uma camada extensa de progressão própria.

A operação chamada abortar tentativa incrementa tentativas e reinicia o contexto do terminal, mas não restaura por si só todos os efeitos e etapas da missão. O nome da ação pode sugerir uma reversão maior do que a implementada.

Referência: [motor de missões](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/mission.rs>).

## 18. Salvamento e comportamento ao entrar/sair

Existem cinco slots independentes, saves manuais, autosaves após operações, autosave periódico no desktop, checkpoints de início/fim/decisão e limite de checkpoints por slot. SQLite usa transações e os snapshots têm checksum SHA-256. As preferências globais do aplicativo ficam separadas do mundo da campanha.

O código tenta consolidar uma missão concluída como marco durável e guardar um estado anterior ao iniciar uma missão. Ao fechar normalmente a janela nativa, o GameService aplica a regra de encerramento; uma falha de gravação impede o fechamento e comunica erro.

| Informação                                               | Comportamento implementado                                           |
| -------------------------------------------------------- | -------------------------------------------------------------------- |
| Arquivos, pastas e metadados                             | Salvos no mundo; podem ser revertidos ao abandonar missão            |
| Rede, acesso Wi-Fi e configurações da campanha           | Salvos no mundo; sujeitos à mesma reversão                           |
| Pacotes e serviços habilitados                           | Salvos no mundo; sujeitos à mesma reversão                           |
| Serviços locais em execução                              | Reinicializados no login conforme a lista de habilitados             |
| Ordenação do gerenciador, wallpaper e fonte              | Preferências salvas na campanha; sujeitas à mesma reversão           |
| Preferências globais de áudio, exibição e acessibilidade | Salvas separadamente em app_preferences                              |
| Janelas abertas e geometria                              | Estado da interface; novo login limpa as janelas                     |
| Autostart                                                | Reabre apenas os cinco aplicativos permitidos que foram selecionados |
| Mensagens, decisões e missões concluídas                 | Salvas com o estado durável da campanha                              |
| Missão em andamento                                      | Tentativa é revertida no caminho de fechamento normal implementado   |
| Buffer não salvo do editor/nano e posição do player      | Não são restauração completa de sessão                               |

Há cinco problemas importantes:

1. Encerrar sessão pela GUI está incompleto. Desktop e Campanha chamam end_session, mas não há comando Tauri correspondente em commands.rs nem registro no invoke_handler. O método interno existe e é chamado pelo fechamento nativo; isso não torna o botão da GUI funcional.
2. O rollback usa uma cópia inteira do WorldState. Arquivos, configurações, redes e outros dados alterados depois do início da missão também voltam atrás. A regra ainda não distingue progresso da missão de alterações pessoais que o usuário quer manter.
3. A referência missionBaseline é global. O motor não impede múltiplas missões ativas; iniciar outra pode substituir a referência, e concluir uma remove a referência global. A consistência de missões simultâneas precisa ser resolvida e testada.
4. Save manual pode guardar uma missão ativa. O caminho de restauração manual permite recuperar esse estado, o que cria uma exceção à regra de sempre perder progresso da missão ao sair.
5. Fechamento forçado, falha do processo ou interrupção de energia não passam necessariamente pelo encerramento. Como load carrega o snapshot sem uma normalização equivalente do progresso ativo, um autosave intermediário pode ser recuperado. A regra de descarte ainda não é uniforme em todos os caminhos de saída/entrada.

Além disso, session_start reinicia serviços locais e o contexto do terminal, mas não reconstrói de modo geral a lista de processos do mundo. Ainda não há garantia de uma sessão totalmente limpa em todos os subsistemas.

Referências: [chamada da GUI](<C:/Users/chris/OneDrive/Documentos/Cyber war/src/features/desktop/Desktop.tsx:51>), [registro de comandos](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/lib.rs>), [encerramento interno](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/service.rs:138>), [carregamento de saves](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/save.rs>).

## 19. Isolamento e limites da simulação

Os comandos do jogo trabalham nos arquivos e redes virtuais. Não há fallback do terminal para executar comandos do Windows, abrir sockets arbitrários ou executar os binários reais do catálogo. Os relatórios e inspeções consultam o mundo autoral. O aplicativo nativo acessa o disco real para sua própria base de dados e recursos, o que é distinto de expor o disco ao terminal do jogador.

Esse isolamento combina com o escopo do jogo. Porém, autenticação, permissões e firewall representam regras de gameplay. O checksum do save detecta corrupção; não é criptografia nem um mecanismo completo contra edição intencional. A senha fictícia do sistema está armazenada no estado da campanha.

## 20. Plataformas, empacotamento e documentação

A configuração atual é voltada a Windows, com instaladores NSIS/MSI e Tauri/WebView. Há scripts de desenvolvimento, ferramentas Rust locais e artefatos de builds anteriores. A interface web comum não executa o núcleo Rust: precisa do aplicativo Tauri ou dos transportes de cenário/preview existentes.

Os recursos Android/iOS agora estão atualizados, mas src-tauri/gen contém esquemas, sem projetos nativos dessas plataformas. Não foram gerados ou testados APK, AAB ou IPA. A presença dos ícones e do ponto de entrada móvel em Rust não significa que a experiência mobile esteja pronta; janelas, teclado, resolução e ciclo de vida móvel ainda precisam adaptação e validação.

A documentação é extensa, com arquitetura, especificações, narrativa, referências e roteiro de QA. Parte está desatualizada: README ainda descreve toda a sessão 2 como futura, embora duas missões já sejam carregadas; os textos de persistência não explicam todas as regras atuais; docs/QA.md registra resultados de versões antigas.

Há configuração e histórico de distribuição Windows, mas o build atual precisa ser produzido novamente depois de resolver a compilação e as pendências. Não há assinatura de release ou sistema de atualização automática implementado no escopo auditado. A validação em máquina limpa continua necessária para uma distribuição estável.

## 21. Verificações realizadas nesta auditoria

| Verificação                    | Resultado atual                                                                                |
| ------------------------------ | ---------------------------------------------------------------------------------------------- |
| Conteúdo                       | Aprovado: 12 missões, 2 tópicos, 2 hosts iniciais, 303 entradas                                |
| Tipagem TypeScript             | Aprovada                                                                                       |
| ESLint e Stylelint             | Aprovados                                                                                      |
| Formatação Prettier do projeto | Aprovada antes da inclusão deste relatório; os novos documentos foram formatados separadamente |
| Testes Vitest                  | 19 arquivos aprovados, 70 testes aprovados                                                     |
| Testes Rust                    | Não executados: a compilação falhou antes de rodarem                                           |
| Ícones Android/iOS             | 33 PNGs com tamanhos verificados; iOS RGB; prévias inspecionadas                               |
| Aplicativo nativo atualizado   | Não recompilado/ensaiado nesta auditoria                                                       |
| APK/AAB/IPA                    | Não gerados                                                                                    |

A primeira tentativa de Vitest foi impedida pela restrição de leitura do ambiente. A repetição autorizada fora dessa restrição passou. O Rust foi localizado em .tools e executado com o ambiente do projeto, em modo offline.

Erros de compilação observados, ambos E0716, temporary value dropped while borrowed:

- [terminal.rs, linha 1585](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/terminal.rs:1585>): referência obtida de operands(args).first() usada depois que a coleção temporária já foi descartada, no comando man.
- [terminal.rs, linha 1817](<C:/Users/chris/OneDrive/Documentos/Cyber war/src-tauri/src/terminal.rs:1817>): o mesmo padrão no tratamento de killall/pkill.

Os 70 testes de interface aprovados não garantem funcionamento ponta a ponta do núcleo nativo. Não há cobertura automatizada equivalente para player/legendas, visualizador, logout pela GUI, política completa de inicialização e todas as combinações de persistência de missão.

## 22. Ordem de trabalho indicada pelos achados

1. Desbloquear a compilação Rust e executar seus testes. Sem isso, não é possível produzir nem validar a versão nativa atual.
2. Completar a integração de end_session e testar saída pelo painel, Campanha e fechamento da janela.
3. Corrigir a fronteira entre dados pessoais persistentes e progresso temporário de missão; cobrir missão simultânea, save manual, saída forçada e login.
4. Finalizar os pedidos visuais pendentes: entrada seca dos logs de boot e conferência específica dos ícones da versão Kali solicitada.
5. Definir e testar um contrato por comando Linux: sintaxe, flags, saída, erro, permissões, efeitos e códigos de retorno. Priorizar os comandos realmente usados nas missões.
6. Fechar a fidelidade do nano e a semântica do shell dentro de um escopo documentado, eliminando opções que aparentam funcionar sem efeito.
7. Implementar armazenamento binário e completar associações, áudio/vídeo, legendas, imagens, PDF e arquivos compactados conforme os formatos realmente suportados.
8. Aprofundar serviços, processos, perfis de rede e ferramentas específicas antes de aumentar apenas a quantidade de ícones/comandos.
9. Expandir a campanha a partir das duas missões Orion existentes, com objetivos e consequências testáveis.
10. Atualizar documentação, gerar novos instaladores e validar o jogo completo em ambiente limpo. Tratar mobile como uma etapa própria de adaptação e build.

Essas prioridades refletem os problemas encontrados no código atual; não representam mudanças já implementadas nesta auditoria.
