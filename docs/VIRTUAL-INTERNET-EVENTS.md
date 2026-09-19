# Internet virtual — eventos e temporalidade

Segunda etapa implementada em 18/09/2026. O pacote `web-core@2` conserva as dez marcas e passa a ter **990 documentos, 22 entidades e 3.021 links validados**. A expansão se limita a quatro publicações que demonstram um acontecimento compartilhado entre sites.

## Experiência disponível

Abra `shopnow.com/produto/nexphone-x2` no navegador do jogo. O relógio usa o tempo acumulado da campanha; as transições abaixo não dependem do relógio civil nem de uma conexão real.

| Tempo acumulado | Preço do X2 | Estoque          | Efeito público                                             |
| --------------- | ----------- | ---------------- | ---------------------------------------------------------- |
| Antes de 10 min | R$ 1.899,00 | Em estoque       | Catálogo inicial                                           |
| 10 min          | R$ 1.799,00 | Em estoque       | Nota institucional na Nexora e página de oferta na ShopNow |
| 20 min          | R$ 1.799,00 | Últimas unidades | Conversa sobre a campanha no Redditor                      |
| 30 min          | R$ 1.899,00 | Indisponível     | A página temporária da oferta sai do ar e da busca         |
| 40 min          | R$ 1.849,00 | Em estoque       | Reposição noticiada pelo B1                                |

Produto, cards, recomendações, carrinho e resultados do Goggle consultam a mesma oferta. A página do produto mostra a data de atualização e um histórico expansível de preço e estoque. Itens indisponíveis continuam removíveis do carrinho, ficam fora do total e não podem ser adicionados novamente. Incluir no carrinho não reserva unidades, não congela o preço e não cria um pedido.

As páginas abertas recebem atualizações enquanto o desktop está ativo. A verificação ocorre a cada dez segundos, sem gravar nada quando não há uma transição prevista. Publicações e comentários futuros permanecem ocultos; comentários autorais ligados a uma publicação narrativa mantêm seu intervalo relativo à publicação.

O tempo passado depois de encerrar a sessão não é acrescentado à campanha ao entrar novamente. A tela bloqueada mantém a sessão ativa, como já ocorre com o autosave do jogo.

## Contrato e persistência

`virtual_web/events.rs` projeta definições imutáveis sobre o relógio e as flags de missão. Cada evento contém `id`, `requiredFlag` opcional, `afterSeconds`, `publish`, `remove` e alterações de oferta (`documentId`, `priceCents`, `stock`). Um documento tem no máximo um evento de publicação. A remoção prevalece sobre a publicação enquanto seu evento estiver ativo.

As alterações são aplicadas por instante de ativação; o ID desempata eventos simultâneos. Eventos de tempo usam o instante agendado, mesmo quando várias transições são processadas numa única chamada. Eventos narrativos registram o instante em que a condição foi observada. Repetir a sincronização não duplica efeitos. Reverter o relógio ou as flags recompõe o estado correspondente.

O save mantém `eventStartedAt`, os IDs ativos e as remoções projetadas. Não copia documentos, versões completas de página ou histórico de ofertas: esses dados são reconstruídos do pacote. As versões 1 e 2 do save são aceitas e normalizadas; versões futuras continuam sendo recusadas.

`GameService` sincroniza os eventos antes de validar uma interação e novamente após avaliar as missões. O comando `web_tick` só abre uma transação quando uma fronteira temporal foi atingida. A atualização só chega ao mundo ativo depois do commit SQLite. Uma falha de gravação preserva o estado anterior e permite tentar novamente. Carga, restauração e encerramento também normalizam as projeções.

O cache de busca inclui os instantes dos eventos. Visibilidade é conferida igualmente em navegação direta, busca textual, autocomplete e imagens. Um item oculto que já esteja no carrinho aparece como “Produto indisponível”, sem revelar título, preço ou URL bloqueados.

O pipeline rejeita preço negativo, estoque desconhecido, alteração de oferta em documento que não é produto, publicação duplicada, atualização anterior à publicação, links para documentos futuros e referências que antecipem eventos narrativos. As definições do cenário estão em `scripts/web-world-events.mjs`.

## Verificação

- **128 testes Rust** e **186 testes React** passaram. Cobrem limites exatos de tempo, idempotência, rollback, versões de save, cinco slots SQLite, coerência das ofertas, resposta do cache, comentários futuros, carrinho indisponível, atualização de páginas abertas e ausência de chamadas simultâneas do relógio.
- **8 testes do pipeline** passaram: `node --test scripts/web-schema.test.mjs`.
- A suíte inclui uma falha SQL induzida: nenhum efeito chega ao mundo ativo antes do commit; a tentativa seguinte conclui o mesmo evento.
- TypeScript, ESLint, Stylelint, validação do pacote e build web passaram.
- O ensaio de **10.000 buscas e 10.000 aberturas** passou alternando relógio e acesso narrativo. Usa 17 consultas repetidas. Busca P50/P95/P99: **0,136 / 0,967 / 1,818 ms**; resolução de página: **2,835 / 5,726 / 8,263 ms**. Cache: 64 entradas; histórico: 200 entradas; deltas web: 27.108 bytes. São medições do núcleo em debug, sem IPC, SQLite ou renderização.

As prévias em `/artifacts/web-review/index.html` incluem `offer-base`, `offer-live`, `offer-low`, `offer-empty` e `offer-restock`, exportadas pelo roteador Rust e renderizadas com os componentes de produção. São cenas estáticas, sem IPC. O histórico expansível usa o controle nativo do navegador. As cinco fases foram conferidas em 1024×640, 1280×720, 1366×768, 1920×1080 e 2560×1440. Nas quatro fases com histórico, as vinte combinações de fase e resolução também passaram com o controle expandido, sem transbordamento horizontal. Essa revisão corrigiu o dimensionamento da imagem do produto quando o histórico aumenta a altura do conteúdo.

Para regenerar:

```powershell
node scripts/build-web-content.mjs --check
node --test scripts/web-schema.test.mjs
$env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
$env:RUSTUP_HOME = Join-Path (Get-Location) '.tools/rustup'
& ./.tools/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
& ./.tools/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml virtual_web -- --ignored
node scripts/review-web.mjs
```

## Limites

Esta etapa cobre eventos de publicação, retirada, preço e estoque; não completa toda a Fase D da especificação. SocialGraph, anúncios, arquivo de páginas removidas, pedidos, simulação quantitativa de vendas e expansão editorial permanecem pendentes. O histórico de ofertas não é um arquivo de páginas. Os ensaios ainda não substituem testes de várias horas no aplicativo desktop nem medições em hardware modesto.

O Clippy estrito mantém ocorrências anteriores em `domains.rs`, `packages.rs`, `procfs.rs` e `terminal.rs`; não apontou ocorrências nos módulos desta etapa. O build web conserva os avisos de Zod e tamanho do bundle. Nenhum instalador desktop foi gerado.
