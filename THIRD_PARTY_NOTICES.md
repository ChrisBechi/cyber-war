# Recursos do Kali Linux

A interface do LifeOS adapta o tema **Kali-Dark** para HTML/CSS. Os ícones em
`public/assets/kali` são recursos SVG dos projetos oficiais abaixo. O jogo não
inclui o ambiente GTK/Xfce nem os executáveis dos aplicativos Kali.

- **kali-themes**, commit `83ecd11256b5e3e09ba1cc41cf3032ace45c405a`:
  <https://gitlab.com/kalilinux/packages/kali-themes>.
  Copyright 2015–2019 Offensive Security; tema de Jared Sot e Daniel Ruiz de Alegría.
- **kali-menu**, commit `cb91e822db8f9c175e1513446915a8dd14195e18`:
  <https://gitlab.com/kalilinux/packages/kali-menu>.
  Copyright © 2012 Kali.

Ambos os pacotes declaram GPL-3.0-or-later. Os avisos originais e o texto da licença
acompanham os recursos em `public/assets/kali/`: `kali-menu-copyright.txt`,
`kali-themes-copyright.txt` e `COPYING.txt`. Os SVG distribuídos são seu formato
fonte, sem alterações. A adaptação dos estilos está em `src/styles/kali.css`.
Marcas e logotipos pertencem aos respectivos titulares; sua presença identifica
as referências visuais e os aplicativos do catálogo, sem sugerir afiliação.

O fundo `public/assets/kali/installer-boot.png` é uma cópia sem alterações de
`build/boot/x86/pics/kali.png` do pacote oficial `debian-installer`, commit
`bfb1e50a300c52c64801ac60a052f215219bdfcc`, referenciado pelo submódulo
`src/boot-menu/debian-installer` da versão de `kali-themes` acima. A arte é
gerada de `src/boot-menu/syslinux.svg` e está coberta pelos avisos do tema.
Fonte: <https://gitlab.com/kalilinux/packages/debian-installer/-/blob/bfb1e50a300c52c64801ac60a052f215219bdfcc/build/boot/x86/pics/kali.png>.

O fundo `public/assets/kali/system-boot.png` é uma cópia sem alterações de
`share/grub/themes/kali/grub-4x3.png` do mesmo commit de `kali-themes`. Ele é
usado no menu de inicialização apresentado após concluir as configurações.
Fonte: <https://gitlab.com/kalilinux/packages/kali-themes/-/blob/83ecd11256b5e3e09ba1cc41cf3032ace45c405a/share/grub/themes/kali/grub-4x3.png>.

O catálogo local foi gerado a partir dos metadados dos pacotes oficiais,
<https://www.kali.org/tools/kali-meta/> e <https://www.kali.org/tools/all-tools/>,
consultados em 12 de setembro de 2026. A edição selecionada é `kali-linux-default`
com `kali-desktop-xfce`; os conjuntos opcionais `large` e `everything` não entram
no catálogo. Comandos `Exec` dos atalhos Linux não são importados nem executados.

O cabeçalho do instalador usa apenas a faixa superior da referência `1.png`
fornecida pelo usuário, preservada sem alterações em
`public/assets/kali/installer-reference.png` e enquadrada por CSS.

O instalador inclui DejaVu Sans 2.37 (regular, negrito e oblíquo), sem alterações.
Fonte: <https://dejavu-fonts.github.io/Download.html>. O arquivo original
`dejavu-fonts-ttf-2.37.zip` foi verificado pelo SHA-256
`7576310b219e04159d35ff61dd4a4ec4cdba4f35c00e002a136f00e96a908b0a`.
A licença acompanha as fontes em `public/assets/kali/DejaVu-LICENSE.txt`.
