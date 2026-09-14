# Spec 02 — LifeOS / Kali UI

## Alvo visual

Interface inspirada no **Kali Linux com XFCE**, sem VM real.

## Componentes

- intro/menu fora do Kali;
- desktop;
- painel superior;
- Applications;
- notificações;
- file manager estilo Thunar;
- terminal;
- navegador fictício;
- mensageiro;
- fórum;
- editor;
- ferramentas hackers.

## Consistência

`/home/kali/Desktop` alimenta os ícones do desktop.

Criar `notes.txt` no terminal deve fazê-lo aparecer no file manager imediatamente.

Mover um arquivo pela GUI deve alterar `ls`.

## WindowManager

Apps Kali são janelas internas React, não janelas Tauri separadas.

Suportar foco, z-index, mover, redimensionar, minimizar, maximizar e fechar.

Maximizar/restaurar interpola posição e tamanho em 240 ms. Minimizar/restaurar pela barra interpola escala, deslocamento até o botão do aplicativo e opacidade em 220 ms. A janela minimizada continua montada para preservar buffers, mas fica inerte e fora da acessibilidade; a ocultação visual ocorre ao fim da transição. Arraste e redimensionamento manual respondem imediatamente. Respeitar `prefers-reduced-motion`.

O botão da barra de título mostra um quadrado para maximizar e dois sobrepostos para restaurar; título e nome acessível acompanham esse estado. Restaurar recupera a posição e as dimensões anteriores.

O aplicativo inicia em tela cheia nativa. `F11` e os botões no menu e painel superior alternam entre tela cheia e janela, inclusive com foco no terminal.

Os ícones do desktop preenchem a altura disponível antes de iniciar a próxima coluna; redimensionar a janela recalcula as linhas. A grade fica a 4 px das laterais e dos painéis, com colunas de 76 px, linhas de no mínimo 72 px e gaps de 2 px entre linhas e 4 px entre colunas. A altura restante é distribuída entre as linhas, sem reservar espaço para uma célula adicional que não cabe.

Os 12 aplicativos, pastas e arquivos usam SVGs próprios, com geometria centralizada em uma tela de 48 × 48. O mesmo conjunto aparece no desktop, launcher, barra de tarefas, títulos de janelas e gerenciador de arquivos. Os nomes têm até duas linhas com título completo acessível.

Dentro de cada célula, o nome fica a 4 px da borda inferior, e a área acima é ocupada por um SVG de até 48 px. Nomes em duas linhas reduzem a área do SVG proporcionalmente, sem sobreposição nem espaço vazio reservado abaixo do nome.

## Atualização 0.2.0 — referências oficiais Kali

O painel superior e o Whisker menu seguem as referências visuais fornecidas.
Usar ícones do kali-themes/kali-menu e adaptação do Kali-Dark para CSS. Manter
quatro áreas de trabalho, busca, categorias/subcategorias, favoritos e recentes
persistentes por campanha. A edição de software é kali-linux-default com Xfce;
não incluir ferramentas opcionais. O catálogo distribuído contém 303 entradas.

O painel direito inclui atividade do mundo virtual, rede, volume, notificações,
energia virtual, calendário, bloqueio e saída com autosave. F11 continua global;
a ação equivalente fica em Configurações. Ferramentas do catálogo abrem janelas
independentes, e inspeções/relatórios passam pela autoridade Rust/VFS. Detalhes
sobre fidelidade, fontes e escopo da simulação: docs/KALI-DESKTOP.md.