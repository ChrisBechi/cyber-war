# Spec 06 — Mission Engine

## Objetivo

Evitar missões hardcoded em React.

## Mission Definition

Conteúdo data-driven com:

- id;
- title;
- requirements;
- startTriggers;
- stages;
- outcomes;
- rewards;
- worldChanges.

## Stage

Pode possuir objetivos, entrada, conclusão, diálogos, eventos, entidades virtuais, escolhas, caminhos alternativos e checkpoints.

## Condições

Consultar WorldState:

- flags;
- arquivos;
- hosts;
- técnicas;
- inventário;
- dinheiro;
- reputação;
- guilda;
- decisões anteriores.

## Não linearidade

A ordem das missões disponíveis pode variar, mas **apenas uma missão fica ativa por vez**, conforme decisão do usuário em 2026-09-14. O núcleo rejeita uma segunda tentativa; a interface desabilita seu início até concluir ou abandonar a atual. As referências anteriores a missões simultâneas estão substituídas por essa regra.

Tentativas possuem diário de efeitos temporários. Saída, load manual/autosave e restore descartam a tentativa, sem restaurar o mundo inteiro. Arquivos pessoais e configurações continuam duráveis. Novas mecânicas devem declarar seus recursos e preservar a proveniência de arquivos temporários. Consulte [a implementação e seus limites](../RELATORIO-CORRECOES-0.4.2.md).

## Checkpoints

Criar automaticamente no início/fim de missão e antes de decisões irreversíveis definidas pelo conteúdo.
