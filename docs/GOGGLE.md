# Goggle — descoberta da internet virtual

Abra `goggle.com`, `www.goggle.com` ou `https://www.goggle.com` no navegador do jogo. O favorito Goggle foi incluído nas preferências iniciais; favoritos já salvos são preservados.

## Experiência

- Página clara com wordmark próprio, campo arredondado, sugestões, teclado QWERTY, voz simulada e busca por imagem.
- Enter e Pesquisar Goggle abrem `/search?q=...`, preservando acentos e maiúsculas. Estou com sorte abre o primeiro resultado do mesmo ranking; sem resultados, abre a pesquisa vazia.
- Abas Todos, Imagens, Vídeos, Notícias e Compras, paginação de 20 documentos, resultados vazios e falhas explícitas.
- Criar conta/Fazer login usam **@goggle.com** e senha fictícia. O cabeçalho autenticado mostra **Mail**, Imagens, nove pontos e avatar. O serviço se chama **Goggle Mail**.
- Images e Account estão implementados. Mail, Drive, Maps, News, Calendar, Photos e Meet exibem **Em breve**.
- Sobre, Como funciona a Pesquisa, Privacidade e Termos são documentos do catálogo e podem ser substituídos por eventos narrativos.
- O histórico começa desativado. Pode ser ativado e limpo em Gerenciar sua conta ou pelo link na política de privacidade. É uma preferência da campanha.

## Autoridade e persistência

`src-tauri/src/search/` é o serviço central `VirtualSearchEngine`. React consulta comandos IPC e mantém somente estado temporário de interação. A implementação não depende da prévia web.

`index.rs` constrói uma vez o índice invertido dos documentos em `content/search/documents.json`. As buscas intersectam tokens normalizados, desconsiderando caixa e acentos. Título, palavras-chave, descrição, autoridade, popularidade, missão ativa e modificadores determinam o ranking. Consultas pelo nome de uma organização favorecem sua página principal. O desempate usa o ID do documento e não depende de relógio ou aleatoriedade.

`WorldState.search`, com `serde(default)`, guarda somente documentos alterados/adicionados/removidos, modificadores, sugestões narrativas, associações adicionais de imagem, contas, sessão e preferências/histórico. O catálogo global não é duplicado no save. O campo ausente em saves anteriores assume o estado inicial. Mutações de conta e preferências usam as transações/autosaves existentes de `GameService`.

O DTO de sessão não expõe a senha virtual. A credencial persistida é um digest com identificador da conta, suficiente para a simulação local; não é um provedor de identidade real.

## Publicação e missões

Sites podem acrescentar documentos ao catálogo global ou aplicar `SearchEffect::AddDocument` no núcleo. Cada documento tem ID, URL virtual, domínio, título, descrição, conteúdo, palavras-chave, tipo, autoridade, popularidade, tags de missão e visibilidade. `requiredFlags` e `requiredMissions` filtram publicações e sugestões antes de qualquer ranking. As rotas adicionais cadastradas são renderizadas pelo navegador virtual.

O formato das missões existentes recebe um efeito `search` com uma destas operações:

```json
{
  "kind": "search",
  "effect": {
    "kind": "CHANGE_SEARCH_VISIBILITY",
    "id": "documento-da-investigacao",
    "visibility": "PUBLIC"
  }
}
```

| Operação                   | Campos                                                       |
| -------------------------- | ------------------------------------------------------------ |
| `ADD_SEARCH_DOCUMENT`      | `document`: SearchDocument completo                          |
| `REMOVE_SEARCH_DOCUMENT`   | `id`                                                         |
| `CHANGE_SEARCH_VISIBILITY` | `id`, `visibility`: `PUBLIC` ou `HIDDEN`                     |
| `ADD_SEARCH_SUGGESTION`    | `id`, `suggestion`: `{ "text": "...", "requiredFlags": [] }` |
| `CHANGE_SEARCH_RANKING`    | `id`, `boost`: inteiro entre −100000 e 100000                |

Os efeitos participam do diário de recursos das missões. Descartar uma tentativa desfaz somente os documentos, sugestões e pesos pertencentes àquela tentativa; contas e preferências pessoais permanecem. Efeitos de conclusão são duráveis.

O catálogo inicial indexa Wipédia, Archive, B1, Mercado, MeuDomínio, SECTOR IX, Orion e páginas institucionais Goggle. FakeBook exige a missão `girl` ativa ou concluída. `NULL` não tem resultados inicialmente: após `SESSION_1_COMPLETE`, aparecem Archive e Blackwire. O noticiário sobre o vazamento Orion usa o mesmo sinal narrativo.

## Extensão da internet virtual

A [internet virtual](VIRTUAL-INTERNET.md) integra 210 marcas e 1.608 documentos ao mesmo índice, com correção de consultas, intenções, cache limitado, abas de vídeos/notícias/compras, anúncios identificados e imagens locais. Os resultados abrem renderizadores de plataforma e navegação interna. A seção de revalidação abaixo registra o checkpoint anterior do Goggle; os resultados atuais, incluindo Clippy e build, estão no [relatório de expansão](VIRTUAL-INTERNET-EXPANSION.md).

## Imagens e isolamento

`images.rs` mantém as associações entre imagens, documentos e entidades como `company:orion` e `place:orion-campus`. `WorldState.search.images` permite acrescentar ou substituir registros narrativos. Os IDs de entidades também podem representar pessoas, publicações e artigos.

A busca reversa aceita uma URL registrada nesse índice ou um arquivo legível no VFS local. Compara o hash dos bytes da imagem, por isso uma cópia renomeada mantém a associação. Arquivos binários usam os hashes do sistema de blobs existente; SVGs narrativos textuais usam SHA-256 do conteúdo. Não há reconhecimento visual: imagens desconhecidas retornam zero associações.

As prévias são SVGs próprios em `public/assets/goggle/`, selecionados por IDs permitidos na interface. Endereços virtuais nunca são usados como `src` ou `href` remoto. A busca não usa HTTP, DNS real, OAuth, filesystem do host, seletor de arquivos do Windows ou permissão de microfone. O formulário de imagem lista exclusivamente o VFS. A simulação respeita o estado de conexão da rede virtual.

Para uma missão disponibilizar uma imagem de referência, grave os bytes do asset correspondente no VFS e registre suas associações no índice de imagens. As duas referências iniciais são:

- `https://www.orion.com/media/campus.svg`
- `https://www.archive.org/media/archive.svg`

## Extensão da omnibox

`goggle-model.ts` exporta `DefaultSearchProvider` e `goggleProvider`. O contrato produz URLs de busca sem interpretar texto da omnibox como pesquisa automaticamente nesta versão.

## Verificação

Os testes Rust cobrem ranking, 5000 documentos no índice invertido, paginação, restrições narrativas, cinco modificadores, integração com o diário de missões, imagens textuais e binárias do VFS, rejeição de caminhos/URLs externos, contas, histórico, SQLite e migração de saves.

Os testes React cobrem home, autenticação, logout, Enter/botões, sugestões por teclado e mouse, respostas obsoletas, teclado virtual, voz simulada, imagens do VFS, prévias locais e navegação real entre os componentes do navegador. Os testes de UI substituem o IPC; a lógica de busca e persistência é exercitada separadamente nos testes do núcleo Rust.

`node scripts/review-goggle.mjs` gera páginas descartáveis em `artifacts/goggle-review/` usando os componentes React e CSS reais, incluindo variantes `embedded-*` dentro da estrutura do navegador. São páginas estáticas para revisão visual, sem transporte alternativo de gameplay. Sirva pelo Vite e confira 1024×640, 1366×768 e 1920×1080. Não é necessário publicar nada.

A página pública do Google foi consultada somente como referência visual de espaçamento e organização. Nenhum asset, SVG, fonte remota ou código do Google integra o Goggle.

### Resultado da revalidação — 17/09/2026

- Suíte React completa: **177 testes passaram**; 14 deles cobrem Goggle diretamente.
- Suíte Rust completa: **113 testes passaram**. Os 9 testes de busca foram repetidos após os ajustes finais do índice e do catálogo.
- TypeScript, ESLint, Stylelint, validação de conteúdo, formatação dos arquivos da entrega e build Vite passaram.
- Home dentro da estrutura do navegador revisada em 1024×640, 1366×768 e 1920×1080: rodapé integralmente visível e nenhuma rolagem horizontal. Capturas em `artifacts/goggle-review/`.
- O Clippy com `-D warnings` continua bloqueado por ocorrências preexistentes em `domains.rs`, `packages.rs`, `procfs.rs` e `terminal.rs`. O build Vite também mantém os avisos das dependências Zod e do tamanho do bundle. Não foram gerados novos instaladores desktop nesta entrega.
