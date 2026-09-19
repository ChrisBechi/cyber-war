# Internet virtual — expansão e evidências

Checkpoint de 18/09/2026. Este relatório descreve o pacote `web-core@3` e os ensaios executados nesta máquina Windows. Ensaio prolongado, outros perfis físicos, sessão completa no desktop e julgamento humano de relevância continuam abertos.

## Conteúdo entregue

- 210 marcas, 1.608 documentos, 257 entidades, 4.742 links validados; JSON compacto de 2.054.508 bytes.
- 630 páginas autoradas não iniciais, 768 variantes materializadas e 210 homes. As 192 marcas de nicho têm duas páginas próprias cada e compartilham renderizadores. Combinações distintas de tipografia, cabeçalho, cartões, densidade e superfície são verificadas automaticamente; isso não prova qualidade editorial ou singularidade visual por si só.
- LinkUp, FeiraLivre, anúncios, arquivo, preço/popularidade, eventos, materialização limitada e ferramentas de desenvolvimento integrados. Guia em [VIRTUAL-INTERNET.md](VIRTUAL-INTERNET.md).
- Auditoria sem corpos ou comentários exatamente duplicados. Similaridade semântica e naturalidade ainda exigem leitura humana.

| Etapa | Marcas | Documentos | Links |
| ----- | -----: | ---------: | ----: |
| 1     |     50 |      1.128 | 3.462 |
| 2     |    100 |      1.278 | 3.862 |
| 3     |    150 |      1.428 | 4.262 |
| 4     |    210 |      1.608 | 4.742 |

## Testes e inspeção

Passaram **190 testes React**, **139 testes Rust** e **15 testes do pipeline de conteúdo**. Após as últimas correções de visibilidade e curtidas, passaram novamente os seis testes Rust de comunidade e os 13 testes React da internet virtual. Clippy estrito, TypeScript, ESLint, Stylelint, geração determinística e validação do conteúdo passaram.

Seis ensaios Rust explícitos passaram: carga de busca/navegação, exportação visual, corpus de recuperação, entradas adversariais, escala do índice e crescimento do arquivo. São testes ignorados por padrão devido ao custo. A exportação da fixture desktop é um sétimo teste ignorado e não deve ser repetida sobre um banco existente.

Persistência: testes transacionais fecham e reabrem um arquivo SQLite real nos cinco slots e verificam publicações, mensagens, seguidores, anúncios e negociações independentes. Uma falha injetada verifica rollback. Isso testa o serviço e o banco, sem validar cliques reais no WebView.

Revisão visual: 220 cenas geradas com os componentes de produção. Nesta expansão, **199 cenas** (sete cenas novas e 192 homes) foram verificadas na matriz de **1024×640, 1280×720, 1366×768, 1920×1080 e 2560×1440**, totalizando 995 combinações sem transbordamento horizontal nem ausência de título. Capturas por amostragem foram inspecionadas para social, classificados, arquivo e identidade editorial. A revisão encontrou e corrigiu a duplicação do botão de curtir no LinkUp. São páginas estáticas; formulários e persistência ficam cobertos pelos testes de componentes/núcleo, não por essas capturas. Resumo em `artifacts/web-audit/visual.json`.

O teste nativo chegou ao menu e à confirmação de saída, mas não concluiu um percurso de jogo e reabertura. Uma nova tentativa iniciou o processo sem expor uma janela à automação; esse processo de QA foi encerrado. Não há evidência suficiente para classificar o percurso como aprovado nem para atribuir o encerramento anterior a um crash. O banco isolado de QA fica em `artifacts/desktop-web-qa`, sem substituir saves do jogador.

## Carga no núcleo

Execução em **debug nesta máquina**, sem IPC, SQLite ou renderização. A carga repete 17 consultas, alternando relógio e acesso narrativo; não são 100 mil intenções diferentes.

| Operação                     | Execuções |      P50 |      P95 |       P99 |
| ---------------------------- | --------: | -------: | -------: | --------: |
| Busca                        |   100.000 | 0,198 ms | 2,648 ms |  5,543 ms |
| Resolução de página          |    10.000 | 1,063 ms | 9,538 ms | 14,565 ms |
| Entrada adversarial distinta |    10.000 | 2,643 ms | 6,292 ms |  9,076 ms |

O cache de busca e o de materialização terminaram com 64 entradas cada; histórico com 200; deltas web com 27.974 bytes. Os limites foram respeitados. O fuzz não encontrou panic, URL duplicada ou vazamento de conteúdo bloqueado nos casos exercitados. Esses resultados não medem vazamentos de memória após horas.

## Relevância e consultas distintas

O corpus mecânico possui **10.000 consultas distintas**, extraídas de passagens de 1.182 documentos, com IDs de origem rastreáveis.

| Medida                                   |                 Resultado |
| ---------------------------------------- | ------------------------: |
| Precision@1                              |                    0,8638 |
| Precision@3                              |                    0,3856 |
| Precision@10                             |                    0,1613 |
| Documento esperado entre os 20 primeiros |                    99,10% |
| Consultas sem resultado                  |                57 (0,57%) |
| P50 / P95 / P99                          | 2,913 / 9,382 / 23,188 ms |

Os julgamentos são incompletos: contabilizam o documento de origem e algumas origens compartilhadas, podendo desconsiderar outros resultados relevantes. Passagens curtas podem conter apenas palavras funcionais. Por isso Precision@K não deve ser apresentada como avaliação humana de qualidade nem usada para afirmar que 99,10% das pesquisas livres serão atendidas. O corpus autorado menor de 29 consultas também passou; não substitui a coleta de intenções reais.

## Escala e armazenamento

Índice sintético, separado do catálogo do jogo, com 25 mil consultas positivas e 25 mil negativas por tamanho. Os tempos abaixo medem consulta ao índice, excluindo ranking completo, correção, montagem de página e persistência.

| Documentos | Construção do índice | Consulta P99 | Bytes serializados |
| ---------- | -------------------: | -----------: | -----------------: |
| 10.000     |               506 ms |         3 µs |          4.320.041 |
| 50.000     |             3.373 ms |         7 µs |         21.866.841 |
| 100.000    |             7.969 ms |         8 µs |         43.800.341 |
| 250.000    |            26.556 ms |        18 µs |        110.500.841 |

A construção a frio de 250 mil registros continua relevante e não pode ser confundida com latência de consulta. Não há um catálogo editorial de 250 mil páginas nem garantia de desempenho em hardware modesto.

O ensaio de armazenamento escreveu conteúdo sintético JSONL em 10k/50k/100k/250k/500k registros. O maior arquivo teve **97.666.685 bytes**, escrito em 6.779 ms. Ele mede crescimento do pacote, não um banco SQLite com 500 mil páginas: o conteúdo base não fica no banco dos saves. O crescimento prolongado dos deltas no desktop continua pendente.

## Reproduzir as auditorias

```powershell
node scripts/build-web-query-corpus.mjs
node scripts/audit-web-content.mjs
node scripts/validate-web-expansion.mjs
$env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
$env:RUSTUP_HOME = Join-Path (Get-Location) '.tools/rustup'
& ./.tools/cargo/bin/cargo.exe test --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture --skip export_desktop_web_qa
node scripts/review-web.mjs
```

Saídas locais ignoradas pelo Git: `artifacts/web-audit/{content,expansion,retrieval,fuzz,scale,storage}.json`, corpus, logs e `artifacts/web-review/coverage.json`. O relatório JSON do Vitest fica em `artifacts/web-audit/frontend-tests.json` quando usado `--reporter=json --outputFile=artifacts/web-audit/frontend-tests.json`.

Build dos instaladores com Node e toolchain Rust local:

```powershell
$env:CARGO_HOME = Join-Path (Get-Location) '.tools/cargo'
$env:RUSTUP_HOME = Join-Path (Get-Location) '.tools/rustup'
$env:Path = (Join-Path (Get-Location) '.tools/cargo/bin') + ';' + $env:Path
node node_modules/@tauri-apps/cli/tauri.js build --ci --config scripts/tauri-build.config.json
```

Saídas em `src-tauri/target/release/bundle/{nsis,msi}`. A configuração só troca o comando de build frontend; não publica nem instala o aplicativo. Avisos conhecidos do build: anotações `PURE` em Zod e chunk JavaScript grande. A checagem global do Prettier também encontra 23 arquivos anteriores fora desta tarefa em Vigília; os arquivos desta implementação são verificados separadamente.

**Build final aprovado em 18/09/2026**, após a correção do botão de curtir: `Cyber War_0.4.2_x64-setup.exe` e `Cyber War_0.4.2_x64_en-US.msi` gerados. Log, horários, tamanhos e SHA-256 estão em `artifacts/web-audit/desktop-build.log`, `desktop-build-result.json` e `desktop-artifacts.json`. A compilação Rust release levou 5 min 26 s; instalação e upgrade não foram executados.

## Homologação ainda aberta

- Completar no aplicativo nativo o percurso Goggle → perfil → seguir → mensagem → classificado → proposta → reserva, fechar/reabrir e comparar os cinco slots.
- Navegar por pelo menos duas horas e medir processo principal, subprocessos WebView, listeners, conexões SQLite e degradação de renderização. `scripts/measure-web-soak.ps1 -GameProcessId <PID> -Minutes 120` mede RAM, CPU, handles, threads e responsividade do processo principal; criar o script não equivale a executar o ensaio. Subprocessos/listeners exigem coleta adicional.
- Repetir medições em hardware físico modesto e intermediário, registrando CPU, memória e armazenamento. As medições atuais são de um único host.
- Sessões humanas de 30 minutos de pesquisa livre e de tentativa de encontrar inconsistências, com consultas, resultados, links, repetição e naturalidade registrados. Revisar a profundidade dos pequenos publicadores e criar julgamentos completos para relevância.
- Instalação/upgrade dos pacotes em ambiente limpo. Gerar instaladores não verifica instalação.

MP4 em ViewTube, pedidos com pagamento e conteúdos específicos das futuras campanhas permanecem condicionados ao gameplay, conforme a especificação. A implementação atual exibe metadados de vídeo, carrinho e negociação local sem pagamento.
