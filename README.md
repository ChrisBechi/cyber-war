# Cyber War

Jogo desktop Windows em Tauri 2, React/TypeScript e Rust. O LifeOS contém terminal xterm, arquivos, redes e serviços virtuais compartilhados por todas as aplicações. Os saves ficam em SQLite local.

## Jogar

Versão atual: **0.4.2**. Consulte as alterações em [CHANGELOG.md](CHANGELOG.md).

Consolidação técnica em andamento: consulte o [relatório de correções](docs/RELATORIO-CORRECOES-0.4.2.md) e o [baseline](docs/BASELINE-CONSOLIDACAO-0.4.2.md). Os executáveis/instaladores existentes podem anteceder essas alterações; não são uma certificação deste checkpoint.

Use o instalador NSIS em `src-tauri/target/release/bundle/nsis/` ou o MSI em `src-tauri/target/release/bundle/msi/`. O executável independente está em `src-tauri/target/release/game-hacker.exe` e requer o WebView2 do Windows. O jogador não precisa de Node, Rust ou servidor.

Em **Novo jogo**, escolha um dos cinco slots e passe pelo configurador do LifeOS em português: idioma, localização, teclado, mídia, rede, domínio, nome do computador, conta, senha, relógio, particionamento guiado com cinco esquemas, seleção de desktop e ferramentas, modo de tela e resolução. O particionamento é inteiramente virtual e segue as telas do instalador Kali, incluindo confirmação, formatação e instalação do sistema básico. Ao continuar um save, o sistema exibe a tela de login com o fundo Kali antes de abrir o desktop. Abra **Trabalhos** para acompanhar objetivos. Há pistas opcionais em cada missão. Duplo clique abre os ícones do desktop; `Ctrl+Alt+T` abre o terminal.

O desktop começa com o papel de parede `kali-waves`; ele também pode ser trocado no aplicativo Configurações.

Botão direito no desktop abre o menu de arquivos, atalhos e aplicativos. Os oito comandos de arquivos/navegação revisados têm [contratos de flags e limites](docs/CONTRATOS-COMANDOS-0.4.2.md); o terminal continua uma simulação isolada, não um Linux completo.

Todas as etapas do configurador têm o botão **Ajuda**, com explicações para as opções da rede, dos discos, das partições e dos pacotes. As escolhas ficam salvas na campanha: `ip`/`ifconfig` mostram a interface selecionada e `lsblk` mostra o layout virtual criado pelo esquema de partição.

O laboratório de redes sem fio abre já com a varredura virtual preenchida, incluindo `Vizinho_5G`; o sinal fica disponível como pista para a missão de Wi-Fi.

O configurador começa no menu gráfico de boot do Kali e exibe o menu de inicialização ao concluir a instalação. O fundo usa a arte Kali em 3840 × 2160, com enquadramento responsivo para preservar a proporção sem faixas.

O jogo inicia em tela cheia. `F11` alterna entre tela cheia e janela, também disponível pelo botão no menu e no painel superior. No terminal, `Tab` completa comandos e caminhos da pasta atual; `←`/`→`, `Home`/`End`, `Backspace` e `Delete` permitem editar a linha antes de executar. Os ícones do desktop preenchem cada coluna de cima para baixo e passam à próxima conforme a altura disponível.

O recorte jogável inclui First Boot, V1, V2 com solução alternativa, Wi-Fi, pendrive, trabalhos de VEX, hardening/incidente e A Garota. A sessão 2 possui as duas missões iniciais Orion: NO AR e EM CLARO. Os demais arcos permanecem em documentação. Apenas uma missão pode estar ativa por vez; concluir ou abandonar libera a possibilidade de aceitar outra.

### SECTOR IX — Protocolo Zero

O jogo externo de `C:\Users\chris\Documentos\ultimo-magnata\artifacts\vigilia` foi incorporado integralmente em `games/vigilia/` e empacotado para a execução interna em `public/games/vigilia/`. Ele aparece em **Aplicativos > Jogos > SECTOR IX**, abre inicialmente em janela e mantém a opção de tela cheia no próprio cabeçalho do jogo. Perfil, créditos, personagens, arsenal, volumes e progresso do SECTOR IX continuam persistidos no armazenamento do runtime.

O painel lateral do aplicativo é o trainer interno: permite ajustar créditos, vida, munição, pontuação, desbloquear o arsenal, proteger a integridade da agente, trocar de operação e concluir a fase atual. As alterações atuam apenas no estado virtual do jogo embarcado.

Para a página de download, abra `https://www.vigilia.org` no navegador interno. Ela explica o fluxo Linux com `wget`, `chmod`, `bash` e `sector-ix`; o botão de download grava `sector-ix-linux.sh` e `SECTOR-IX-README.txt` em `~/Downloads`, sem tocar no sistema operacional real. Ao executar o instalador, a árvore visual é criada em `~/Games/sector-ix`, com arquivos representativos vazios; somente `saves/sector-ix.save` contém o estado funcional do jogo.

## Desenvolver

Pré-requisitos: Node 24, pnpm 11.19.0, Rust estável com rustfmt/clippy, Microsoft C++ Build Tools e WebView2.

```powershell
pnpm install --frozen-lockfile
pnpm dev
pnpm check
pnpm build
```

Nesta máquina, a instalação de Rust foi isolada em `.tools/`. O script abaixo configura o ambiente da sessão e localiza o pnpm já disponível:

```powershell
.\scripts\dev.ps1 dev
.\scripts\dev.ps1 check
.\scripts\dev.ps1 build
```

`pnpm dev:web` serve apenas a interface para desenvolvimento. O gameplay usa IPC com Rust e só funciona no aplicativo Tauri. Não existe servidor HTTP do núcleo nem substituto de gameplay em JavaScript.

## Persistência

Cinco slots independentes, save manual, autosave após mudanças e ao fechar, checkpoints de início/fim/decisão e até 20 checkpoints por slot. A tela **Campanha** permite salvar e restaurar. **Continuar** usa o autosave; o menu também permite recuperar o último save manual.

Carregar qualquer snapshot descarta a tentativa de missão ainda em andamento, inclusive após saída forçada. Efeitos temporários são separados de arquivos pessoais, configurações, pacotes e acessos de rede. Conclusões são duráveis. No login, janelas e terminais são limpos; só aplicações e serviços explicitamente habilitados voltam. Bloquear a tela mantém a sessão.

O fórum usa conteúdo e comunidade fictícios locais, não uma rede social online. Terminal, shell, nano, serviços e ferramentas implementam subconjuntos documentados; não equivalem aos executáveis reais do Linux. O [aprofundamento atual](docs/APROFUNDAMENTO-0.4.2.md) inclui importação explícita de arquivos binários (até 32 MiB), armazenamento deduplicado em SQLite e integração com os players. Codecs dependem do WebView2; PDF e arquivos compactados ainda não têm leitor/extrator. Blobs antigos são conservados para proteger checkpoints; a capacidade binária retida do banco é 512 MiB, sem coleta automática nesta etapa.

O banco `game-hacker.db` fica no diretório de dados do aplicativo, normalmente `%APPDATA%/com.gamehacker.desktop/` no Windows. Atualizações usam migrações idempotentes. Cada snapshot inclui SHA-256; gravação e checkpoints pertencem à mesma transação. A memória só muda após o commit. Saves incompatíveis/corrompidos são recusados com erro visível.

## Organização

- `src/features/`: aplicativos e gerenciamento de janelas.
- `src/lib/`: contratos Zod, cache de UI e fila de IPC.
- `src-tauri/src/`: VFS, rede, comandos, missões, navegador, transações e saves.
- `content/`: missões, mensagens das missões, tópicos de fórum e rede inicial.
- `scripts/content-schema.mjs`: validação de conteúdo usado pelo build/check.
- `docs/specs/`: as 15 especificações extraídas do pacote original de `specs/`.
- `docs/IMPLEMENTATION.md`: decisões, cobertura por spec e limites da entrega.
- `docs/QA.md`: verificações e procedimento reproduzível.

O CommandEngine possui apenas comandos internos. Não há subprocessos, sockets de gameplay, acesso genérico ao disco do Windows nem fallback para um shell real.
