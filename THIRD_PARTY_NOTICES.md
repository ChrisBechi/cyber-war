# Recursos do Kali Linux

## Archive e compressão

O backend utiliza zip 8.6.0 (MIT), tar 0.4.46, flate2 1.1.10, bzip2 0.6.1 e
liblzma 0.4.8 (MIT ou Apache-2.0). O decoder bzip2 utiliza libbz2-rs-sys 0.2.5
(bzip2-1.0.6); a biblioteca nativa liblzma é fornecida por liblzma-sys 0.4.9
com código XZ Utils sob 0BSD. O build Windows usa embed-resource 3.0.11 (MIT).

As licenças completas das dependências adicionadas e seus componentes transitivos
estão em `src-tauri/resources/archive-licenses.txt`, incluído nos recursos do bundle.
A decisão está em `docs/adr/0002-vfs-archives.md`.

## Interface

O dragão ASCII dos comandos virtuais `fastfetch` e `neofetch` vem de
`src/logo/ascii/k/kali.txt` do [Fastfetch](https://github.com/fastfetch-cli/fastfetch),
commit `02dc6a4428c015cdf8b003239be3c4238fdcd5f4`. A cópia original e a licença MIT
estão em `public/assets/fastfetch/`. A implementação do jogo converte os marcadores
de cor do desenho em ANSI; não inclui nem executa o programa Fastfetch do sistema.

O gerenciador de arquivos utiliza SVGs originais, sem alterações, do tema
**Flat-Remix-Blue-Dark**, de Daniel Ruiz de Alegría e colaboradores:
<https://github.com/daniruiz/flat-remix>, commit
`e7de6c346da46e008987228f363b0eae6e638637`. Os arquivos-fonte estão em
`public/assets/flat-remix/`, acompanhados da licença GPL-3.0 (`LICENSE.txt`),
autoria (`AUTHORS.txt`) e caminhos de origem com hashes SHA-256 (`sources.json`).
Os links simbólicos do repositório foram resolvidos para os SVGs originais;
apenas tamanho de exibição e cor dos ícones simbólicos são definidos por CSS.

A ilustração `public/assets/browser/no-connection.svg` é distribuída sem alterações
do [repositório oficial do Firefox](https://github.com/mozilla-firefox/firefox/blob/main/toolkit/themes/shared/illustrations/no-connection.svg),
consultado em 15 de setembro de 2026. Mozilla Public License 2.0; o cabeçalho
original e a licença em `public/assets/browser/MPL-2.0.txt` acompanham o arquivo.

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
