# CYBER WAR — Opening Cinematic

> **Direção vigente — revisão 4:** cada tomada aparece uma única vez; não repetir BLACKWIRE, notícias ou a mesma conversa com VEX. Variar conteúdo e disposição dos terminais. Incluir composições de duas e três ações simultâneas, painéis em movimento, mais transições e efeitos breves nos cortes. Substituir a trilha tranquila por música de tensão crescente, com graves e percussão marcados. Preservar a gravação aprovada do final, acrescentando somente o cursor piscando ao lado do título. Manter login `root`, cenas sem deformação, duração de 68,1 segundos e inicialização Linux com comando no rodapé. As seções anteriores permanecem como histórico.

## 1. Objetivo

Esta sequência é a **cinemática de abertura do jogo**, exibida depois da intro da **Studio Bechi Games** e antes da tela **Clique para iniciar**.

Ela **não é um trailer promocional**. O jogador já abriu o jogo. A função da sequência é introduzir o clima, o universo e a escalada da história através do próprio computador do protagonista.

Fluxo:

```text
STUDIO BECHI GAMES
        ↓
OPENING CINEMATIC
        ↓
CYBER WAR_
        ↓
CLIQUE PARA INICIAR
        ↓
MENU PRINCIPAL
```

## 2. Regras de direção

- Duração máxima absoluta: **1 minuto e 30 segundos**.
- Duração alvo: **aproximadamente 1:25–1:30**.
- Nenhuma cena principal deve durar menos de **3 segundos**.
- Preferência por planos de **4–6 segundos**.
- A aceleração deve acontecer **dentro das telas**, e não por cortes frenéticos.
- Não usar narrador.
- Não usar slogans promocionais.
- Não apresentar mecânicas como uma lista de features.
- As missões aparecem como fragmentos/presságios da história que o jogador ainda viverá.
- Não explicar quem são VEX, NULL ou as guildas.
- O título **CYBER WAR** nasce visualmente do próprio terminal.

---

# 3. Fidelidade absoluta ao jogo

A abertura deve usar o **mesmo visual existente no gameplay**.

Reutilizar:

- desktop do Kali simulado;
- wallpaper atual;
- painel superior;
- WindowManager;
- bordas e barras das janelas;
- terminal real;
- prompt real;
- cores ANSI;
- cursor;
- File Manager;
- navegador;
- fórum;
- mensageiro;
- notificações;
- ícones;
- ferramentas;
- sons de interface.

Não criar uma versão cinematográfica separada das aplicações.

## Proibido

- HUD futurista inexistente;
- hologramas;
- terminal genérico;
- janelas sci-fi;
- fontes exclusivas do vídeo;
- interfaces falsas só para parecer hacker;
- chuva Matrix durante toda a abertura;
- nomes de técnicas sobrepostos à imagem;
- cards apresentando missões ou guildas.

Se uma tela ainda não existir no jogo, seu visual definitivo deve ser criado antes de utilizá-la na abertura.

---

# 4. Identidade visual

A abertura começa usando exatamente a paleta normal do jogo.

Referência inicial:

```text
Terminal background: #111315
Terminal foreground: #e7e7e7
Window title bar:    #24272a
```

Prompt:

```text
┌──(kali㉿game-hacker)-[/home/kali]
└─$
```

O **verde só ganha protagonismo no clímax final**, quando a saída do terminal começa a dominar a tela.

---

# 5. Roteiro

## 00:00–00:06 — O computador

Tela preta.

Som ambiente eletrônico muito baixo.

Fade para o **desktop real do jogo**.

O painel superior, wallpaper, atalhos e demais elementos são exatamente os usados durante a campanha.

O cursor se move.

O jogador abre o Terminal.

A janela aparece.

```text
┌──(kali㉿game-hacker)-[/home/kali]
└─$
```

O cursor pisca.

A trilha começa com um pulso grave discreto.

---

## 00:06–00:12 — Mais dois terminais

O jogador abre um segundo terminal.

Reposiciona rapidamente a janela.

Abre um terceiro.

Ao final da cena existem **três terminais reais do jogo visíveis simultaneamente**.

Não cortar individualmente para cada terminal.

A própria abertura e movimentação das janelas conduz a atenção.

---

## 00:12–00:18 — A máquina trabalhando

Os três terminais começam a trabalhar simultaneamente.

Um apresenta reconhecimento de uma rede virtual.

Outro percorre arquivos e diretórios.

Outro apresenta logs/conexões do mundo simulado.

Fragmentos podem aparecer:

```text
host discovered
service detected
session active
```

e:

```text
/home/kali/projects
/home/kali/Downloads
/home/kali/tools
```

A quantidade de informação aumenta, mas o espectador ainda consegue acompanhar.

---

## 00:18–00:24 — File Manager

Corte para o **File Manager real**.

O jogador entra em:

```text
/home/kali/Downloads
```

Abre uma pasta.

Seleciona um arquivo.

Volta.

Entra em `projects`.

Abre outro arquivo.

Toda a ação acontece no mesmo plano.

O objetivo é mostrar naturalmente que o computador possui um filesystem vivo e que terminal e interface gráfica fazem parte do mesmo sistema.

---

## 00:24–00:29 — VEX

Voltamos ao desktop.

Uma notificação real do mensageiro aparece:

```text
VEX
Tenho um trabalho pra você.
```

O jogador abre o mensageiro.

Algumas mensagens aparecem.

Não explicar quem é VEX.

Antes que a conversa revele demais:

**corte.**

---

## 00:29–00:34 — Fórum / NULL

O fórum hacker real aparece.

O jogador rola um tópico.

Mensagens, nicknames e respostas passam pela tela.

A rolagem diminui.

Uma resposta fica visível:

```text
NULL:
script kiddie.
```

Pequena pausa.

Corte para o desktop.

---

## 00:34–00:40 — SIGNAL / NO AR

A ferramenta wireless real do jogo aparece.

Redes próximas são exibidas.

Uma rede corporativa chama atenção.

O jogador interage com ela.

Durante o mesmo plano, a ação progride até alguma evidência do ambiente da empresa.

O objetivo narrativo é sugerir que existe algo escondido ali — não explicar a técnica.

---

## 00:40–00:46 — EM CLARO

A interface real utilizada para observar as comunicações aparece.

Durante os seis segundos, os fragmentos surgem progressivamente:

```text
Conselho aprovou.
```

Depois:

```text
Não menciona Orion aqui.
```

Depois:

```text
Assinatura sexta.
```

E finalmente:

```text
US$ 2.4B
```

A trilha reage ao último fragmento.

Corte.

---

## 00:46–00:52 — Outra operação

Mostrar outra missão já definida utilizando uma técnica diferente.

A cena deve possuir começo, ação e resultado parcial dentro dos mesmos seis segundos.

Pode envolver, conforme estiver implementado:

- IoT;
- acesso a host virtual;
- análise de executável;
- engenharia reversa;
- arquivos;
- dispositivo;
- browser;
- permissões.

Não colocar o nome da técnica na tela.

O jogador simplesmente a utiliza.

---

## 00:52–00:58 — Escalada

Voltamos ao desktop.

Agora existem várias aplicações abertas.

Os três terminais continuam trabalhando.

O File Manager permanece aberto.

O navegador está atrás.

Uma mensagem chega.

Uma notificação aparece.

Uma janela recebe foco.

Outra continua produzindo saída.

A sensação de velocidade nasce das **ações simultâneas dentro da mesma tela**.

---

## 00:58–01:03 — Te peguei

O mensageiro recebe foco.

Nova mensagem.

```text
NULL
Te peguei.
```

A música praticamente desaparece.

O cursor para.

A mensagem fica na tela.

Pequena pausa desconfortável.

Então:

**corte seco.**

---

## 01:03–01:09 — O computador sai do controle

Voltamos ao desktop.

Agora a quantidade de atividade aumenta muito.

Os três terminais produzem saída.

Mensagens chegam.

Uma notificação aparece.

O navegador muda.

Um arquivo termina de ser processado.

Uma janela abre.

Outra é movida.

Tudo acontece durante **um único plano de seis segundos**.

A trilha volta com força.

---

## 01:09–01:14 — Terminal assume a tela

O jogador seleciona um dos terminais.

A janela vem para frente.

É maximizada usando a animação real do WindowManager.

O restante do computador desaparece atrás dela.

Então o terminal entra em fullscreen.

A trilha reduz.

Agora só existe:

```text
terminal
cursor
prompt
```

---

## 01:14–01:20 — Operação final

Prompt real:

```text
┌──(kali㉿game-hacker)-[/home/kali]
└─$
```

Uma operação fictícia pertencente ao mundo do jogo é iniciada.

Não é necessário mostrar um comando ofensivo real.

Enter.

A saída começa devagar:

```text
node discovered
route established
session opened
packet received
access granted
```

Depois surgem elementos técnicos:

```text
0x00AF21
01001001
```

A velocidade aumenta progressivamente.

---

## 01:20–01:25 — O terminal é tomado

As linhas começam a aparecer mais rapidamente.

O scroll acelera.

Mais caracteres.

Mais dados.

A saída passa a preencher praticamente todo o terminal.

O verde começa a dominar a composição.

A transformação ainda parece consequência natural do terminal.

---

## 01:25–01:30 — CYBER WAR_

Os caracteres verdes começam a descer pela tela.

Alguns param.

Outros continuam.

Novos grupos congelam em posições específicas.

Em poucos segundos, os caracteres estacionários começam a formar letras.

O restante desaparece.

Fundo preto absoluto.

No centro:

```text
CYBER WAR
```

Grande.

Verde.

Então surge o cursor:

```text
CYBER WAR_
```

O cursor pisca.

A música desaparece.

**Fim da Opening Cinematic.**

> Durante a edição final, priorizar terminar a movimentação aproximadamente em `01:25` e preservar o máximo possível dos últimos segundos para a formação do título e o cursor. A duração total nunca pode ultrapassar `01:30`.

---

# 6. Tela seguinte

A Opening Cinematic termina.

Depois de uma breve transição preta, o jogo retorna ao fluxo interativo em React:

```text
CYBER WAR

CLIQUE PARA INICIAR
```

Essa tela **não faz parte do vídeo/cinemática**.

---

# 7. Ritmo

Não utilizar a lógica tradicional de trailer:

```text
corte
corte
corte
corte
```

O ritmo deve vir de:

- digitação;
- scroll;
- janelas abrindo;
- janelas se movendo;
- mensagens;
- notificações;
- múltiplos terminais;
- arquivos;
- processos;
- alterações dentro das aplicações;
- aumento da velocidade da saída.

Regra:

> **3 segundos é o piso absoluto para uma cena principal.**

Preferência:

```text
4–6 segundos
```

Momentos de tensão podem permanecer mais tempo.

---

# 8. Missões como presságios

As missões mostradas não existem para apresentar conteúdo ao comprador.

Elas funcionam como **flashes do futuro da história**.

O jogador que assiste pela primeira vez pode não entender:

```text
Quem é VEX?
Quem é NULL?
O que é Orion?
Por que US$ 2.4B?
O que aconteceu naquela rede?
```

Isso é desejável.

Quando essas situações acontecerem posteriormente durante a campanha, o jogador poderá reconhecer imagens da abertura.

---

# 9. Guildas

Não é obrigatório mostrar todas as guildas.

LEAK, BREACH, SIGNAL, GHOSTMARKET e ROOT só devem aparecer quando fizerem sentido dentro das telas utilizadas.

Não interromper a sequência para apresentar:

```text
LEAK
BREACH
SIGNAL
GHOSTMARKET
ROOT
```

como cards.

A Opening Cinematic não é um catálogo do jogo.

---

# 10. Áudio

A música deve seguir aproximadamente:

```text
ambiente
→ pulso
→ ritmo
→ tensão
→ escalada
→ caos
→ queda
→ clímax
→ silêncio
```

Reutilizar os sons reais da interface:

- mouse;
- teclado;
- terminal;
- notificações;
- mensagens;
- janelas;
- downloads;
- alertas.

SFX cinematográficos podem reforçar transições e impactos, mas nunca substituir a identidade sonora real do jogo.

---

# 11. Sem narrador e sem slogans

Não usar:

```text
EVERY SYSTEM HAS A WEAKNESS
ARE YOU READY?
THE WORLD IS CONNECTED
```

Não precisamos vender o jogo.

As próprias situações contam a abertura:

```text
VEX:
Tenho um trabalho pra você.
```

```text
NULL:
script kiddie.
```

```text
US$ 2.4B
```

```text
NULL:
Te peguei.
```

```text
uid=0(root)
```

---

# 12. OpeningScenarioMode

Criar um modo interno de desenvolvimento:

```text
OpeningScenarioMode
```

Não disponível ao jogador.

Possíveis estados:

```text
opening/desktop
opening/three-terminals
opening/files
opening/vex
opening/null-forum
opening/no-ar
opening/em-claro
opening/operation
opening/escalation
opening/null-got-you
opening/chaos
opening/final-terminal
opening/title
```

Cada estado utiliza o **mesmo WorldState e os mesmos componentes do gameplay**.

---

# 13. Timeline determinística

As ações devem poder ser reproduzidas exatamente.

Exemplo:

```text
T+0000  fade desktop
T+1800  cursor move
T+2600  open terminal
T+6000  open terminal 2
T+8500  reposition
T+9600  open terminal 3
T+12000 start simultaneous activity
...
```

Isso permite recapturar a Opening Cinematic quando a interface do jogo evoluir.

---

# 14. Componentes

Estrutura recomendada:

```text
src/features/
├── desktop/
├── terminal/
├── file-manager/
├── messenger/
├── browser/
├── forum/
├── notifications/
├── window-manager/
│
└── opening/
    ├── OpeningCinematic.tsx
    ├── OpeningDirector.ts
    ├── OpeningTimeline.ts
    ├── scenarios/
    └── title/
        └── CyberWarReveal.tsx
```

A pasta `opening` apenas orquestra.

Não duplicar:

```text
TerminalWindow
FileManager
Messenger
Browser
Desktop
Window
Notification
```

---

# 15. Formação do título

A transformação final:

```text
terminal real
→ saída acelerando
→ caracteres verdes
→ caracteres descendo
→ caracteres parando
→ CYBER WAR_
```

pode utilizar React/Canvas para controle preciso.

Entretanto deve copiar exatamente:

- fonte do terminal;
- verde ANSI;
- background;
- cursor;
- espaçamento;
- densidade;
- tamanho visual.

A transição deve ser imperceptível.

O espectador não deve perceber onde termina o terminal normal e começa o renderer especial.

---

# 16. Checklist

- [x] Opening Cinematic, não trailer promocional.
- [x] Duração total <= 01:30.
- [x] Nenhuma cena principal < 3 segundos.
- [x] Maioria dos planos entre 4 e 6 segundos.
- [x] Desktop igual ao gameplay.
- [x] Wallpaper igual.
- [x] Painel igual.
- [x] WindowManager real.
- [x] Terminal real.
- [x] Prompt real.
- [x] Fonte real.
- [x] File Manager real.
- [x] Mensageiro real.
- [x] Fórum real.
- [x] Navegador real.
- [x] Notificações reais.
- [x] Sons de UI reais.
- [x] Sem HUD promocional.
- [x] Sem interfaces sci-fi.
- [x] Sem apresentação de features.
- [x] Sem narrador.
- [x] Sem slogans.
- [x] Missões funcionam como presságios.
- [x] A velocidade vem das ações, não de cortes de 1 segundo.
- [x] Verde domina apenas no clímax.
- [x] CYBER WAR nasce do terminal.
- [x] Cursor final pisca.
- [x] Depois entra Clique para iniciar.

---

# 17. Sensação final

A abertura deve parecer que o jogador observou, por cerca de um minuto e meio, **o computador do protagonista antecipando fragmentos do caos que está por vir**.

No início existe apenas um terminal.

Depois três.

Depois arquivos, pessoas, redes, mensagens e operações começam a ocupar aquele computador.

Até que tudo converge novamente para uma única coisa:

```text
CYBER WAR_
```

O cursor pisca.

E agora é a vez do jogador assumir o computador.

---

# 18. Histórico da primeira execução — 13/09/2026

Implementado a partir deste arquivo, o único documento presente na pasta `specs` nesta execução. O fluxo é Studio Bechi Games → Opening Cinematic → transição preta → CYBER WAR / Clique para iniciar → menu principal. A tela interativa permanece fora do vídeo. A preferência persistida `skipTrailer` foi mantida por compatibilidade e agora aparece como “Pular cinemática ao abrir”.

A abertura final foi capturada em 1440 × 900 com os componentes reais do gameplay. Os arquivos locais `public/assets/video/cyber-war-opening.mp4` e `public/assets/video/cyber-war-opening.webm` têm exatamente 90,000 segundos e 30 quadros por segundo. Todos os 16 planos duram 5 ou 6 segundos. O teaser promocional anterior foi substituído.

`OpeningScenarioMode` é carregado apenas em desenvolvimento, em `/?capture=1#opening/desktop`. A linha do tempo utiliza um snapshot validado e exportado pelo núcleo Rust, incluindo os resultados reais dos comandos e as páginas do navegador. O transporte de desenvolvimento fica isolado dos comandos nativos e restaura os stores ao desmontar; não grava campanhas. Os aplicativos são os mesmos do desktop. O renderer final lê o buffer, a fonte, a cor ANSI e a geometria das células do xterm.

As ferramentas de redes sem fio e de leitura de capturas foram integradas ao gameplay. As missões NO AR e EM CLARO podem ser iniciadas após a sessão 1 e preservam a evidência no filesystem virtual. Também estão disponíveis múltiplas janelas do terminal pelo lançador, atalho do desktop e Ctrl+Alt+T, além do controle de fullscreen do terminal com saída por Esc.

A trilha original acompanha a escalada e mantém os efeitos da interface em um canal separado, sujeito a Master/SFX. O vídeo usa Master/Music. A música mede aproximadamente −52 dB no trecho de NULL e −18,4 dB na volta do caos, chegando a silêncio no final. A prévia com música e efeitos misturados está em `artifacts/opening/cyber-war-opening-preview.mp4`.

Verificações concluídas:

- `pnpm check`: formatação, ESLint, Stylelint, TypeScript, validação do conteúdo, 40 testes de frontend, Rustfmt, Clippy e 19 testes Rust aprovados.
- Conteúdo: 12 missões, 2 tópicos, 2 hosts e 303 entradas do catálogo padrão Kali, sem pacotes opcionais adicionados.
- Captura: 411 ações executadas, 1.794 quadros capturados, 214 eventos sonoros e nenhum erro de aplicação, recurso ou ação. Os quadros de inspeção das cenas estão em `artifacts/opening/scene-*.png`.
- `artifacts/opening/manifest.json` registra cenas, eventos, áudio e erros; `media-verification.json` registra codecs, duração e SHA-256 dos arquivos finais.
- Testes cobrem a progressão e persistência das investigações, limite de 90 segundos, ordem da linha do tempo, múltiplos terminais, fullscreen/restauração, canais de áudio separados, encerramento único, fallback de vídeo e pausa em segundo plano.

Para recapturar após mudanças no gameplay, com Rust, Node, Playwright, Chrome e FFmpeg disponíveis:

```powershell
$env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
$env:RUSTUP_HOME = Join-Path (Get-Location) '.tools/rustup'
$env:CARGO_INCREMENTAL = '0'
$env:CYBER_WAR_EXPORT_OPENING = Join-Path (Get-Location) 'src/features/opening/scenarios/gameplay-world.json'
& .tools/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml opening_snapshot_uses_valid_gameplay_world_and_real_command_results
Remove-Item Env:CYBER_WAR_EXPORT_OPENING
node node_modules/prettier/bin/prettier.cjs --write src/features/opening/scenarios/gameplay-world.json
node scripts/generate-opening-audio.mjs
# Manter pnpm dev:web em outro terminal, na porta 1420.
node scripts/capture-opening.mjs
node scripts/finish-opening.mjs
```

O script aceita `PLAYWRIGHT_MODULE` e `CHROME_PATH` para instalações em outros caminhos. O snapshot é regenerado a partir do jogo; os scripts não precisam de gravação manual de janelas ou edição do vídeo.

**Pendência de distribuição:** o código está versionado como 0.4.0, mas o novo executável e os instaladores Windows ainda não foram gerados. A revisão automática bloqueou a chamada de `scripts/dev.ps1 build` porque o limite de uso da conta foi atingido. Após a liberação, executar esse build e validar a reprodução no aplicativo empacotado. Os testes e a captura acima não substituem essa verificação final no instalador.

---

# 19. Revisão 2 — nova direção do usuário, 13/09/2026

A versão vigente substitui a montagem de 90 segundos descrita na seção 18. Tem **67,7 segundos a 30 fps**, com 42 planos, 22 inserts abaixo de um segundo, 28 cortes secos e 14 transições curtas. Sete composições mostram ações simultâneas: três com duas cenas e quatro com três cenas. As divisões aproveitam a tela inteira, com separadores estreitos. O material continua vindo dos aplicativos reais do gameplay, reorganizado em saltos entre operações e enquadramentos de detalhe.

O início mostra o login fictício `kali@lifeos.local`, com digitação de e-mail e senha mascarada. No final, o terminal recebe e executa o comando antes de a chuva ocupar toda a largura. Os fluxos misturam seis famílias de fragmentos de programas, dados hexadecimais, binários, assinaturas e cargas codificadas. São elementos visuais locais, sem execução de criptografia ou conexão externa.

Os próprios caracteres da chuva se fixam para formar **CYBER WAR**. Cada letra usa uma matriz explícita; o E tem três barras bem definidas e espaços abertos entre elas. O título permanece composto exclusivamente por letras e números pequenos. O renderer anterior que convertia o resultado em texto sólido foi removido.

A trilha original foi refeita em estéreo a 150 BPM, com bateria, baixo, acordes, arpejos, viradas e crescimento por etapas. Há uma queda breve em NULL, redução durante a digitação final e retomada no clímax da chuva. A música e os 224 eventos de interface continuam em canais independentes no jogo. A prévia mistura os dois canais para reprodução direta.

Arquivos e verificação:

- `artifacts/opening/cyber-war-opening-preview.mp4`: montagem final com música e efeitos.
- `public/assets/video/cyber-war-opening.{webm,mp4}`: arquivos de reprodução do jogo, ambos com 67,7 segundos; VP9/Opus e H.264/AAC.
- `public/assets/audio/cyber-war-opening-sfx.ogg`: efeitos sincronizados, também com 67,7 segundos.
- `artifacts/opening/edit-verification.json`: duração e contagem dos cortes, inserts e composições.
- `artifacts/opening/media-verification.json`: codecs, duração e SHA-256 dos arquivos integrados.
- `artifacts/opening/bookends/manifest.json`: 62 eventos, 909 quadros capturados e nenhum erro de aplicação ou ação no login/final revisado.
- `pnpm check` aprovado: formatação, ESLint, Stylelint, TypeScript, conteúdo, **43 testes de frontend e 19 testes Rust**, Rustfmt e Clippy. Os novos testes verificam a formação do título, o contorno do E e a variedade dos blocos de código.

Para refazer a montagem mantendo a captura de gameplay já validada em `artifacts/opening/source-v1.mp4` e `source-v1-manifest.json`, manter o servidor de desenvolvimento na porta 1420 e executar:

```powershell
node scripts/capture-opening.mjs --bookends
node scripts/edit-opening.mjs
node scripts/finish-opening.mjs
```

Para atualizar também as cenas do gameplay, primeiro regenerar o snapshot Rust conforme a seção 18, executar `node scripts/capture-opening.mjs` e copiar os novos `gameplay-silent.mp4` e `manifest.json` de `artifacts/opening` para `source-v1.mp4` e `source-v1-manifest.json`, respectivamente. Depois executar os três comandos acima. `OpeningBookends` e o modo de captura são carregados apenas em desenvolvimento e não integram o JavaScript de produção.

O WebM usa faixa de vídeo limitada e matriz BT.709. A exportação anterior em faixa completa apresentava erro de decodificação no Chromium/Windows, apesar de ser aceita pelo FFmpeg; o teste de reprodução real identificou essa diferença. A conversão preserva a imagem e mantém o MP4 como alternativa. O arquivo de teste temporário de codec é removido antes do empacotamento.

**Estado da distribuição após a revisão 2:** o build de produção e os instaladores Windows 0.4.0 (NSIS e MSI) foram gerados com sucesso. Esses pacotes incluem a nova montagem, mas antecedem a correção final de compatibilidade do WebM. A prévia de produção reproduziu corretamente pelo MP4, com som, efeitos sincronizados, transição para Clique para iniciar e cinco ações no menu; a rota de captura ficou ausente. O teste nativo iniciou a reprodução, mas não concluiu a verificação por salto de tempo. Não se considera a reprodução nativa integral aprovada.

A amostra de quatro segundos do WebM corrigido passou no Chrome. O arquivo completo de 67,7 segundos foi então reexportado, mas sua nova checagem no Chrome foi bloqueada pela revisão automática por limite de uso da conta, com horário de nova tentativa informado como 9h38. A chamada seguinte de build não chegou a executar. Permanecem pendentes a validação do WebM completo, o novo empacotamento, a atualização de `artifacts/SHA256SUMS.txt` e a reprodução nativa sem saltos até o menu. O estado detalhado está em `artifacts/opening/execution-status.json`; a prévia MP4 final já está disponível.

**Atualização posterior — revisão do painel e terminal:** a validação do WebM completo no Chrome passou, com áudio, sincronização, quadro da chuva, encerramento e menu, sem erro de decodificação ou fallback. Os pacotes Windows 0.4.0 foram recompilados com esse WebM e os ajustes solicitados na logo do painel, no visual anterior do terminal e na escala dos controles de janela. Os hashes foram atualizados. As verificações de formatação, lint, TypeScript, conteúdo, 43 testes de frontend, Rustfmt, Clippy e 19 testes Rust passaram. As medidas e a conferência visual do painel e terminal estão em `artifacts/terminal-review-verification.json`.

O executável iniciou a reprodução do WebM corrigido com som e efeitos sincronizados em fullscreen (1366 × 768). A janela encerrou antes do fim da verificação integral, inclusive na repetição com o processo pai mantido ativo; a causa não foi estabelecida. Apenas a conferência nativa sem interrupções até o menu permanece pendente, sem bloqueio atual de revisão automática.

---

# 20. Revisão 3 — cenas completas e novo acabamento

A montagem atual substitui os enquadramentos da revisão 2. Todas as fontes foram recapturadas em **1600 × 900, 16:9**, usando o desktop e os aplicativos reais. Não há filtros de recorte, ampliação, deformação ou preenchimento preto na edição. São **32 planos em 68,1 segundos**, com 12 inserts abaixo de um segundo e 8 transições de quatro quadros. Os demais encontros de planos usam cortes secos.

O login mostra `root`. O trailer alterna fórum, conversa com VEX, mensagens de NULL, notícias B1, arquivos do diretório BLACKWIRE, investigação de rede sem fio, leitura de tráfego, arquivos recebidos e comandos de terminal. Um comando abre outros dois terminais; as três janelas executam saídas simultâneas. BLACKWIRE e suas páginas são conteúdos locais do navegador simulado, disponíveis também no gameplay, sem acesso externo.

O final preserva a execução digitada antes da chuva. A chuva ocupa toda a tela com caracteres pequenos e densos, removendo os blocos horizontais organizados em colunas. Esses mesmos caracteres formam CYBER WAR em traços de espessura uniforme, com diagonais e espaços internos definidos. Não há uma camada de texto sólido sobreposta ao título.

A tela anterior ao trailer usa linhas horizontais de inicialização Linux, reveladas sequencialmente com movimento da direita para a esquerda. No rodapé, o prompt `root` digita `./cyber-war --start`, confirma a execução e avança para o trailer. A trilha estéreo original a 150 BPM acompanha a nova duração; efeitos de interface e transições são sincronizados em canal separado.

Para reproduzir esta versão, regenerar o snapshot Rust conforme a seção 18 e manter o servidor de desenvolvimento na porta 1420. Os comandos vigentes são:

```powershell
node scripts/capture-opening.mjs --bookends
node scripts/capture-opening.mjs --montage
node scripts/edit-opening.mjs
node scripts/finish-opening.mjs
```

O editor verifica a dimensão e o número de quadros de cada plano antes de montar o arquivo final. A publicação recusa uma revisão ou duração divergente. As capturas e os relatórios atuais ficam em `artifacts/opening/bookends`, `artifacts/opening/montage`, `edit-verification.json`, `media-verification.json` e `execution-status.json`. A cópia anterior para comparação está em `revision-2-preview.mp4`. O estado final de testes, reprodução e distribuição é registrado em `execution-status.json`.

**Verificação final da revisão 3:** `pnpm check` aprovado, incluindo 42 testes de frontend, 19 testes Rust, formatação, lint, TypeScript, conteúdo e Clippy. As 14 cenas do gameplay e o login/final foram capturados sem erros de ações, aplicação ou recursos. O WebM de 68,1 segundos foi reproduzido integralmente, sem saltos, no Chrome de produção e no executável Windows em fullscreen de 1366 × 768, com música, efeitos sincronizados e evento de encerramento confirmado. Ambos chegaram a Clique para iniciar e às cinco ações do menu. A pendência de reprodução nativa das revisões anteriores está resolvida nesta versão. EXE, NSIS e MSI 0.4.0 foram gerados com a revisão 3, e `artifacts/SHA256SUMS.txt` foi atualizado.


---

# 21. Revisão 4 — variedade, composições e tensão

A montagem contém 20 tomadas de gameplay distintas, cada uma usada uma vez, além do login e do final. BLACKWIRE, B1, fórum e a conversa com VEX têm uma única aparição. As novas ações incluem o CodeLab, processos, comparação de hashes, consulta de rotas, codificação de uma sessão e proteção de arquivos. Os resultados dos comandos são exportados do núcleo Rust.

As composições simultâneas usam duas ou três aplicações reais, com dimensões próprias e conteúdo adaptado ao espaço: CodeLab com terminal, processos com terminal e documento, rotas com biblioteca, arquivos com código e outras combinações. Há terminais lado a lado, sobrepostos e em arranjo horizontal, com reorganização animada. Os painéis ocupam o desktop, preservando proporções e legibilidade. A edição adiciona transições de seis quadros, flashes breves e separação sutil de canais de cor em cinco impactos.

A trilha foi substituída por uma composição original de tensão a 166 BPM: baixo distorcido, ostinato em intervalos tensos, percussão metálica, bateria mais forte e impactos sincronizados. A masterização usa folga para os transientes e exporta AAC e Opus diretamente do master PCM, evitando compressões sucessivas. Os efeitos de interface permanecem separados no jogo. A prévia final mede −19,14 LUFS integrados e pico verdadeiro de −5,46 dBTP; todos os arquivos distribuídos passaram na checagem de picos, sem saturação.

O final aprovado está preservado em `artifacts/opening/approved-v3-final.mp4`. A edição usa esse arquivo como fonte, sem recapturar a chuva nem reconstruir as letras; a única adição visual é um cursor verde em forma de sublinhado, piscando ao lado do título depois da formação. A versão anterior completa está em `revision-3-preview.mp4`.

Para reproduzir: exportar o snapshot Rust, executar `node scripts/capture-opening.mjs --montage`, `node scripts/edit-opening.mjs` e `node scripts/finish-opening.mjs`. Manter os arquivos aprovados da revisão 3 e a captura do login. Não recapturar os bookends para esta revisão. O relatório final está em `artifacts/opening/execution-status.json`.


**Verificação final da revisão 4:** 43 testes de frontend e 19 testes Rust aprovados, além de formatação, ESLint, Stylelint, TypeScript, conteúdo, Rustfmt e Clippy. A captura terminou com 1.274 quadros e todas as ações executadas; as 20 tomadas foram conferidas sem erros. O cursor foi verificado nos estados ligado e apagado. A comparação com a gravação final aprovada, fora da região do cursor, apresentou SSIM 0,994482 após compressão. A trilha e os efeitos passaram na checagem de picos dos formatos entregues. O Chrome e o executável Windows reproduziram a sequência completa, sem saltos, com som e efeitos sincronizados, encerramento, tela de início e cinco ações no menu. O teste nativo confirmou fullscreen de 1366 × 768. Os instaladores NSIS e MSI 0.4.0 incluem a versão final com a mixagem corrigida; os hashes foram atualizados.

---

# 22. Revisão 5 — inicialização em duas telas e acompanhamento automático

A tela de inicialização da Studio Bechi mantém os mesmos efeitos, a duração de sete segundos, a logo e o comando `./cyber-war --start`, mas agora apresenta um log Linux/Kali com duas telas de altura. As mensagens alternam linhas curtas de status e registros longos do kernel, rede, dispositivos e serviços para evitar um bloco uniforme.

O log ocupa o viewport sem criar barra de rolagem. As novas linhas entram em ordem e o conteúdo acompanha automaticamente a segunda tela com uma translação contínua; o jogador não precisa rolar a página. A segunda tela deixa 104 px livres no rodapé, e o prompt `root` permanece separado da última linha antes de digitar o comando e avançar ao trailer. O layout mede a altura, largura e métrica da fonte do viewport e recalcula as duas páginas quando a janela muda de resolução.

Em `prefers-reduced-motion`, as linhas continuam visíveis e o acompanhamento troca para um salto no fim da sequência, preservando a leitura e a margem inferior sem uma animação prolongada. O trailer aprovado, seus arquivos de vídeo, trilha de tensão, efeitos e cursor final não foram recapturados nem alterados nesta revisão.

**Estado da distribuição 0.4.1:** código, Cargo e documentação estão versionados como 0.4.1. A checagem local passou com 43 testes de frontend, 19 testes Rust, Prettier, ESLint, Stylelint, TypeScript, conteúdo, Rustfmt e Clippy. A validação visual automatizada está em `artifacts/boot-0.4.1/verify.cjs`; a execução em navegador com escalonamento externo ficou bloqueada pelo limite de uso da conta, portanto a release registra essa limitação até o build local dos instaladores ser concluído.

---

# 23. Configuração inicial do sistema e autenticação de campanha

Ao iniciar uma campanha nova, o menu abre um instalador visual do LifeOS inspirado na instalação do Kali. O fluxo tem quatro etapas: idioma fixo em português do Brasil, layout do teclado, modo de tela e conta local. O teclado oferece ABNT2, ABNT e EUA internacional; a tela permite fullscreen ou janela com resolução inicial; a conta define o nome do computador, usuário e senha com validação antes da instalação.

As escolhas de idioma, teclado, modo, resolução, usuário e senha são gravadas nas configurações do mundo da campanha. A preferência de tela também é aplicada às configurações globais da janela para que o desktop entre no modo escolhido. A campanha só é criada depois que a última etapa é confirmada.

Ao continuar um save existente, o mundo é carregado primeiro e a tela de login aparece sobre `public/assets/kali-cubes2.jpg`. O painel central apresenta o logo Kali, usuário, senha, hostname e estado da sessão. A entrada no desktop só ocorre após a validação das credenciais persistidas; saves antigos usam o usuário da campanha e a senha compatível de primeiro acesso como fallback.

O instalador continua o fluxo com as etapas traduzidas de mídia, componentes, rede, hostname, domínio, particionamento virtual, nome completo, usuário, senha, fuso horário e modo de tela. A seleção de disco é apenas uma representação do armazenamento virtual da campanha e nunca particiona arquivos reais.
