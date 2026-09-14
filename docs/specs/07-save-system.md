# Spec 07 — Save System

## Política implementada no checkpoint 0.4.2

Snapshots novos usam schema 2; schema 1 tem migração seletiva. Toda entrada por load/restore descarta a tentativa ativa antes da publicação e persistência normalizada. Save manual durante tentativa não permite retomá-la. Conclusão promove resultados ao manual; arquivos pessoais e configurações não são revertidos junto com a tentativa. Ausência de referência necessária em save legado produz erro, preservando o original.

O encerramento passa pelo mesmo `GameService` na GUI, fechamento nativo e saída do aplicativo. Falha SQL não publica rollback parcial. Contextos de terminal e buffers não são restaurados. As regras e limites atuais estão no [relatório de correções](../RELATORIO-CORRECOES-0.4.2.md); elementos futuros descritos abaixo não equivalem a sistemas já completos.

## Requisito fixo

O jogador possui **5 slots principais de armazenamento**.

## Estrutura

Cada slot mantém:

- snapshot atual/manual;
- autosave;
- checkpoint de início de missão;
- checkpoint de fim de missão;
- checkpoints de decisões;
- histórico rotativo.

## UI

```text
[ SLOT 1 ]  12h43m  Missão: NO AR
[ SLOT 2 ]  Vazio
[ SLOT 3 ]  08h12m  Missão: A GAROTA
[ SLOT 4 ]  Vazio
[ SLOT 5 ]  31h05m  Missão: ROOT
```

## Checkpoints

Limite inicial: **20 checkpoints por slot**.

Tipos:

- `mission_start`;
- `mission_end`;
- `decision`;
- `autosave`.

Isso permite **Voltar ao início desta missão** sem consumir um sexto slot.

## Autosave

Salvar em pontos seguros: início/fim de missão, mudanças grandes de estado e encerramento correto.

Não autosalvar depois de decisão irreversível sem checkpoint anterior.

## Integridade

Cada snapshot usa SHA-256 para detectar corrupção acidental.

Binários são armazenados fora do JSON em `vfs_blobs` com referências SHA-256/tamanho/MIME. Migração do banco SQLite: user_version 3; schema do mundo permanece 2 (campo blob opcional). Bytes novos e snapshot pertencem à mesma transação. Load/restore hidratam e verificam blobs, inclusive os guardados apenas no journal de tentativa, antes da normalização/publicação. Erros preservam o estado vivo e o save. Não há coleta de blobs antigos nesta etapa: limite de 512 MiB retidos, somados os slots/checkpoints. Veja os demais limites no [aprofundamento](../APROFUNDAMENTO-0.4.2.md).

## Estado persistido

Flags, missões, dinheiro, reputação, contatos, mensagens, inventário, VFS, redes, hosts alterados, credenciais, decisões, tempo, estado relevante de apps, personas/campanhas futuras e evidências futuras.
