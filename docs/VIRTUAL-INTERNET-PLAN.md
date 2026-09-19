# Internet virtual — plano de conclusão

Autorização: implementar todas as pendências, em 18/09/2026. Este documento acompanha o trabalho; uma caixa só é concluída com implementação e evidência, não apenas com um plano ou ferramenta criada.

Evidências e limitações de cada etapa: [VIRTUAL-INTERNET-EXPANSION.md](VIRTUAL-INTERNET-EXPANSION.md). Implementação funcional concluída; homologação da especificação ainda parcial.

## 1. Plataformas e mundo conectado

- [x] LinkUp: perfis, vínculos profissionais, seguidores, feed, publicações, comentários, notificações e mensagens locais persistentes.
- [x] Classificados: vendedor, condição, localização, preço, negociação, reserva e venda; estados isolados por save.
- [x] Publicidade: campanhas locais, segmentação, orçamento e resultados identificados como patrocinados, respeitando visibilidade narrativa.
- [x] Arquivo: snapshots de páginas retiradas, data da captura e navegação para a versão corrente, sem revelar conteúdo futuro.
- [x] Testes de integração no serviço/SQLite e revisão visual estática das novas plataformas; percurso nativo permanece na etapa 4.

## 2. Conteúdo e coerência

- [x] Expandir assuntos, entidades, personas e textos independentes; reduzir a predominância de variantes de produtos. São 630 páginas autoradas não iniciais e 768 variantes.
- [x] Modelos coerentes de preço, popularidade, avaliações e relações entre sites.
- [x] Materialização determinística sob demanda, com cache limitado e compatibilidade de saves.
- [x] Ampliar marcas progressivamente (50, 100, 150, 210), validando conteúdo e identidade em cada etapa. Os 192 publicadores de nicho têm duas páginas próprias e compartilham renderizadores; aliases não entram na contagem.
- [x] Cobertura de domínios de nicho conforme resultados da auditoria, com profundidade e contagens discriminadas no relatório.

## 3. Ferramentas de desenvolvimento

- [x] Explorador de marcas, documentos, entidades e eventos.
- [x] Diagnóstico da busca com componentes do score, visibilidade e cache, apenas em desenvolvimento.
- [x] Relatórios de duplicação exata, links intencionais, cobertura e consistência temporal.

## 4. Critérios de entrega

- [x] Corpus mecânico de 10.000 consultas distintas, com documentos de origem rastreáveis; Precision@1/3/10, ausência de resultados e latência registradas. Julgamentos humanos completos permanecem pendentes.
- [x] Fuzzing de 10.000 entradas distintas e carga de 100.000 buscas/10.000 navegações; cinco saves e limites de materialização testados.
- [x] Benchmarks sintéticos do índice em 10k, 50k, 100k e 250k documentos; crescimento de arquivo JSONL até 500k. Não são benchmarks completos do desktop nem SQLite com 500k páginas.
- [ ] Sessões prolongadas no desktop, medindo memória, handles, listeners e degradação.
- [ ] Perfis de hardware: distinguir medições reais de simulação/restrição de recursos.
- [ ] Fluxos completos no desktop, fechar/reabrir e isolamento dos cinco slots.
- [ ] Revisão visual e editorial, exploração livre e busca de inconsistências por testadores humanos.
- [x] Build desktop release: EXE e MSI gerados com sucesso após as correções finais; resultados e limites registrados. Instalação/upgrade em ambiente limpo ainda não ensaiados.

As etapas 1–3 permitem execução local autônoma. Avaliação humana e medição em máquinas físicas diferentes exigem participantes/hardware; preparar os instrumentos não substitui essas evidências. Pedidos financeiros e MP4 não são requisitos universais da especificação: entram quando o conteúdo ou gameplay exigir.
