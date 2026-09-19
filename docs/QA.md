# Verificação

## Internet virtual — expansão de 18/09/2026

O estado atual da internet virtual e suas evidências estão em [VIRTUAL-INTERNET-EXPANSION.md](VIRTUAL-INTERNET-EXPANSION.md): 210 marcas, 1.608 documentos, 190 testes React, 139 testes Rust e ensaios explícitos de carga, fuzzing, recuperação, índice e armazenamento. O relatório separa revisão estática de páginas, persistência testada no núcleo e homologação ainda aberta no aplicativo nativo, em sessões prolongadas e por participantes humanos. Os checkpoints abaixo preservam resultados históricos de outros marcos.

## QA 0.4.2 — sites virtuais e fórum

Os sites do navegador virtual foram verificados pela fronteira Rust e a regressão cobre todos os endereços implementados: `wipedia.org`, `archive.org`, `fakebook.com`, `b1.tech`, `pg6mmjiyjmcrsslvykfwnntlaru7p5svn6y2ymmju6nubxndf4pscryd.onion`, seu caminho `/archive`, `mercado.com.br` e `meudominio.com.br`. Os sites tradicionais possuem IP virtual; o Onion Service v3 é mantido separadamente na rede Tor, sem DNS ou IP tradicional, com disponibilidade intermitente. O teste também confirma que o FakeBook permanece bloqueado antes da missão `girl`, libera o conteúdo quando a missão está ativa e que endereços desconhecidos são recusados. Nenhum endereço é aberto no Windows ou na internet real.

O navegador canonicaliza endereços públicos para `https://www.<domínio>`; `www`, ausência de protocolo e `https` não criam registros duplicados. Um `http://` explícito preserva o protocolo inseguro e exibe o alerta de ausência de HTTPS/certificado virtual. O laboratório interno `vex.local` continua reservado ao fluxo de SSH e não é um site público do navegador.

O Terminal Board foi verificado no frontend e no backend: navegação por grupos/categorias/tópico, perfil e busca; criação de tópico; resposta com citação; preservação de rascunho quando a gravação falha; tópico bloqueado somente para leitura; estado offline; filtros por acentos e flags narrativas; conteúdo Markdown/código sem interpretar HTML. Há 11 tópicos de conteúdo validados, com posts de missão liberados somente pelas flags corretas.

Resultado atual do núcleo: 93 testes Rust aprovados e Clippy com `-D warnings` aprovado. A última suíte frontend aprovada teve 163 testes em 28 arquivos, incluindo 6 testes do fórum, 7 do navegador e 5 do Task Manager. Uma repetição posterior ficou impedida pelo limite de uso/permissão do executor do ambiente, não por uma falha de teste; TypeScript, ESLint e Stylelint haviam passado após os últimos ajustes visuais.

## Checkpoint 0.4.2 — consolidação atual

Fechamento dos menus: menu contextual, lançador e popups de terminal/rede/áudio/energia/calendário fecham ao pressionar ou clicar fora. O restante da barra não conta como interior do popup; seu botão de origem permite alternar sem reabrir acidentalmente. Eventos são observados em captura, inclusive quando outra janela interrompe propagação. Cliques dentro de submenus e controle de volume permanecem funcionais. Oito regressões adicionais; suíte frontend com 152 testes em 27 arquivos aprovada. Sem mudança visual ou inspeção nativa nesta correção.

Resultados atuais, baseline e limitações estão em [RELATORIO-CORRECOES-0.4.2.md](RELATORIO-CORRECOES-0.4.2.md). As contagens e inspeções das versões abaixo são históricas e não constituem aprovação deste código.

Regressões acrescentadas: missão única com rejeição atômica da segunda; abandono/saída forçada/manual preservando arquivos pessoais e configurações; conclusão durável; migração de tentativas antigas; logout SQL com falha; três contextos de terminal; nano isolado com conflito/restrições/backups; navegação de logout/login e autostart por IPC simulado; Tab e nova linha do nano; categorias/tópicos/respostas/citações/perfis do fórum.

QA manual pendente: conferir fórum nas larguras pequenas e grandes; duas janelas nano no WebView; teclado e fullscreen; legendas/seek/volumes; reinstalação/upgrade e nova conta de Windows sem ferramentas de desenvolvimento. A inspeção visual atual não foi executada porque a abertura da aba foi recusada pela revisão automática. Nenhuma verificação histórica foi reutilizada como evidência nova.

Desktop: menu contextual testado por componentes (ordem das opções, submenus, teclado, clipboard, erros, criação e janelas independentes). O núcleo testa criação sem sobrescrita/travessia e persistência de itens/ordenação após logout/load. QA visual ainda pendente: conferir bordas/submenus em diferentes resoluções, criação com teclado e colagem entre duas janelas de arquivos; comparar popup claro e destaque roxo com a referência do usuário. Não houve inspeção visual desta implementação.

Terminal, checkpoint de arquivos/navegação: 17 novos testes Rust cobrem os [oito contratos auditados](CONTRATOS-COMANDOS-0.4.2.md), incluindo flags/`--`, CRLF, linha final incompleta, numeração, contagens, headers, UTF-8, limite de saída, sobrescrita/no-clobber, travessia/permissões, contextos local/SSH e redirecionamento. Uma regressão passa pelo `GameService`, salva a saída válida de um comando com erro parcial e verifica o arquivo após encerramento/carregamento. Resultado desta etapa: 70 testes Rust e 130 testes frontend aprovados. Nenhuma mudança visual do terminal nem execução de GNU/Linux real para comparação.

## Gate automatizado

Checkpoint de aprofundamento, posterior aos oito comandos: 88 testes Rust e 144 frontend em 27 arquivos aprovados. Contratos e lacunas em [APROFUNDAMENTO-0.4.2.md](APROFUNDAMENTO-0.4.2.md). Novos testes cobrem consultas/permissões, aspas e expansão do shell sem reinterpretação, status/exportação/source, serviços/processos, integridade/transação/deduplicação binária, antes-imagem binária no diário de missão, busca/substituição/seleção/saída do nano e importação/controles de mídia. Não houve execução de Linux real nem decodificação nativa nos testes de componentes.

Executado no Windows com Node 24, pnpm 11.19.0, Rust 1.98.1 e MSVC. `pnpm check` passou na versão 0.2.0 com 15 testes Rust e 15 testes de interface, além de formatação, lint, tipagem e validação de conteúdo (10 missões, 2 tópicos, 2 hosts e 303 entradas padrão do Kali).

O teste de campanha passa pela mesma fronteira transacional do IPC. Ele percorre First Boot → V1 → V2 → Wi-Fi/pendrive → VEX → Garota; verifica solução antecipada, ordens alternativas, incidente, retorno à decisão, hardening, evidência e save/load final. A corrupção de autosave, falha provocada de SQLite e recuperação manual são cobertas separadamente.

As regressões de 0.1.1 cobrem autocomplete de comandos/caminhos relativos e absolutos, aspas e Unicode, cursor no meio da linha, ambiguidade, permissões e SSH virtual. No frontend, cobrem inserção/remoção no cursor, histórico com rascunho, grafemas, linha longa em terminal estreito e chamadas de entrada/saída de tela cheia com F11.

## Inspeção do aplicativo Windows

O aplicativo Tauri foi aberto e inspecionado por acessibilidade e captura de tela: menu, seleção de cinco slots, desktop, mensageiro e ícones derivados do VFS estavam presentes. O fluxo completo de campanha e persistência foi validado automaticamente no core; os passos abaixo permanecem como roteiro de regressão visual.

Na versão 0.1.1, a captura do desktop confirmou nove ícones em duas colunas e o símbolo `{}` do CodeLab inteiro. A inspeção também confirmou os controles de tela cheia no menu e no painel. A interação manual foi interrompida pelo usuário com Esc antes de concluir os testes de F11 e teclado do terminal no aplicativo; esses comportamentos passaram nos testes automatizados descritos acima.

Na versão 0.1.2, uma prévia isolada com o componente Desktop real e 20 itens de exemplo foi inspecionada no navegador. Foram confirmados 7 ícones por coluna em 1024 × 640, 9 em 1366 × 768 e 13 em 1920 × 1080, com margem lateral de 4 px, zero sobra após a última célula da coluna preenchida e todos os itens entre os painéis. A captura incluiu os 14 SVGs redesenhados. A prévia está em `artifacts/desktop-preview.html` e pode ser aberta pelo servidor Vite; não usa os saves do jogador. A configuração Tauri agora inicia a janela em fullscreen; a abertura nativa dessa versão ainda não foi ensaiada manualmente.

## Revisão 0.2.0 — tema e menu Kali

A prévia isolada confirmou painel de 36 px, menu de 566 px, recursos SVG oficiais
sem imagens ausentes e sem erros de console. Foram exercitados busca e Enter,
favoritos, recentes, abertura de nmap e burpsuite, relatório com dados explícitos
da fixture, maximização com controle Restaurar e minimização. A calculadora
produziu 72 para 9 × 8; bloquear e desbloquear preservou o resultado e a janela.
O calendário exibiu os dias do mês corrente. Em 1024 × 640, 1366 × 768 e
1920 × 1080, o desktop manteve 7, 9 e 13 ícones por coluna, 4 px de margem lateral,
zero sobra após a última célula e nomes sem sobreposição.

O núcleo real foi verificado nos testes Rust: catálogo padrão, favoritos/recentes
serializados, resultados baseados em hosts existentes, recusa de alvos externos,
permissões do VFS, proteção de relatórios existentes e regressão de campanha.
A interface web utiliza uma fixture e não substitui a inspeção nativa da nova
versão nem o ensaio de instalação em máquina limpa.

## Roteiro de smoke test desktop

Na versão 0.1.3, `artifacts/window-preview.html` exercitou o AppWindow real e registrou quadros intermediários para maximizar (620 × 360 → 1280 × 651), minimizar com escala/opacidade, reabrir pela barra e restaurar a geometria original. O botão alternou entre Maximizar e Restaurar. Os testes React também verificam preservação de buffer, restauração rápida, inércia da janela minimizada e duplo clique sem acionamento indevido da barra de título. A prévia do desktop confirmou 4 px abaixo do nome Mensagens, sem sobreposição entre SVG e nomes em uma ou duas linhas.

1. Abrir o executável, passar pela intro e criar uma campanha em um slot vazio.
2. No terminal: `echo "nota de teste" > Documents/notes.txt`.
3. Abrir Arquivos → Documents → notes.txt; conferir conteúdo e editar no HackPad.
4. Salvar e consultar `cat Documents/notes.txt` no terminal.
5. Salvar campanha, fechar normalmente, reabrir e usar Continuar; conferir conteúdo novamente.
6. Criar um arquivo em `Desktop/`; conferir o novo ícone. Mover pela GUI e verificar `ls`.
7. Alterar um arquivo pelo terminal enquanto estiver aberto no editor; confirmar que salvar um buffer antigo retorna conflito e preserva o arquivo recente.
8. Minimizar/maximizar, redimensionar, trocar workspace e voltar ao app.
9. Em Campanha, restaurar o checkpoint inicial da missão e conferir o retorno do estado.
10. Preencher o desktop além de uma coluna, redimensionar a janela e verificar se todos os ícones continuam acessíveis; conferir o símbolo do CodeLab.
11. Pressionar F11 no menu e no terminal com foco: ocupar toda a tela e retornar ao tamanho anterior. Repetir pelo botão em Configurações → Exibição.
12. No terminal, digitar `cd ~/Doc` e Tab: obter `cd ~/Documents/`. Digitar `echo ac`, pressionar esquerda e inserir `b`: Enter deve imprimir `abc`. Conferir também direita, Backspace, Delete e histórico.

## Artefatos de release

`pnpm build` concluiu para 0.2.0 e gerou NSIS (5.649.000 bytes), MSI (6.242.304 bytes) e executável (8.612.864 bytes). A compilação habilitou `custom-protocol`, incorporando os assets de produção ao binário. Os hashes da versão atual estão em `artifacts/SHA256SUMS.txt`. O release 0.1.0 havia sido aberto separadamente e exibido o menu e os slots existentes; a inspeção visual das correções de 0.1.1 usou o aplicativo de desenvolvimento, e as de 0.1.2/0.1.3/0.2.0 usaram as prévias isoladas descritas acima.

## Distribuição

Os builds NSIS e MSI devem ser verificados também em uma máquina Windows limpa, sem Node/Rust, com instalação, execução offline, atualização e desinstalação. Esta sessão utiliza a máquina de desenvolvimento; não substitui esse ensaio em máquina limpa. Windows Sandbox não está instalado nesta máquina e nenhuma VM limpa foi disponibilizada.

O diretório de saves é externo à instalação e não é removido pelo código do jogo. O script NSIS gerado foi inspecionado: a remoção do diretório de dados só ocorre com a opção explícita de apagar dados marcada, e não ocorre no modo de atualização. Uma atualização deve ser ensaiada com uma cópia de saves anteriores antes de distribuição ampla. A opção padrão do instalador Tauri pode instalar WebView2 se ausente; esse primeiro setup pode exigir internet, embora o jogo não utilize rede real.

Assinatura Authenticode não foi aplicada: exige certificado e credenciais do responsável pela distribuição, mantidos fora do repositório. Não há telemetria, auto-update nem publicação remota nesta entrega.
