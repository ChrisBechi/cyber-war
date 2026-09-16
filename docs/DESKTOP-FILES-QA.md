# Arquivos, atalhos e arrastar e soltar

## Atalhos de arquivos

Funcionam no desktop e no gerenciador de arquivos. Campos de texto mantêm seus
atalhos normais de edição. Diálogos, bloqueio e transições da sessão impedem ações
sobre os arquivos ao fundo.

| Atalho                   | Ação                                        |
| ------------------------ | ------------------------------------------- |
| Ctrl+Shift+N             | Criar pasta                                 |
| Ctrl+N                   | Criar arquivo                               |
| Ctrl+C / Ctrl+X / Ctrl+V | Copiar, recortar e colar a seleção          |
| Ctrl+A                   | Selecionar todos os itens exibidos          |
| Setas                    | Selecionar item                             |
| Enter                    | Abrir o item selecionado                    |
| F2                       | Renomear o item selecionado                 |
| Delete                   | Enviar a seleção para a lixeira recuperável |
| F1                       | Exibir a lista completa de atalhos          |

Ctrl+clique permite selecionar vários itens. A ajuda também lista atalhos para
aplicativos, janelas e áreas de trabalho. Minimizar para mostrar o desktop guarda
quais janelas estavam visíveis para restaurá-las depois.

## Arrastar e soltar

- Arquivos, pastas e atalhos do desktop e do gerenciador produzem uma imagem
  fantasma com o ícone e o nome. Seleções múltiplas mostram a quantidade.
- Pastas e áreas vazias do desktop/gerenciador recebem os itens por movimentação.
- A lixeira recebe itens sem apagá-los definitivamente.
- Aplicativos compatíveis, seus lançadores no painel e suas janelas recebem um
  arquivo por vez para abri-lo. O arquivo permanece na pasta original.
- O navegador mostra arquivos virtuais de texto, imagem, áudio e vídeo. Texto e
  HTML são exibidos como texto, sem executar scripts do arquivo.
- Destinos incompatíveis, arquivos comuns, controles da interface, pastas sem
  permissão, colisões de nomes e movimentos para dentro da própria pasta têm
  `dropEffect = none`. Soltar nesses destinos não altera arquivos nem abre apps.
- O destino válido recebe destaque. O destaque e o fantasma são removidos ao
  concluir ou cancelar o arraste. O espaçamento dos ícones permanece em 2 px.

O arraste aceita somente itens do sistema de arquivos virtual. Importar arquivos
do computador continua disponível pelo botão existente. A configuração Tauri
`dragDropEnabled: false` permite o arraste HTML no WebView2 do Windows.

## Nomes repetidos

Copiar e colar e fazer downloads preservam o arquivo já existente e escolhem o
primeiro contador disponível: `notas.txt`, `notas (1).txt`, `notas (2).txt`.
Extensões compostas são preservadas, como `pacote (1).tar.gz`. Pastas usam
`Pasta (1)`. A numeração já presente no nome é reconhecida.

Downloads do navegador, anexos e downloads por wget/curl que gravam arquivos usam
a mesma regra. Os caminhos retornados e as mensagens de download indicam o nome
real gravado. Movimentar/renomear continua rejeitando colisões; o comando `cp` do
terminal mantém sua semântica anterior.

## Verificação em 15/09/2026

- 196 testes do frontend e 115 testes do backend passaram.
- TypeScript, ESLint dos arquivos alterados, Stylelint e Clippy passaram.
- Build web de produção e executável nativo de desenvolvimento compilados.
- Sessão nativa isolada, com SQLite em memória e IPC real: criação de pasta e
  arquivo pelo teclado, renomear, copiar com `(1)`/`(2)`, mover para pasta/lixeira,
  Delete, abrir texto e imagem no navegador, downloads numerados e ajuda F1.
- Eventos de arraste no WebView confirmaram bloqueio sobre app incompatível,
  arquivo comum e menu, sem alterações no VFS; pasta válida recebeu `move` e
  destaque. Um início de arraste por mouse produziu o fantasma com ícone e nome.
- A captura de janela pelo controle do Windows falhou com `SetIsBorderRequired`
  (`0x80004002`). Capturas pelo próprio WebView permitiram inspecionar a interface.
  A aparência do cursor nativo durante todo o gesto não foi confirmada visualmente.
- `artifacts/desktop-shortcuts-qa.png`: lista de atalhos na sessão nativa.
- `artifacts/desktop-drag-preview-qa.png`: elemento real da imagem fantasma,
  colocado temporariamente em uma posição visível apenas para inspeção da aparência.

O build web mantém os avisos anteriores sobre o tamanho do bundle e comentários
de pureza do Zod. O executável/instalador de distribuição não foi reconstruído.
