# CYBER WAR — SPEC
# VIRTUAL INTERNET / MINI INTERNET SIMULADA

Status: Planejamento para implementação
Prioridade: Alta
Escopo: Internet inteiramente fictícia e local do CYBER WAR

---

# 1. VISÃO

Construir uma mini internet totalmente fictícia dentro do CYBER WAR.

Ela deve transmitir ao jogador a sensação de estar utilizando uma internet
real, apesar de todo o conteúdo pertencer ao universo do jogo.

Não haverá WebView para internet real nesta implementação.

Não haverá navegação para sites reais.

Não haverá dependência de serviços externos para o funcionamento da campanha.

A internet deve possuir:

- mecanismos de busca;
- portais;
- notícias;
- blogs;
- redes sociais;
- fóruns;
- vídeos;
- lojas;
- marketplaces;
- classificados;
- receitas;
- educação;
- documentação;
- empresas;
- bancos;
- fintechs;
- mapas;
- serviços públicos;
- entretenimento;
- tecnologia;
- games;
- viagens;
- empregos;
- imóveis;
- automóveis;
- ciência;
- curiosidades;
- sites pessoais;
- comunidades underground;
- sites relacionados às missões.

A internet deve ser:

1. navegável;
2. pesquisável;
3. coerente;
4. persistente;
5. dinâmica;
6. integrada à história;
7. leve;
8. determinística;
9. extensível;
10. funcional offline.

---

# 2. PRINCÍPIO FUNDAMENTAL

Não criar:

"centenas de páginas React independentes."

Criar:

"uma plataforma capaz de representar centenas de sites."

Arquitetura:

VirtualWebWorld
│
├── DomainRegistry
├── BrandRegistry
├── PlatformRegistry
├── ContentRepository
├── EntityGraph
├── KnowledgeBase
├── SearchIndex
├── SearchEngine
├── VirtualRouter
├── AssetRegistry
├── ContentGenerator
├── RecommendationEngine
├── AdNetwork
├── SocialGraph
├── WorldEventBus
├── WebHistory
├── Cache
└── Persistence

Os sites são manifestações desse mundo.

---

# 3. META DE ESCALA

Meta inicial:

~200–250 marcas principais navegáveis.

Meta secundária:

~1.000–3.000 domínios long-tail.

Meta de índice:

~50.000–150.000 SearchDocuments.

Esses números NÃO são requisitos rígidos para o primeiro commit.

Devem ser alcançados progressivamente após testes de cobertura.

A qualidade e diversidade são mais importantes que atingir um número
arbitrário.

---

# 4. CATEGORIAS INICIAIS

Planejar aproximadamente:

Notícias / portais .............. 12
Tecnologia ...................... 12
Games ........................... 10
Redes sociais ................... 6
Fóruns / comunidades ............ 8
E-commerce ...................... 10
Classificados ................... 6
Receitas / culinária ............ 8
Educação ........................ 10
Cinema / séries ................. 6
Música / entretenimento ......... 5
Vídeos .......................... 4
Automóveis ...................... 8
Imóveis / casa .................. 6
Viagens ......................... 7
Finanças ........................ 8
Bancos / fintechs ............... 8
Empregos / carreira ............. 6
Empresas / corporativos ......... 15+
Desenvolvimento / docs .......... 8
Enciclopédia / conhecimento ..... 5
Ciência / curiosidades .......... 6
Saúde / bem-estar ............... 5
Governo / serviços .............. 6
Mapas / localização ............. 2
Cloud / e-mail / produtividade .. 6
Segurança ....................... 7
Underground ..................... 10+
Blogs pessoais .................. 15+
Sites especializados ............ 10+

A distribuição deve ser ajustada conforme testes reais de busca.

---

# 5. IDENTIDADE DAS MARCAS

REGRA:

Duas marcas não podem parecer simplesmente:

"mesmo site com outra logo."

Cada BrandDefinition deve possuir identidade.

Exemplo:

BrandDefinition {
    id
    name
    domain
    category
    platform
    description

    identity {
        logo
        favicon
        typography
        layoutFamily
        headerVariant
        navigationVariant
        cardVariant
        contentDensity
        borderStyle
        spacingProfile
        imageTreatment
        sidebarStyle
        footerStyle
    }

    editorial {
        tone
        audience
        topics
        headlineStyle
        articleLength
        credibility
        sensationalism
        politicalProfile?
        technicalDepth
    }

    behavior {
        comments
        accounts
        search
        categories
        recommendations
        ads
        trending
        pagination
    }
}

---

# 6. DESIGN SYSTEM POR PLATAFORMA

Criar engines reutilizáveis.

Exemplos:

SearchEnginePlatform
EditorialPlatform
BlogPlatform
SocialPlatform
ForumPlatform
CommercePlatform
ClassifiedPlatform
VideoPlatform
RecipePlatform
EducationPlatform
CorporatePlatform
BankingPlatform
GovernmentPlatform
DocumentationPlatform
EncyclopediaPlatform
TravelPlatform
JobsPlatform
MapsPlatform
CloudPlatform

Cada plataforma deve possuir várias famílias de layout.

Exemplo:

EditorialPlatform:

NEWSPAPER
MAGAZINE
TECH
TABLOID
LOCAL
MINIMAL
BUSINESS
ENTERTAINMENT

Isso permite compartilhar infraestrutura sem produzir clones visuais.

---

# 7. NÍVEIS DE SITE

Criar:

SiteDepth

FULL
STANDARD
LONG_TAIL

FULL:

- navegação profunda;
- múltiplas áreas;
- pesquisa própria;
- contas quando aplicável;
- páginas;
- interações;
- comentários;
- histórico;
- conteúdo relevante para gameplay.

STANDARD:

- home;
- categorias;
- páginas individuais;
- navegação;
- identidade completa;
- menos sistemas exclusivos.

LONG_TAIL:

- identidade persistente;
- algumas páginas;
- conteúdo procedural;
- navegação limitada, porém real;
- nunca simplesmente uma tela vazia.

---

# 8. REGRA CONTRA "SITE DE PAPELÃO"

Proibido:

resultado Goggle
→ click
→ página contendo apenas título + texto.

Todo resultado navegável deve parecer pertencer a um site.

No mínimo:

- logo;
- domínio;
- header;
- navegação;
- conteúdo;
- links relacionados;
- footer;
- identidade visual;
- contexto do site.

Mesmo LONG_TAIL deve parecer um site real.

---

# 9. VIRTUAL ROUTER

Criar VirtualRouter.

Entrada:

https://techbyte.com/review/nexphone-x2

Resolução:

domain
→ BrandRegistry

route
→ PlatformRouter

resource
→ ContentRepository

renderer
→ PlatformRenderer

Não criar arquivo HTML físico para cada URL.

---

# 10. DOMÍNIOS

DomainRegistry deve controlar:

domain
subdomain
brandId
platformId
status
createdAt
expiresAt?
redirects?
aliases
securityMetadata
missionMetadata

Suportar:

domain.com
blog.domain.com
support.domain.com
store.domain.com

quando necessário.

---

# 11. INTERNET NÃO É ESTÁTICA

Sites podem:

- surgir;
- desaparecer;
- mudar;
- publicar;
- remover páginas;
- alterar preços;
- alterar estoque;
- alterar perfis;
- receber comentários;
- publicar notícias;
- responder acontecimentos.

Criar WorldClock + WorldEventBus.

---

# 12. WORLD ENTITY GRAPH

Criar EntityGraph central.

Tipos:

Person
Company
Product
Software
Game
Device
Location
Organization
Event
Article
Video
Course
Recipe
Vehicle
Property
GovernmentAgency
Bank
Technology
Topic
etc.

Relacionamentos:

WORKS_AT
OWNS
CREATED
MENTIONS
LOCATED_AT
SELLS
BOUGHT
REVIEWED
POSTED
RELATED_TO
MANUFACTURED_BY
EMPLOYED_BY
ATTENDED
RELEASED
etc.

---

# 13. COERÊNCIA ENTRE SITES

Uma mesma entidade deve permanecer consistente.

Exemplo:

NEXPHONE_X2

pode aparecer em:

TechByte
→ review

ShopNow
→ produto

ViewTube
→ vídeo

Redditor
→ discussão

FakeBook
→ posts

Nexora
→ página oficial

Goggle
→ todos os anteriores

Não criar seis "NexPhone X2" independentes.

Todos referenciam a mesma WorldEntity.

---

# 14. KNOWLEDGE BASE

Criar WorldKnowledgeBase.

Objetivo:

permitir pesquisas comuns não relacionadas à campanha.

Categorias:

objetos
animais
alimentos
tecnologia
programação
carros
casa
viagem
educação
entretenimento
ciência
história
cotidiano
etc.

Exemplo:

RUBBER_DUCK

aliases:

pato de borracha
patinho de borracha
rubber duck

relations:

toy
bath
programming
rubber duck debugging

Isso permite que:

"pato de borracha"

produza resultados coerentes.

---

# 15. TRÊS FONTES DE CONHECIMENTO

Separar:

GENERAL_KNOWLEDGE

CYBER_WAR_LORE

CAMPAIGN_STATE

GENERAL_KNOWLEDGE:
conhecimento comum.

CYBER_WAR_LORE:
empresas, personagens, produtos e acontecimentos fictícios.

CAMPAIGN_STATE:
o que aconteceu especificamente naquele save.

SearchEngine consulta os três.

---

# 16. CONTEÚDO ARTESANAL

Conteúdo crítico é HANDCRAFTED.

Inclui:

- missões;
- pistas;
- personagens;
- empresas centrais;
- vazamentos;
- páginas importantes;
- notícias-chave;
- documentos;
- golpes;
- fóruns narrativos;
- eventos;
- informações necessárias para progressão.

Nunca substituir conteúdo narrativo crítico por geração procedural.

---

# 17. CONTEÚDO PROCEDURAL

Usado para ambientação.

Exemplos:

reviews comuns;
receitas;
produtos;
posts;
comentários;
artigos cotidianos;
blogs;
vídeos ambientais;
classificados;
cursos;
perguntas;
respostas.

Deve ser determinístico.

---

# 18. SEEDS

Todo conteúdo procedural persistente recebe seed.

Exemplo:

ContentSeed {
    worldSeed
    siteSeed
    contentSeed
}

Mesma seed:

→ mesmo resultado.

Nunca fazer um blog mudar completamente apenas porque foi reaberto.

---

# 19. GERAÇÃO DURANTE DESENVOLVIMENTO

Preferir geração OFFLINE durante desenvolvimento.

Pipeline:

ContentGenerator
↓
validation
↓
deduplication
↓
consistency checks
↓
content packs
↓
game assets/data

O jogo final não deve depender de LLM online para funcionar.

---

# 20. CONTENT PACKS

Organizar conteúdo em packs.

Exemplo:

core-web.pack
technology.pack
games.pack
food.pack
shopping.pack
education.pack
travel.pack
automotive.pack
entertainment.pack
campaign-session-1.pack
campaign-session-2.pack
signal.pack
ghostmarket.pack
root.pack

Permitir atualizar conteúdo sem reconstruir toda a internet.

---

# 21. GOGGLE

Goggle será o principal mecanismo de descoberta.

Deve possuir:

- home;
- autocomplete;
- correção ortográfica;
- resultados;
- imagens;
- vídeos;
- notícias;
- compras;
- paginação;
- "Estou com sorte";
- histórico;
- sugestões;
- pesquisas relacionadas.

---

# 22. SEARCH DOCUMENT

SearchDocument {

    id
    url
    domain
    title
    description
    bodyTokens
    keywords
    entities
    category
    contentType

    authority
    popularity
    freshness
    relevanceBoost

    visibility
    publishedAt
    removedAt?

    missionRequirements?
    storyRequirements?
}

---

# 23. INDEXAÇÃO

Indexar:

title
headings
description
body
keywords
entities
domain
author
categories
tags

Não indexar conteúdo:

MISSION_LOCKED
PRIVATE
DRAFT
REMOVED

exceto quando regras específicas permitirem.

---

# 24. RANKING

Criar ranking semelhante conceitualmente a mecanismos reais.

Considerar:

text relevance
title match
entity match
authority
popularity
freshness
domain relevance
user intent
content quality
story visibility

Não usar story relevance para entregar respostas descaradamente.

O Goggle NÃO deve parecer:

"máquina de mostrar pista".

---

# 25. INTENÇÃO DA BUSCA

Detectar:

NAVIGATIONAL
INFORMATIONAL
SHOPPING
RECIPE
VIDEO
NEWS
TECHNICAL
LOCAL
EDUCATIONAL
TRANSACTIONAL
GENERAL

Exemplo:

"comprar pato de borracha"

SHOPPING.

"o que é pato de borracha"

INFORMATIONAL.

"rubber duck debugging"

TECHNICAL.

---

# 26. FUZZY SEARCH

Suportar:

typos;
acentos;
plural;
singular;
aliases;
sinônimos;
abreviações.

Exemplo:

pato de boraxa

→

Você quis dizer:
pato de borracha

---

# 27. CONSULTAS ABSURDAS

Não tentar responder perfeitamente tudo.

Se:

asdkjasdhqwe

retornar zero:

mostrar zero resultados.

Se consulta extremamente específica possuir apenas dois documentos:

mostrar dois.

Isso aumenta o realismo.

---

# 28. TEMPO DE BUSCA

META:

cache ................. <100ms
índice ................. <300ms
busca complexa ......... <1s
procedural ............. <2s
pior caso .............. <3s

HARD TIMEOUT:

5 segundos.

Nunca ultrapassar 5 segundos esperando resultados.

---

# 29. PROGRESSIVE RESULTS

Pode apresentar resultados progressivamente.

Mas:

NÃO reorganizar resultados já clicáveis constantemente.

Primeiro estabilizar resultados principais.

Depois acrescentar resultados secundários abaixo.

---

# 30. CACHE

Criar SearchCache.

Guardar:

normalizedQuery
resultIds
timestamp
worldVersion
campaignVersion

Invalidar quando acontecimentos relevantes alterarem o índice.

---

# 31. MATERIALIZAÇÃO SOB DEMANDA

Pesquisa não precisa criar página completa.

Primeiro:

SearchDocument.

Somente no click:

ContentResolver
↓
ContentRepository

Se materializado:
→ carregar.

Se procedural ainda não materializado:
→ materializar deterministicamente.

---

# 32. TEMPO DE ABERTURA

Meta:

site existente ........ <150ms
conteúdo cacheado ...... <100ms
materialização ......... <500ms
pior caso .............. <1s

A página deve começar a renderizar imediatamente.

---

# 33. NUNCA MOSTRAR "GERANDO"

Não mostrar:

Gerando artigo...
IA pensando...
Criando conteúdo...

Para o jogador:

é simplesmente carregamento de página.

---

# 34. NAVEGAÇÃO

Implementar corretamente:

Back
Forward
Reload
Home
Address bar
Tabs
History
Bookmarks

URLs precisam ser reais dentro do mundo virtual.

---

# 35. DEEP LINKS

Se jogador digitar:

techbyte.com/reviews/nexphone-x2

deve abrir diretamente.

Se URL não existir:

404 da própria marca.

Não redirecionar tudo ao Goggle.

---

# 36. 404

Cada plataforma deve possuir 404 compatível com sua identidade.

TechByte:

404 TechByte.

ShopNow:

produto/página não encontrada.

Redditor:

thread inexistente.

Isso aumenta muito a fidelidade.

---

# 37. REDIRECTS

Suportar redirects virtuais.

Exemplo:

oldcompany.com

→

newcompany.com

Também úteis narrativamente.

---

# 38. HISTÓRICO WEB

BrowserHistoryEntry {

    url
    title
    favicon
    timestamp
}

Persistir conforme design de save.

---

# 39. FAVORITOS

Permitir bookmarks.

Folders opcionais.

Persistir por save/perfil conforme arquitetura atual.

---

# 40. SOCIAL ENGINE

Redes sociais devem possuir:

accounts
profiles
posts
comments
likes
followers
following
media
search
notifications
messages quando aplicável

Cada rede deve possuir propósito diferente.

Não criar cinco clones de Instagram.

---

# 41. FORUM ENGINE

Suportar:

communities
boards
threads
posts
replies
votes
users
moderators
timestamps
deleted posts
locked threads
search

Blackwire pode utilizar extensões próprias.

---

# 42. COMMERCE ENGINE

Suportar:

categories
products
brands
search
filters
price
stock
seller
reviews
ratings
cart
orders quando gameplay exigir
recommendations

---

# 43. MARKETPLACE / CLASSIFIEDS

Separar comércio tradicional de pessoa-para-pessoa.

Suportar:

seller profiles
used/new
location
negotiation
messages
listing age
sold
reserved
scam signals

Isso será especialmente útil para GHOSTMARKET.

---

# 44. EDITORIAL ENGINE

Suportar:

homepage
sections
articles
authors
tags
related articles
breaking news
trending
comments
publication dates
corrections
updates

---

# 45. BLOG ENGINE

Blogs precisam parecer pessoais.

Variações:

layout;
autor;
bio;
arquivo;
tags;
frequência;
qualidade;
tom;
fotografia;
sidebar.

Evitar blogs proceduralmente idênticos.

---

# 46. RECIPE ENGINE

Suportar:

ingredients
steps
prep time
difficulty
servings
ratings
comments
related recipes
categories

---

# 47. EDUCATION ENGINE

Suportar:

courses
teachers
lessons
ratings
duration
price/free
progress quando necessário
categories
search

---

# 48. VIDEO ENGINE

Um VideoDocument pode existir sem MP4.

VideoMetadata:

thumbnail
title
channel
views
likes
duration
comments
description
publishedAt

Somente vídeos importantes precisam possuir mídia real.

---

# 49. CORPORATE ENGINE

Empresas:

home
about
products
services
team
careers
investors
contact
press
support

Permitir páginas antigas/removidas.

Muito importante para OSINT.

---

# 50. DOCUMENTATION ENGINE

Documentações técnicas devem suportar:

versions
sidebar
search
code blocks
API references
downloads virtuais
deprecated pages
release notes

Muito importante para ROOT, SIGNAL e BREACH.

---

# 51. WIPÉDIA / ENCICLOPÉDIA

Criar enciclopédia fictícia abrangente.

Artigos:

summary
infobox
sections
references fictícias
related pages
history quando necessário

Deve ser uma das principais fontes de conhecimento geral.

---

# 52. ADS

Criar VirtualAdNetwork.

Anúncios podem aparecer em:

Goggle
portais
blogs
social
vídeo
commerce

Ads possuem:

campaign
targeting
budget
keywords
landingPage
quality
fraudRisk

Isso posteriormente pode ser usado em GHOSTMARKET.

---

# 53. PATROCINADOS NO GOGGLE

Suportar:

Patrocinado

Resultados patrocinados podem aparecer acima dos orgânicos.

Isso permite simular:

SEO
SEM
tráfego pago
phishing
domínios parecidos
campanhas fraudulentas

sem usar serviços reais.

---

# 54. RECOMMENDATION ENGINE

Sites podem recomendar:

artigos;
vídeos;
produtos;
posts;
cursos.

Basear em:

content similarity
popularity
site profile
world state

Não precisa modelar usuário com complexidade extrema inicialmente.

---

# 55. WORLD EVENT BUS

Exemplo:

ORION_ACQUIRES_NEURON

pode gerar efeitos:

B1
→ notícia

TechByte
→ análise

Redditor
→ thread

FakeBook
→ comentários

Goggle
→ indexação

CompanySite
→ press release

Market
→ mudanças relevantes

Tudo referenciando o mesmo WorldEvent.

---

# 56. TEMPORALIDADE

Conteúdo deve respeitar tempo.

Não mostrar:

artigo publicado amanhã

antes de amanhã.

Suportar:

publishedAt
updatedAt
removedAt

baseados no relógio narrativo.

---

# 57. SPOILERS

Criar validação automática de conteúdo bloqueado.

Um SearchDocument não pode revelar:

missão futura;
identidade secreta;
aquisição ainda não conhecida;
morte futura;
informação confidencial;

antes do evento correto.

---

# 58. ARCHIVE

Criar posteriormente ou preparar:

WebArchiveEngine.

Página removida pode continuar existindo em snapshot arquivado.

Isso permite gameplay investigativo.

---

# 59. OSINT

A internet deve permitir investigação emergente.

Exemplo:

Pessoa
↓
LinkUp
↓
empresa
↓
site corporativo
↓
produto
↓
TechByte
↓
documentação
↓
Redditor

Não criar somente caminhos únicos de missão.

---

# 60. ASSET POOL

Não duplicar assets.

Criar:

WebAssetRegistry

Categorias:

people
food
technology
cars
houses
travel
office
products
nature
generic
avatars
logos
icons

---

# 61. FORMATOS

Preferir assets otimizados.

Imagens:

WebP/AVIF conforme suporte real do projeto.

Criar:

thumbnail
small
medium
large

Não carregar imagem 4K para card de 200px.

---

# 62. LAZY LOADING

Obrigatório para:

images
feeds
products
comments
search results
video thumbnails

---

# 63. VIRTUALIZAÇÃO

Feeds grandes devem usar lista virtualizada quando necessário.

Não renderizar:

10.000 posts

simultaneamente no DOM.

---

# 64. MEMORY BUDGET

Criar métricas.

Medir:

Browser baseline
1 site
5 tabs
10 tabs
feed longo
commerce
search
social

Definir budgets depois dos primeiros benchmarks.

Não otimizar no escuro.

---

# 65. CONTENT STORAGE

Não guardar HTML completo para tudo.

Preferir estruturas compactas:

ArticleDocument
ProductDocument
PostDocument
ThreadDocument
RecipeDocument
VideoDocument
etc.

Renderer transforma dados em interface.

---

# 66. DATABASE

Escolher armazenamento compatível com arquitetura atual.

Separar conceitualmente:

STATIC WORLD CONTENT

de:

SAVE MUTATIONS.

Conteúdo base não deve ser duplicado integralmente em cada save.

Save guarda diferenças.

---

# 67. DELTA SAVE

Exemplo:

Base:
post likes = 120

Save:
+ jogador curtiu
+ evento alterou estado

Guardar delta.

Não duplicar banco inteiro da internet em cada slot.

Importante considerando os 5 slots de save.

---

# 68. VERSIONAMENTO

Todo content pack possui versão.

Exemplo:

web-core@1
technology@3
session3@2

Save deve registrar compatibilidade.

Planejar migrations.

---

# 69. DETERMINISMO

Dado:

worldSeed
saveState
contentPackVersion

o resultado procedural deve ser reproduzível.

Criar testes de snapshot.

---

# 70. LINKS QUEBRADOS

Links podem quebrar intencionalmente.

Mas precisam possuir motivo:

REMOVED
EXPIRED
DOMAIN_GONE
PRIVATE
404

Não deixar links quebrados por erro de conteúdo.

---

# 71. LINK VALIDATOR

Criar ferramenta de desenvolvimento:

VirtualWebLinkChecker.

Varre:

todos os documentos
↓
extrai links
↓
resolve URLs
↓
reporta broken links

Separar:

INTENTIONAL
ERROR

---

# 72. SEARCH COVERAGE TEST

Criar corpus de consultas.

Inicialmente:

10.000+ queries de teste.

Categorias:

shopping
technology
food
animals
cars
education
entertainment
travel
health
programming
random
typos
questions
mission queries

---

# 73. RANDOM QUERY FUZZING

Gerar queries aleatórias e verificar:

crash
latency
duplicate results
empty results
bad ranking
spoilers
broken URL

Rodar milhares automaticamente.

---

# 74. "PATO DE BORRACHA TEST"

Criar conjunto permanente de smoke tests aparentemente inúteis.

Exemplos:

pato de borracha
bolo de cenoura
curso javascript
cachorro pode comer banana
placa de vídeo
hotel barato
como trocar pneu
filmes de terror
cadeira gamer
história do rádio

Objetivo:

garantir que Goggle funcione fora da campanha.

---

# 75. SEARCH QUALITY METRICS

Medir:

Precision@1
Precision@3
Precision@10
zero-result rate
duplicate rate
latency P50
latency P95
latency P99

Também:

mission spoiler rate = 0

---

# 76. LATENCY TARGETS

Search:

P50 < 300ms
P95 < 1s
P99 < 3s

Hard timeout:

5s

Page:

P50 < 150ms
P95 < 500ms
P99 < 1s

Validar em hardware modesto.

---

# 77. HARDWARE QA

Não testar apenas máquina de desenvolvimento.

Criar perfis:

LOW
MID
HIGH

Windows prioritário.

Medir:

startup
RAM
CPU
search
tab switching
feeds
image loading
database size

---

# 78. VISUAL QA

Criar screenshots automatizados de marcas.

Detectar:

overflow
texto cortado
logo quebrada
imagem deformada
footer incorreto
layout idêntico
responsividade

Testar:

1024x640
1280x720
1366x768
1920x1080
2560x1440

---

# 79. BRAND SIMILARITY TEST

Criar auditoria visual/manual.

Pergunta:

"Se removermos o logo, ainda consigo perceber que são sites diferentes?"

Se resposta for NÃO:

identidade insuficiente.

Esse é um critério importante.

---

# 80. CONTENT DUPLICATION

Criar detector de duplicação.

Evitar:

mesmo título;
mesmo snippet;
mesmo artigo;
mesma descrição de produto;
mesmo comentário;

aparecendo excessivamente.

---

# 81. LANGUAGE QUALITY

Conteúdo em português deve possuir:

gramática coerente;
variação de tom;
registro adequado ao site;
variação regional quando planejada;
sem frases obviamente procedurais.

Não permitir placeholders.

---

# 82. PERSONALIDADE

Cada marca possui VoiceProfile.

Exemplo:

TechByte:
técnico + acessível.

Curioso:
popular + curioso.

B1:
jornalístico.

Blog pessoal:
informal.

Redditor:
usuários variados.

Corporate:
institucional.

Renderer não controla isso.

Content profile controla.

---

# 83. USUÁRIOS

Criar PersonaPool.

Usuários recorrentes podem aparecer em múltiplas plataformas quando fizer
sentido.

Não gerar sempre:

João Silva
Maria Santos
Carlos Oliveira.

Criar diversidade plausível de nomes, avatares e comportamentos.

---

# 84. COMENTÁRIOS

Comentários devem relacionar-se ao conteúdo.

Não preencher com:

"Muito bom!"
"Legal!"
"Gostei!"

em massa.

Criar comment intents:

question
agreement
disagreement
joke
experience
correction
complaint
recommendation
spam

---

# 85. DATAS

Datas precisam fazer sentido.

Artigo de 2023:

comentários não podem ser de 2022.

Produto lançado em 2026:

review não pode ser de 2024 sem justificativa narrativa.

Criar TemporalConsistencyValidator.

---

# 86. PREÇOS

Commerce deve possuir coerência interna.

Produtos equivalentes devem ter faixas plausíveis dentro da economia
fictícia.

Não:

celular básico = R$ 50
cabo USB = R$ 4.000

sem motivo.

Criar PriceModel.

---

# 87. ESTOQUE

Produtos podem possuir:

IN_STOCK
LOW_STOCK
OUT_OF_STOCK
DISCONTINUED
PREORDER

Estado pode mudar com WorldEvents.

---

# 88. REVIEWS

Rating precisa combinar com avaliações.

Não:

★ 1.2

e 95% dos comentários elogiando.

Criar distribuição coerente.

---

# 89. SOCIAL PROOF

Views
likes
followers
comments
sales
reviews

devem seguir escalas coerentes com popularidade.

Não gerar números completamente aleatórios.

Criar PopularityModel.

---

# 90. LINKS ENTRE SITES

Conteúdo deve referenciar outros sites naturalmente.

Exemplo:

TechByte
→ fonte: Nexora.

Redditor
→ link TechByte.

Blog
→ produto ShopNow.

Goggle
→ todos.

Isso cria a sensação de Web.

---

# 91. EXTERNALIDADE FICTÍCIA

Todos os links continuam dentro da Virtual Web.

Mesmo quando parecem:

https://example.com

o VirtualRouter resolve localmente.

Nenhum request HTTP real deve ocorrer.

---

# 92. SEGURANÇA

Garantir:

Browser virtual não acessa internet real.

VirtualNetwork não acessa internet real.

Terminal não acessa hosts reais.

Sites virtuais não possuem acesso privilegiado ao host.

Nenhum conteúdo gerado executa código arbitrário.

---

# 93. HTML DINÂMICO

Se conteúdo permitir rich text:

sanitizar.

Não permitir scripts arbitrários vindos de content packs.

Criar formato controlado de blocos:

paragraph
heading
image
quote
list
table
code
embedVirtual
etc.

---

# 94. MISSÕES

MissionEngine pode:

publicar conteúdo;
remover conteúdo;
alterar perfil;
alterar produto;
criar notícia;
liberar documento;
alterar ranking;
criar anúncio;
modificar site.

Sempre via eventos definidos.

Evitar manipulação direta de componentes React.

---

# 95. SIGNAL

Preparar conteúdo para:

fabricantes;
roteadores;
IoT;
telecom;
documentações;
reviews;
empresas;
produtos wireless.

Isso permitirá pesquisa e OSINT durante missões.

---

# 96. GHOSTMARKET

Preparar:

marketplaces;
anúncios;
bancos;
classificados;
redes sociais;
e-mail;
sites comerciais;
domínios parecidos;
landing pages;
tráfego pago fictício.

Tudo dentro da simulação.

---

# 97. ROOT

Preparar:

software;
apps;
games;
documentação;
repositórios;
fóruns;
reviews;
downloads virtuais;
versionamento;
DRM fictícia;
modding.

---

# 98. BREACH / LEAK

Preparar:

corporate sites;
notícias;
documentos;
cloud;
portais;
fóruns;
sites institucionais;
publicações;
archives.

---

# 99. OBSERVABILIDADE DE DESENVOLVIMENTO

Criar VirtualWebDevTools.

Somente development build.

Exibir:

current URL
brand
platform
document ID
seed
source
world state
mission visibility
search score
cache hit
render time

Isso será essencial para debug.

---

# 100. SEARCH DEBUGGER

Modo dev:

query
↓
mostrar score detalhado.

Exemplo:

TechByte result

text .......... 0.82
title ......... 1.00
authority ..... 0.73
freshness ..... 0.61
entity ........ 0.90
final ......... 0.84

Nunca disponível em produção.

---

# 101. CONTENT EXPLORER

Ferramenta dev para navegar:

Brands
Domains
Entities
Articles
Products
Users
Threads
SearchDocuments
WorldEvents

Permitir localizar inconsistências rapidamente.

---

# 102. CONTENT VALIDATION PIPELINE

Antes de um pack entrar na build:

SchemaValidator
↓
LinkValidator
↓
EntityValidator
↓
TemporalValidator
↓
DuplicateValidator
↓
SpoilerValidator
↓
AssetValidator
↓
SearchIndexer
↓
PackBuilder

Falha crítica:

build do pack deve falhar.

---

# 103. TESTES UNITÁRIOS

Cobrir:

VirtualRouter
SearchEngine
ranking
fuzzy search
KnowledgeBase
EntityGraph
ContentResolver
seed
cache
visibility
WorldEvents
DomainRegistry
BrandRegistry
PriceModel
PopularityModel
TemporalModel

---

# 104. TESTES DE INTEGRAÇÃO

Testar:

search → result → click → page

page → link → another site

mission event → article published → search updated

product → company → review

person → social → employer → corporate

removed page → 404/archive

save → close game → reopen → same world state

---

# 105. TESTES E2E

Fluxos:

A)
Goggle
→ pato de borracha
→ ShopNow
→ produto
→ reviews
→ vendedor

B)
Goggle
→ JavaScript
→ Educa+
→ curso
→ professor

C)
Goggle
→ Nexora
→ corporate
→ product
→ docs
→ TechByte

D)
Redditor
→ link
→ blog
→ Goggle Back

E)
missão
→ evento
→ notícia aparece
→ discussão aparece
→ busca atualiza.

---

# 106. TESTE DE PERSISTÊNCIA

Criar conteúdo procedural.

Salvar.

Fechar jogo.

Reabrir.

Resultado precisa ser idêntico.

Seed, URL e conteúdo não podem mudar.

---

# 107. TESTE DOS 5 SLOTS

Slot 1 e Slot 2 podem possuir WorldStates diferentes.

Exemplo:

Slot 1:
Orion acquisition PUBLIC.

Slot 2:
ainda SECRET.

Goggle deve produzir resultados diferentes.

Conteúdo base continua compartilhado.

---

# 108. STRESS TEST

Automatizar:

100.000 searches

10.000 navigations

milhares de materializações

múltiplos saves

Verificar:

memory leak
cache growth
database growth
broken references
latency degradation

---

# 109. SOAK TEST

Deixar Browser simulando navegação por horas.

Monitorar:

RAM
handles
listeners
cache
DB connections
render degradation

---

# 110. DATABASE SIZE TEST

Medir crescimento com:

10k docs
50k
100k
250k
500k

Não assumir que 100k é o limite.

Descobrir empiricamente.

---

# 111. SEARCH SCALE TEST

Benchmark:

10k
50k
100k
250k SearchDocuments.

Se 250k continuar barato, podemos aumentar cobertura.

Arquitetura não deve possuir limite artificial de 100k.

---

# 112. CONTENT COVERAGE

Criar dashboard:

Technology ........ 94%
Shopping .......... 91%
Food .............. 87%
Travel ............ 78%
Education ......... 93%
Automotive ........ 82%
Random ............ 68%

Cobertura baixa:

gerar novo content pack.

---

# 113. ZERO RESULT RATE

Não perseguir 0%.

Zero resultados são naturais.

Mas queries comuns não podem falhar frequentemente.

Objetivo será definido após corpus real de testes.

---

# 114. HUMAN QA

Automação não basta.

Realizar sessões:

tester recebe Goggle.

Instrução:

"Pesquise qualquer coisa por 30 minutos."

Registrar:

queries;
resultados ruins;
sites repetidos;
conteúdo estranho;
tempo;
links quebrados;
sensação de artificialidade.

Esse será um dos testes mais importantes.

---

# 115. "BREAK THE INTERNET" QA

Dar aos testers objetivo explícito:

"Tente provar que essa internet é falsa."

Testar:

queries absurdas;
palavrões;
frases enormes;
URLs inventadas;
typos;
nomes aleatórios;
produtos estranhos;
pesquisas muito específicas;
datas;
combinações contraditórias.

Registrar onde a ilusão quebra.

---

# 116. FIDELIDADE

A meta NÃO é:

copiar visualmente a internet atual.

A meta é:

reproduzir seus comportamentos.

Isso inclui:

sites bons;
sites ruins;
layouts antigos;
blogs pessoais;
portais modernos;
publicidade;
404;
links removidos;
reviews;
comentários;
SEO;
resultados ruins;
conteúdo popular;
conteúdo obscuro;
sites corporativos chatos;
fóruns caóticos.

Uma Web perfeita demais parecerá artificial.

---

# 117. IMPERFEIÇÕES CONTROLADAS

Adicionar ocasionalmente:

404;
produto esgotado;
post deletado;
perfil privado;
imagem indisponível;
link antigo;
site com design ultrapassado;
artigo corrigido;
comentário removido;
página lenta simulada.

Nunca de forma que quebre missão obrigatória.

---

# 118. NÃO TRANSFORMAR TUDO EM GAMEPLAY

Muita coisa deve simplesmente existir.

Uma receita pode ser só uma receita.

Um review pode ser só um review.

Um blog sobre jardinagem pode nunca participar de missão alguma.

Isso é necessário para que uma pista real não pareça imediatamente:

"conteúdo colocado aqui pelo desenvolvedor."

---

# 119. NÃO SINALIZAR CONTEÚDO DE MISSÃO

Não usar:

CYBER WAR
MISSION
IMPORTANT
QUEST

nos resultados.

Tudo pertence naturalmente ao mesmo mundo.

O jogador deve avaliar relevância.

---

# 120. CRITÉRIO DE ACEITE — EXPERIÊNCIA

A internet estará pronta quando um tester puder:

1. abrir Goggle;
2. pesquisar assuntos comuns;
3. receber resultados coerentes;
4. clicar;
5. navegar pelo site;
6. acessar outras páginas;
7. seguir links para outras marcas;
8. voltar;
9. pesquisar novamente;
10. encontrar conteúdo completamente diferente;
11. utilizar social;
12. visitar loja;
13. ler receita;
14. procurar curso;
15. ler fórum;
16. acessar documentação;
17. encontrar empresas;
18. descobrir pistas;
19. observar acontecimentos da história alterando a Web;
20. fechar e abrir o jogo sem perder a coerência.

---

# 121. CRITÉRIO DE ACEITE — PERFORMANCE

Obrigatório:

Search P50 < 300ms
Search P95 < 1s
Search P99 < 3s
Search hard timeout <= 5s

Page P50 < 150ms
Page P95 < 500ms
Page P99 < 1s

Sem memory leak significativo.

Sem crescimento infinito de cache.

Sem freeze perceptível ao indexar conteúdo.

---

# 122. CRITÉRIO DE ACEITE — QUALIDADE

Obrigatório:

- nenhuma pista futura indexada;
- nenhum link crítico quebrado;
- nenhuma URL duplicada;
- nenhuma marca principal sem identidade;
- nenhuma página importante genérica;
- nenhuma dependência da internet real;
- conteúdo procedural determinístico;
- navegação persistente;
- save isolation;
- busca livre;
- typos;
- 404;
- redirects;
- temporalidade;
- WorldEvents;
- Goggle funcional;
- integração com MissionEngine.

---

# 123. ORDEM DE IMPLEMENTAÇÃO

FASE A — FOUNDATION

VirtualWebWorld
DomainRegistry
BrandRegistry
VirtualRouter
ContentRepository
schemas
persistence

FASE B — SEARCH

Goggle
SearchIndex
ranking
fuzzy
intent
KnowledgeBase
autocomplete

FASE C — PLATFORM ENGINES

Editorial
Blog
Forum
Commerce
Corporate
Social
Video
Education
Recipe
etc.

FASE D — WORLD

EntityGraph
WorldClock
WorldEventBus
SocialGraph
AdNetwork
RecommendationEngine

FASE E — CONTENT PIPELINE

generator
validators
packs
indexer
asset pipeline

FASE F — SCALE

50 marcas
→ QA

100
→ QA

150
→ QA

200+
→ QA

Não gerar 200 marcas antes de validar as engines.

FASE G — LONG TAIL

500
→ 1.000
→ 2.000+

medindo cobertura e tamanho.

FASE H — POLISH

SEO
ads
404
archives
trending
recommendations
imperfeições
histórico
bookmarks
detalhes visuais.

---

# 124. PRIMEIRO MILESTONE

Antes de escalar, entregar vertical slice com:

Goggle

+

TechByte
Redditor
ShopNow
Wipédia
Cozinha Fácil
Educa+
Nexora Corporate
ViewTube
Blog pessoal
B1

E aproximadamente:

1.000–5.000 SearchDocuments.

Testar intensivamente.

Somente depois multiplicar as marcas.

---

# 125. TESTE DO VERTICAL SLICE

O vertical slice deve conseguir responder razoavelmente:

pato de borracha
javascript
placa de vídeo
bolo de cenoura
celular
wifi
filme de terror
curso inglês
carro usado
inteligência artificial
roteador
cachorro
receita pizza
linux
Nexora

E permitir navegação profunda em diferentes tipos de sites.

Se isso não parecer convincente:

NÃO escalar.

Corrigir primeiro arquitetura, conteúdo e identidade.

---

# 126. PRINCÍPIO FINAL

Não estamos criando 200 sites.

Estamos criando:

UMA INTERNET.

Marcas são habitantes dela.

Páginas são documentos dela.

Goggle é o índice dela.

EntityGraph é o conhecimento dela.

WorldEventBus é o tempo dela.

MissionEngine interfere nela.

ContentPacks dão escala a ela.

E o jogador deve conseguir esquecer, durante alguns minutos,
que absolutamente tudo aquilo está rodando localmente dentro do
CYBER WAR.