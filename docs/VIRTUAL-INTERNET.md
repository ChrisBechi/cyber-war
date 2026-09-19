# Internet virtual — implementação e validação

Estado em 18/09/2026, baseado em [specs/10-virtual-internet.md](specs/10-virtual-internet.md). A internet local contém **210 marcas, 1.608 documentos, 257 entidades e 4.742 links validados**. A expansão funcional e os ensaios automatizados estão implementados; a homologação humana, o ensaio prolongado e a avaliação em outras máquinas permanecem pendentes. Evidências em [VIRTUAL-INTERNET-EXPANSION.md](VIRTUAL-INTERNET-EXPANSION.md); acompanhamento em [VIRTUAL-INTERNET-PLAN.md](VIRTUAL-INTERNET-PLAN.md).

## Experimentar no jogo

Abra `goggle.com` no navegador interno. Pesquise `pato de borracha`, `curso javascript`, `receita pizza`, `Nexora`, `wifi`, `Nara Campos` ou `pato de boraxa`. O cabeçalho mantém **Mail** e as contas usam **@goggle.com**.

| Marca          | Endereço virtual    | Experiência                                                   |
| -------------- | ------------------- | ------------------------------------------------------------- |
| TechByte       | `techbyte.com`      | Análises, guias, referências e comentários                    |
| Redditor       | `redditor.com`      | Comunidades, tópicos, votos e respostas                       |
| ShopNow        | `shopnow.com`       | Produtos, estoque, avaliações, vendedor e carrinho            |
| Wipédia        | `wipedia.org`       | Artigos, seções, fichas e referências                         |
| Cozinha Fácil  | `cozinhafacil.com`  | Ingredientes marcáveis, preparo e porções                     |
| Educa+         | `educamais.com`     | Cursos, professora, aulas e progresso salvo                   |
| Nexora         | `nexora.com`        | Empresa, produtos, documentação e suporte                     |
| ViewTube       | `viewtube.com`      | Canal, capítulos, descrição e curtidas                        |
| Caderno da Lia | `cadernodalia.blog` | Notas pessoais e conversa                                     |
| B1             | `b1.tech`           | Notícias, editorias e publicações da campanha                 |
| LinkUp         | `linkup.com`        | Perfis, conexões, feed, publicações, notificações e mensagens |
| FeiraLivre     | `feiralivre.com`    | Classificados, propostas, reserva e retirada                  |
| Memória Web    | `memoria.web`       | Capturas de páginas retiradas e endereço original             |

Outros cinco veículos editoriais e 192 pequenos publicadores ampliam assuntos como viagens, ciência, arte, esportes, animais, reparos e cultura local. Os pequenos publicadores têm uma home e duas páginas próprias; não representam 192 mecanismos de plataforma diferentes. Identidade, autoria, temas e referências ficam nos dados do pacote.

Percursos: produto → avaliações → vendedor; curso → professora → aula; Nexora → NexPhone X2 → manual/análise/ShopNow; LinkUp → perfil → empregador; FeiraLivre → vendedor → LinkUp; página retirada → captura no Memória Web. Todos os destinos são resolvidos localmente.

## Conteúdo e arquitetura

`src-tauri/src/virtual_web/` contém o repositório imutável compartilhado, contratos, rotas, categorias, recomendações, ações, arquivo e projeção de eventos. `Platform` e `Detail` determinam a experiência. Blocos controlados renderizam texto, listas, tabelas, código e links, sem executar HTML ou scripts do pacote. Ilustrações SVG são locais.

`scripts/web-content.mjs` gera `content/web/web-core.json` offline. O pacote **web-core@3** ocupa **2.054.508 bytes** em JSON compacto. São **630 páginas não iniciais com conteúdo autorado, 768 descritores de variantes de produtos e 210 homes**. O número de documentos não equivale a 1.608 artigos independentes. Os descritores materializam seus blocos deterministicamente ao abrir a página, em cache de até 64 entradas. O conteúdo base nunca é copiado para o save.

O Goggle usa índice invertido com interseção de listas ordenadas, ranking por correspondência/intenção/autoridade/popularidade, aliases, palavras funcionais e correção aproximada restrita ao vocabulário curado. Oferece Todos, Imagens, Vídeos, Notícias e Compras. Não há busca semântica irrestrita nem acesso à internet real.

O cache da busca conserva até 64 respostas em IDs, fora do save. As chaves consideram consulta, modo, página e estado narrativo/temporal. A interface encerra a espera após cinco segundos; o núcleo também verifica o prazo antes de devolver a resposta. Isso não cancela preemptivamente uma tarefa Rust. O índice compartilhado é aquecido em uma thread na inicialização.

`WorldState.web` persiste somente deltas: versões, histórico, carrinho, curtidas, comentários, aulas, eventos, seguidores, publicações pessoais, mensagens, notificações lidas, negociação e cliques pagos. Limites incluem histórico 200, carrinho 100 produtos/9 unidades, 500 comentários pessoais totais, 500 perfis seguidos, 100 publicações, 200 mensagens, 1.000 notificações lidas e 100 negociações. Saves antigos recebem defaults; versões futuras incompatíveis são recusadas.

## Plataformas e mundo conectado

LinkUp possui seis personas autoradas com empregadores, relações e publicações. Seguir alguém alimenta o feed e as notificações. Publicações pessoais e conversas ficam nesta campanha; respostas de NPCs são roteirizadas. Curtidas e comentários usam as ações persistentes do mundo. Ocultar um perfil também oculta suas publicações, feed, mensagens e interações.

FeiraLivre possui seis ofertas com condição, localização, vendedor e preço. Propostas abaixo do mínimo são recusadas; uma proposta aceita fica salva. O fluxo disponível → reservado → vendido e a liberação de reserva são validados no backend. Filtros e cartões acompanham o estado. Confirmar retirada não movimenta saldo bancário.

A rede de anúncios considera palavras-chave, superfície, orçamento, período, qualidade e risco configurados na campanha. Duas campanhas demonstram anúncios identificados como **Patrocinado**, com contagem de cliques persistente. Elegibilidade é recalculada mesmo quando os resultados orgânicos vêm do cache. Páginas ocultas não podem ser promovidas. Os campos de risco preparam futuros conteúdos de gameplay; não constituem uma simulação completa de fraude.

Memória Web disponibiliza capturas somente após a retirada de uma página. O conteúdo usa o relógio anterior à remoção, indica a data e oferece o endereço original. Capturas são somente leitura e continuam respeitando flags e bloqueios narrativos.

Os eventos do MissionEngine e do relógio atualizam publicações, estoque, preço e anúncios. `ORION_ACCESS_REVIEW` e a campanha NexPhone X2 demonstram mudanças coordenadas entre sites, Goggle e carrinho. Rollback revoga o conteúdo correspondente. A cronologia está em [VIRTUAL-INTERNET-EVENTS.md](VIRTUAL-INTERNET-EVENTS.md).

O modelo de preços valida faixas das famílias cadastradas e seus eventos. Avaliações visíveis determinam a nota exibida; popularidade deriva de contagens autoradas em faixas limitadas. Isso fornece coerência local, sem modelar uma economia completa.

Aliases como `educa.com` e `nexora.tech` seguem para URLs canônicas. Rotas ausentes exibem 404 da marca; domínios desconhecidos não resolvem. Nenhuma rota possui fallback para HTTP, DNS ou arquivos reais. Domínios das marcas são reservados no mercado virtual.

## Ferramentas e validação

Em desenvolvimento, o explorador permite filtrar marcas, documentos, entidades, eventos e componentes do ranking, além de observar cache, seed, pacote, visibilidade e tempo de resolução. O comando de diagnóstico é indisponível em release. Os relatórios verificam links, duplicações exatas, datas, famílias de preço e identidade das marcas.

```powershell
node scripts/build-web-content.mjs --check
node scripts/validate-content.mjs
node --test scripts/web-schema.test.mjs
node scripts/audit-web-content.mjs
node scripts/validate-web-expansion.mjs
node node_modules/vitest/vitest.mjs run --maxWorkers=2 --minWorkers=1
node node_modules/typescript/bin/tsc -b --pretty false
node node_modules/eslint/bin/eslint.js . --max-warnings=0
node node_modules/stylelint/bin/stylelint.mjs 'src/**/*.{css,scss}'
$env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
$env:RUSTUP_HOME = Join-Path (Get-Location) '.tools/rustup'
& ./.tools/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml
& ./.tools/cargo/bin/cargo.exe clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

O Rust testa rotas, aliases, 404, visibilidade, rollback, cache, ações e cinco slots SQLite, incluindo fechamento e reabertura do banco. O React testa comportamento da interface com IPC simulado. Esses testes não substituem um fluxo completo no WebView nativo.

Para revisão visual dos componentes de produção com DTOs reais do Rust:

```powershell
& ./.tools/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml export_web_review -- --ignored
node scripts/review-web.mjs
node node_modules/vite/bin/vite.js --host 127.0.0.1 --port 1420
```

Abra `/artifacts/web-review/index.html`: são 220 cenas estáticas, com controles internos sem IPC. Resultados de desempenho, relevância e revisão visual, instruções de build e pendências constam no [relatório de expansão](VIRTUAL-INTERNET-EXPANSION.md).
