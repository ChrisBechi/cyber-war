# Spec 03 — Terminal Engine

## Estado do checkpoint 0.4.2

Cada janela envia `sessionId` para comando, autocomplete e nano; cwd/usuário/SSH/ambiente/editor são independentes. Histórico e buffers são runtime. `rm` remove do VFS e não usa a lixeira da GUI. O shell e os comandos abaixo continuam subconjuntos; não existe certificação GNU. Opções nano sem implementação são recusadas. O [relatório atual](../RELATORIO-CORRECOES-0.4.2.md) distingue o que funciona e o que ainda falta; o roadmap abaixo não é lista de conformidade concluída.

## Regra absoluta

O [contrato vertical dos comandos](../CONTRATOS-COMANDOS-0.4.2.md) registra flags, resultados, erros, permissões e limites testados de pwd/cd/cat/head/tail/cp/mv/rm. Saídas de leitura preservam stdout/stderr mesmo com status não zero; mutações ainda seguem atomicidade do jogo.

O [aprofundamento](../APROFUNDAMENTO-0.4.2.md) documenta ls/grep/find/chmod/chown, serviços/processos, validação de pacotes, parser shell e nano. Expansão produz argumentos, nunca novo código. Shell filho isola contexto, source compartilha e os retornos mantêm stdout/stderr/status. Opções fora do contrato são recusadas; o roadmap não garante suporte.

O terminal visual **nunca executa shell real**.

Proibido encaminhar input do jogador para `std::process::Command`, `cmd.exe`, PowerShell, Bash, WSL, SSH real ou subprocesso arbitrário.

## Pipeline

```text
xterm input
→ tokenizer/parser
→ CommandEngine
→ builtin / virtual tool
→ Simulation Core
→ CommandResult
→ ANSI
→ xterm
```

## Roadmap de comandos

### T0

`pwd`, `cd`, `echo`, `clear`, `whoami`, `id`, `uname`, `help`.

### T1 — filesystem

`ls`, `mkdir`, `touch`, `cat`, `head`, `tail`, `cp`, `mv`, `rm`, `tree`, `stat`, `chmod`, `chown`, `sudo`, `su`.

### T2 — utilitários

`grep`, `find`, `less`, editor bridge, `nano`, `file`, `strings`, `base64`, `sha256sum`, `wc`, `sort`, `uniq`, `cut`, `tr`, `seq`, `basename`, `dirname`, `realpath`, `which`, `df`, `du`, `man`.

### T3 — rede virtual

`ip`, `ifconfig`, `route`, `ping`, `arp`, `ss`, `netstat`, `dig`, `nslookup`, `traceroute`, `curl`, `wget`, `ssh` virtual, `scp` virtual.

### T3 — ambiente e serviços

`hostname`, `date`, `env`, `printenv`, `export`, `groups`, `ps`, `top`, `free`, `uptime`, `killall`, `pkill`, `service`, `systemctl`, `journalctl`.

### T3 — pacotes Debian/Kali

`apt`, `apt-get`, `apt-cache`, `apt-mark`, `dpkg` e `dpkg-query` mantêm um inventário de pacotes dentro do estado salvo do jogo. `update`, `install`, `remove`, `search`, `show`, `list`, `policy` e `-y` são virtuais; alterações administrativas exigem `sudo`.

## Associações de arquivos e mídia

O gerenciador usa a extensão e o MIME declarado no VFS para escolher o aplicativo virtual: texto e documentos abrem no HackPad, `.sh`/`.bash` no Bash virtual, áudio e vídeo no Parole Media Player e imagens no Ristretto Image Viewer. O player de mídia oferece play/pause, avanço e retorno de 10 segundos, seek pela barra, volume, mute, velocidade, repetição, tela cheia e legendas VTT/SRT. O visualizador de imagens oferece ajuste à janela, zoom, rotação e tela cheia.

Scripts são executados linha a linha pelo VFS com shebang, comentários, `export`, argumentos posicionais, `$@`, `$#`, `$?` e `source` controlados. Nenhum `.sh` chama executáveis ou arquivos do Windows. Players usam decodificação do WebView2, assets locais ou bytes explicitamente importados pelo usuário; não lançam um player/processo do host. Importação e limites binários estão na spec 04.

### T4 — ferramentas de gameplay

Ferramentas reais ou fictícias consultam apenas VirtualNetwork e conteúdo de missão.

## Parser

Não implementar Bash inteiro cedo.

Começar com command, args e quotes. Pipes, redirects e vars entram incrementalmente.

## Edição e autocomplete

As setas esquerda/direita movem o cursor; Home/End e Ctrl+A/E navegam até os extremos. Backspace/Delete removem ao redor do cursor, preservando o restante da linha. O histórico mantém o rascunho quando o jogador retorna à linha atual.

Tab completa comandos e caminhos do VFS da sessão ativa, local ou SSH virtual. Caminhos relativos usam o diretório atual; absolutos e `~/` também são aceitos. `cd` sugere apenas diretórios. Aspas, nomes com espaços e edição no meio da linha são preservados. Correspondências ambíguas estendem o prefixo comum ou exibem as opções; a conclusão não acessa arquivos do Windows.

## Nano interativo

`nano` usa a tela alternativa ANSI do xterm e uma sessão transitória no `TerminalSession`; o CSS do terminal não participa da tela do editor. O buffer é lido e salvo exclusivamente no `VirtualFileSystem`, inclusive durante sessões SSH virtuais. `Ctrl+X`, `Ctrl+O`, `Ctrl+W`, `Ctrl+\`, `Ctrl+K`, `Ctrl+U`, `Ctrl+G`, `Ctrl+R`, `Ctrl+_`, marcação, navegação, recorte/cola e desfazer/refazer são tratados no frontend.

O parser nano reconhece opções longas, clusters curtos, argumentos anexados (`-T4`) e `+LINHA,COLUNA`. Isso não equivale a implementar todas as opções GNU: opções sem implementação, inclusive speller, nanorc e locking, retornam erro explícito. A classificação está no relatório de correções; nenhum editor do host é executado.
