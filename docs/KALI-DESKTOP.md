# Desktop Kali no LifeOS

## Edição e aparência

A versão 0.2.0 usa os SVG oficiais de `kali-menu` e `kali-themes`, com adaptação
dos estilos Kali-Dark para a interface React. Os commits e as licenças estão em
`THIRD_PARTY_NOTICES.md`. O painel superior tem 36 px e reproduz a disposição das
duas referências enviadas: lançadores e quatro áreas à esquerda; atividade,
rede, áudio, notificações, energia, relógio, bloqueio e saída à direita.

O gráfico representa atividade do núcleo do jogo; a bateria representa o computador
virtual. Volume é uma preferência da sessão. Bloquear mantém as janelas montadas e
impede interação até desbloquear. Sair grava o autosave antes de retornar ao menu.

## Menu e aplicativos

O menu tem pesquisa, favoritos, recentes, todas as aplicações, aplicações usuais
e as 16 categorias numeradas do Kali atual, incluindo subcategorias. As categorias
sem ferramentas na edição padrão continuam disponíveis. A pesquisa consulta nome,
pacote, descrição e comandos; Enter abre o primeiro resultado. Setas percorrem as
categorias e resultados. Escape ou clique externo fecham o menu. Ctrl+Escape também
alterna sua abertura. A estrela ou clique direito alternam favoritos.

`content/software/kali-default.json` contém 303 entradas de aplicativos, ferramentas
e referências, selecionadas pelas dependências de `kali-linux-default` e
`kali-desktop-xfce` (286 pacotes, incluindo metapacotes e suporte do sistema). Esse
número não representa 303 binários reais instalados. Os opcionais foram excluídos.
Favoritos e até 30 aplicativos recentes são salvos nas configurações da campanha,
sem alteração de schema nem compartilhamento entre slots.

Terminais, arquivos, editor, navegador e processos abrem os aplicativos existentes.
A calculadora possui operações locais. As demais ferramentas abrem workspaces
independentes com metadados oficiais e inspeções do mundo virtual: host/portas,
resposta HTTP, arquivo/hash, Wi-Fi ou processos, de acordo com a categoria.
Os relatórios são gerados no Rust e podem ser gravados no VFS com validação de
permissão e sem sobrescrever arquivos existentes. O laboratório não reproduz todas
as opções, módulos e interfaces dos programas originais. Os links oficiais ficam
disponíveis como referências locais, sem conceder acesso externo ao gameplay.

Os comandos presentes no catálogo participam da conclusão por Tab. Para comandos
sem implementação específica, `--help` descreve o instrumento e `--lab [ALVO]`
executa sua inspeção virtual. Comandos existentes mantêm a precedência e suas regras.

## Fontes e regeneração

`scripts/import-kali-catalog.mjs` lê os metadados baixados em
`artifacts/kali-reference/`: `all-tools.html`, `metapackages.html`, commits, arquivos
`.desktop`, `.directory`, `categories.yaml` e ícones. O script resolve a árvore
de metapacotes padrão e exclui atalhos de pacotes opcionais. Os ícones do tema são
extraídos de `theme-icons.zip` por `scripts/extract-kali-icons.ps1` (apenas SVG).

Os arquivos de referência são dados; nenhum script nem campo `Exec` do repositório
Kali é executado. `pnpm content:check` verifica o catálogo distribuído, incluindo
IDs, categorias, edição, metadados, integridade SVG e fontes das entradas.

## Verificação

Testes de menu verificam ordem dos favoritos, pesquisa, ausência de Ghidra opcional,
subcategorias, teclado, recentes e fechamento. Testes Rust verificam preferências
serializadas, allowlist de aplicativos, inspeção real do mundo, permissões e alvos
fora da sandbox. As janelas de ferramentas também têm teste de independência.
A prévia web em `artifacts/desktop-preview.html` usa uma fixture explícita, sem
acesso aos saves nem ao Rust; serve somente à verificação visual e de interação.
