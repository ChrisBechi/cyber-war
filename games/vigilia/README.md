# SECTOR IX — Protocolo Zero

Jogo original de tiro 2D para navegador, em português, sem dependências de execução.

- Três operações: Distrito Chuva, Usina Helix e Núcleo Zero; chefes em três fases, plataformas, perigos anunciados, checkpoints e seis segredos.
- Seleção gratuita de Dante, Kaia, Ravi e Nika, alternando homem e mulher. Cada personagem usa quatro quadros próprios de animação.
- Loja com roupas, acessórios, equipamentos e quatro armas. Créditos e arsenal são salvos localmente; não há pagamentos reais.
- HUD sobreposta com vida, munição e pausa. Informações da missão, controles, loja, tela cheia e áudio ficam na pausa.
- Trilha procedural original com arranjos por fase, efeitos de combate e volumes separados. O áudio começa após interação do jogador.

## Executar

Sirva a pasta `dist` com um servidor HTTP estático e abra `index.html`. Os módulos ES não devem ser abertos por `file://`.

Controles: A/D ou setas para mover; espaço, W ou seta para cima para salto duplo; mouse para mirar e atirar ou J com mira assistida; Shift/K para impulso; Q para pulso; R para recarga; 1–4 para armas liberadas; Esc/P para pausa; B para loja; M para áudio. Controles de toque aparecem em dispositivos compatíveis.

## Arquitetura e design

`engine.js` contém a simulação determinística a 60 Hz e o desenho das fases. `render.js` desenha cenários, sprites, roupas e partículas. `game.js` coordena telas e entradas; `characters.js` contém o elenco e `store.js` valida o progresso. `audio.js` sintetiza música e efeitos com Web Audio, canais separados e controle de picos.

O design system em `style.css` usa fundo carvão, tipografia condensada nos títulos, verde menta para ações e integridade, rosa para ameaças e âmbar para recompensas. As telas de seleção e loja adaptam a grade ao espaço disponível. A área de jogo mantém suas proporções, e a interface de combate fica sobre o canvas.

Artes originais geradas com ImageGen estão em `dist/assets`; os arquivos `*-prompt.json` e `art-prompts.json` registram os prompts. A transparência dos sprites foi preservada. Recortes e variantes de roupas são aplicados pelo renderer.

## Validação

`node --test tests/engine.test.js` verifica movimento, combate, progressão, extração, compras, seleção e persistência de personagens, atlas de animação e mixagem de áudio. Os arquivos de JavaScript também passam por `node --check`. Os testes de travessia isolam a geometria das fases; não substituem uma partida manual no navegador.
