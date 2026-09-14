# Histórico de versões

## 0.4.2 — 13/09/2026

- Release com o fluxo completo do instalador Kali em português, menus de boot de entrada e saída, ajuda contextual, rede `Vizinho_5G` e persistência das configurações virtuais.
- Instalador refeito conforme a sequência das 25 referências: cabeçalho original, listas, campos e botões no modelo clássico; adicionadas a detecção da conexão e a preparação das partições.
- Barras automáticas com preenchimento dentro de uma trilha fixa, de 0 a 100%, durante 4 segundos por etapa. A tela só muda depois de exibir o preenchimento completo.
- Métodos de particionamento, edição manual e tabelas aplicadas ao sistema virtual, consultáveis em `lsblk` e `/etc/fstab`; rede e seleção de software persistidas na campanha.
- Preferências de janela e resolução seguem o menu de configurações, sem inserir uma tela extra na sequência do instalador.

## 0.4.1 — 13/09/2026

- A inicialização anterior ao trailer preenche duas telas com logs de comprimentos variados.
- O conteúdo acompanha automaticamente as novas linhas, sem barra de rolagem, e preserva espaço no rodapé.
- O comando `./cyber-war --start` aparece ao fim da segunda tela.
- Os efeitos, a duração da intro e o trailer aprovado foram preservados.
- Novo jogo agora usa um configurador de instalação em pt-BR com teclado, modo de tela, resolução, computador, usuário e senha.
- Continuar um save passa pela tela de login com o fundo `kali-cubes2` antes de abrir o desktop.
- O papel de parede inicial do desktop usa `kali-maze`.
- O instalador separa o hostname da conta local, e o terminal abre uma linha de respiro antes da saída de cada comando.
- O fluxo do instalador foi ampliado com mídia, componentes, rede, domínio, particionamento, nome completo, usuário, senha e fuso horário, todos em português.
- O particionamento agora reproduz o fluxo do Kali com seleção do disco virtual, cinco esquemas, visão geral, confirmação de gravação e formatação simulada.
- A instalação inclui seleção de Xfce/GNOME/KDE e perfis de ferramentas, progresso de 4 segundos por etapa e tela final de reinicialização.
- Cada etapa do instalador ganhou ajuda contextual; a interface escolhida aparece em `ip`/`ifconfig` e o layout selecionado pode ser consultado com `lsblk` dentro do terminal virtual.
- A varredura de redes sem fio agora mostra automaticamente o sinal virtual `Vizinho_5G`, disponível como pista para a missão de Wi-Fi.
- O instalador agora começa no menu de boot gráfico do Kali e termina no menu de inicialização do sistema, com fundo `kali-waves` responsivo em 16:9.

Distribuição Windows: instalador NSIS, pacote MSI e executável independente.
