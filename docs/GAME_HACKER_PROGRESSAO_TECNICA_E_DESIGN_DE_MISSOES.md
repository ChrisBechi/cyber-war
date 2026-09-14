# GAME HACKER — Progressão Técnica e Design de Missões

## Status
**Documento consolidado — versão vigente desta área do design.**

Este documento complementa e consolida as decisões já tomadas sobre progressão técnica, Main Quests, Side Quests, guildas, conhecimento do jogador, Wiki, ferramentas, exploits, persistência dos alvos, soluções alternativas e replayability.

---

# 1. Filosofia central

> **Main Quest = Profundidade**  
> **Side Quest = Amplitude**  
> **Guildas = Conhecimento coletivo, recursos especiais e especialização**  
> **Conhecimento real do jogador = nunca bloqueado**

A campanha principal deve ser tecnicamente rica por conta própria.

As Main Quests apresentam técnicas relevantes, elaboradas e progressivamente mais profundas, combinando conhecimentos anteriores em operações cada vez mais complexas.

As Side Quests, Character Quests, fóruns, CTFs, Bug Bounties e guildas complementam esse repertório. Elas apresentam técnicas similares, alternativas, especializações e novos contextos de aplicação.

Não existe obrigação de distribuir todo o catálogo técnico do jogo entre Main e Side Quests. Uma técnica só deve aparecer quando fizer sentido para a narrativa, para a cadeia de personagem, para a guilda ou para o gameplay.

---

# 2. Técnicas não são bloqueadas

Nenhuma técnica suportada pela simulação precisa ser artificialmente desbloqueada para poder ser utilizada.

Se o jogador souber o que está fazendo e encontrar uma situação válida, ele pode tentar utilizar a técnica mesmo que `[NICKNAME]` ainda não tenha recebido uma missão, tutorial ou entrada completa na Wiki sobre ela.

O jogo não deve responder:

> "Você ainda não desbloqueou esta técnica."

O que pode ser desbloqueado é:

- documentação na Wiki;
- explicações passo a passo;
- ferramentas;
- exploits;
- infraestrutura;
- contatos;
- mercados;
- oportunidades;
- pesquisas;
- informações;
- acesso a recursos de guildas.

## Regra

> **O personagem progride adquirindo conhecimento documentado, ferramentas, infraestrutura e contatos. O jogador progride aprendendo o próprio jogo.**

Isso cria replayability natural: em uma nova campanha, `[NICKNAME]` começa novamente com poucos recursos, mas o jogador mantém o conhecimento adquirido na jogatina anterior.

---

# 3. Wiki e aprendizado

A Wiki funciona como uma base de conhecimento progressiva.

Uma técnica pode existir e ser utilizável mesmo com sua entrada incompleta.

Exemplo conceitual:

```text
SQL INJECTION

01. Conceito                 ✓
02. Identificação            ✓
03. Tipos                    ✓
04. Testes                   ?
05. Exploração               ?
06. Mitigação                ?
07. Casos conhecidos         ?
08. Anotações pessoais       2
```

O jogador pode ampliar a documentação através de:

- Main Quests;
- Side Quests;
- especialistas;
- fóruns;
- CTFs;
- Bug Bounties;
- guildas;
- pesquisa;
- experimentação bem-sucedida.

A Wiki ensina e documenta. Ela não funciona como uma trava de permissão.

---

# 4. Conhecimento, prática e capacidade

O jogo pode manter medidores informativos de evolução técnica, mas eles nunca devem impedir a execução de uma ação válida.

Exemplo:

```text
WEB SECURITY

Conhecimento     ███████░░░
Prática          █████░░░░░
Experiência      ██████░░░░
```

Esses valores servem para:

- histórico do jogador;
- estatísticas;
- progressão visual;
- conquistas;
- Wiki;
- avaliação final;
- reconhecimento de especialização.

Eles não devem criar requisitos artificiais como:

```text
SQL Injection requer nível 50.
```

---

# 5. Quando uma técnica conta como utilizada

Uma técnica é registrada como **utilizada com sucesso** quando:

1. o jogador a executa em uma missão ou contra um alvo válido dentro da simulação;
2. a execução produz o resultado esperado daquela técnica.

Não basta digitar um comando ou iniciar uma tentativa.

Exemplo:

```text
$ ping www.google.com

Reply from ...
```

Resultado:

**Ping — utilização bem-sucedida registrada.**

Enquanto:

```text
$ ping mslkmclksm

Unknown host
```

não registra uma utilização bem-sucedida.

Essa regra deve valer para todo o catálogo: o sistema avalia o resultado da ação, não apenas a entrada do jogador.

---

# 6. Technical Journey

Ao longo da campanha, o jogo registra a jornada técnica do jogador.

Ao final, pode apresentar um relatório semelhante a:

```text
TECHNICAL JOURNEY

Techniques discovered:        73
Techniques attempted:         61
Techniques successful:        54
Alternative solutions:        14
Prior knowledge events:        6

Technical completion:         68%
```

O relatório pode mostrar:

- técnicas conhecidas;
- técnicas utilizadas;
- técnicas utilizadas com sucesso;
- técnicas nunca utilizadas;
- áreas mais exploradas;
- soluções alternativas encontradas;
- soluções de conhecimento prévio;
- ferramentas usadas;
- exploits utilizados;
- guildas que influenciaram a jornada.

Isso incentiva novas campanhas.

Quando o jogador conseguir utilizar com sucesso todas as técnicas reconhecidas pelo sistema ao longo de uma ou mais jogatinas, pode desbloquear uma conquista especial de completude técnica.

---

# 7. Prior Knowledge Recognition

O jogo deve reconhecer situações em que o jogador utiliza espontaneamente uma solução que um especialista apresentaria posteriormente como uma abordagem mais inteligente, elegante ou apropriada.

Isso não significa apenas usar uma técnica "antes da hora".

É necessário existir uma solução esperada ou uma recomendação futura de especialista, e o jogador antecipá-la por conhecimento próprio.

## Exemplo — V2 e NULL

Fluxo comum:

```text
Jogador resolve V2 pela abordagem esperada
        ↓
NULL analisa
        ↓
NULL critica a complexidade
        ↓
NULL sugere uma abordagem mais elegante
```

Fluxo de conhecimento prévio:

```text
Jogador já utiliza a abordagem
que NULL normalmente recomendaria
        ↓
Sistema reconhece
        ↓
NULL reage de forma diferente
```

Exemplo de diálogo:

> **NULL:** ...  
> **NULL:** olha só.  
> **NULL:** parece que você já sabia o que eu ia falar, né?

Esse sistema pode existir para outros especialistas, como PATCH, HEX, ZERO, ECHO, ARCHER, RAVEN e GHOST.

Deve ser usado com moderação para que cada reconhecimento pareça especial.

---

# 8. Qualidade das soluções

Uma solução bem-sucedida não significa necessariamente uma boa solução.

O jogo deve permitir:

- solução canônica;
- solução mais inteligente;
- solução mais simples;
- solução mais silenciosa;
- solução mais rápida;
- solução mais barata;
- solução mais arriscada;
- solução mais ruidosa;
- solução tecnicamente pior que ainda funciona.

Cada abordagem pode ser avaliada por fatores como:

| Fator | Possível consequência |
|---|---|
| Número de passos | tempo e complexidade |
| Ruído | chance de chamar atenção |
| Risco | probabilidade de consequências |
| Evidências | quantidade de rastros |
| Custo | dinheiro/infraestrutura/ferramentas |
| Tempo | duração da operação |
| Experiência | qualidade técnica da execução |
| Impacto | dano causado |
| Persistência | consequências futuras |

## Regra

> **Funcionou não significa que foi uma boa solução.**

Uma abordagem melhor pode ter menos passos, gerar menos ruído, ser menos arriscada e produzir mais experiência.

Uma abordagem pior também pode funcionar, mas deixar rastros, causar indisponibilidade, aumentar Heat ou provocar reações futuras.

---

# 9. Main Quest — profundidade

A campanha principal não deve depender das Side Quests para ser tecnicamente interessante.

As Main Quests devem:

- apresentar técnicas importantes;
- aprofundá-las;
- combinar técnicas anteriores;
- criar operações em múltiplas etapas;
- oferecer problemas tecnicamente ricos;
- possuir pelo menos uma rota viável independentemente do conteúdo opcional.

Uma missão avançada pode combinar conceitualmente:

**Reconhecimento → enumeração → análise → exploração → acesso → pós-exploração → objetivo → gestão de rastros.**

As técnicas escolhidas para a Main Quest devem servir à narrativa e à evolução do protagonista.

Não é necessário colocar 100% do catálogo técnico na história principal.

---

# 10. Side Quests e Character Quests — amplitude

As missões secundárias servem para ampliar o repertório.

Elas podem:

- ensinar técnicas novas;
- aprofundar uma técnica já vista;
- apresentar uma técnica semelhante;
- apresentar outro contexto para a mesma vulnerabilidade;
- ensinar uma abordagem alternativa;
- oferecer novas ferramentas;
- introduzir especialistas;
- fornecer infraestrutura;
- abrir mercados;
- fornecer exploits;
- gerar Technical Payoffs futuros.

Side Quest não significa conteúdo descartável.

Uma técnica aprendida opcionalmente pode reaparecer dezenas de missões depois como uma solução alternativa extremamente útil.

---

# 11. Especialistas e áreas da cybersecurity

Cada personagem especialista pode abrir uma área ou conjunto coerente de conhecimentos.

Exemplos já definidos:

| Personagem | Área |
|---|---|
| PATCH | Sistemas, servidores, administração, logs, defesa e forensics |
| ZERO | Redes, protocolos, wireless e infraestrutura |
| HEX | Web, AppSec e APIs |
| ECHO | Android, iOS e mobile |
| RAVEN | Engenharia social e OSINT humano |
| GHOST | OPSEC, anonimato, identidades e rastros |
| WORM | Malware, persistence, RAT, rootkits e áreas relacionadas |
| ARCHER | Vulnerability research, CVEs, exploits e zero-days |
| BYTE | Economia underground, contatos e monetização |
| BROKER | Acessos, dados, exploits e mercados avançados |

Não é necessário que cada personagem ensine todas as técnicas possíveis de sua área.

A cadeia narrativa vem primeiro.

---

# 12. Guildas e distribuição técnica

As guildas são outra fonte importante de progressão técnica.

As técnicas do universo podem ser distribuídas entre:

- Main Quests;
- Side/Character Quests;
- guildas;
- CTFs;
- Bug Bounties;
- fóruns;
- Wiki;
- exploração;
- atividades livres.

Guildas podem possuir identidades técnicas próprias.

Exemplos:

- Web/AppSec;
- redes e infraestrutura;
- reversing e exploit research;
- malware;
- engenharia social e OSINT;
- grupos multidisciplinares de elite.

Conforme o jogador entra em guildas mais relevantes, pode obter acesso a:

- ferramentas privadas;
- exploits;
- infraestrutura compartilhada;
- pesquisas;
- informações;
- mercados internos;
- especialistas;
- técnicas pouco documentadas;
- operações coletivas;
- conhecimento produzido pelos próprios membros.

Guildas melhores não significam apenas missões que pagam mais.

Elas significam acesso a um ecossistema técnico mais sofisticado.

---

# 13. Técnicas podem reaparecer

Uma técnica não deve existir apenas em uma única "missão daquela técnica".

Uma vulnerabilidade ou conceito pode aparecer em contextos diferentes.

Exemplo:

1. CTF;
2. Side Quest de HEX;
3. Bug Bounty;
4. Main Quest;
5. alvo descoberto espontaneamente;
6. organização que já corrigiu a mesma classe de problema.

Isso reforça a ideia de que vulnerabilidades fazem parte do mundo, e não de fases isoladas.

---

# 14. Técnica, CVE e exploit são coisas diferentes

O design deve separar claramente:

### Técnica
Conhecimento ou método utilizado pelo jogador.

### Vulnerabilidade/CVE
Uma falha específica existente em determinado software, serviço ou contexto.

### Exploit
Um recurso/ferramenta utilizado para explorar uma condição específica.

### Ferramenta
Software ou recurso usado durante a operação.

### Infraestrutura
Recursos controlados pelo jogador ou por aliados.

### Contato
Pessoa que pode fornecer conhecimento, serviços, acesso, informação ou recursos.

Isso permite que CVEs e exploits envelheçam, desapareçam ou percam valor sem "apagar" o conhecimento técnico do jogador.

---

# 15. Persistência e reação dos alvos

Depois que o jogador ataca uma rede, site, sistema, empresa ou organização, o alvo pode ficar momentaneamente indisponível.

Quando retorna, o estado pode ter mudado.

Fluxo conceitual:

```text
ATAQUE
   │
   ▼
ALVO TEMPORARIAMENTE INDISPONÍVEL
   │
   ▼
RETORNO
   │
   ├── organização ignora o incidente
   │      └── falha continua
   │
   ├── sistema é atualizado
   │      └── falha pode desaparecer
   │
   ├── correção é insuficiente
   │      └── problema estrutural permanece
   │
   ├── credencial comprometida não é detectada
   │      └── continua válida
   │
   └── comprometimento é descoberto
          └── senha/acesso é alterado
```

A resposta pode variar conforme:

- gravidade;
- natureza da falha;
- organização;
- maturidade de segurança;
- visibilidade do incidente;
- impacto causado.

O mundo deve parecer capaz de reagir ao jogador.

Uma solução que funcionou anteriormente não é garantia de funcionar novamente.

---

# 16. Technical Payoff

Toda especialização opcional relevante deve possuir potencial para retornar posteriormente.

Exemplo:

```text
SIDE QUEST — RAVEN
        │
Pretexting / OSINT humano
        │
        ▼
MAIN QUEST FUTURA
        │
Alvo difícil
        │
        ├── rota canônica
        │
        └── conhecimento de RAVEN
             permite outra abordagem
```

O mesmo pode acontecer com conhecimentos de:

- PATCH;
- HEX;
- ZERO;
- ECHO;
- WORM;
- ARCHER;
- GHOST;
- guildas.

O jogador deve ocasionalmente pensar:

> "Ainda bem que fiz aquela missão."

---

# 17. Múltiplas soluções

Toda Main Quest importante possui uma solução projetada que garante a progressão.

Entretanto, o objetivo pode aceitar outras abordagens suportadas pela simulação.

Essas alternativas podem vir de:

- Side Quests;
- especialistas;
- guildas;
- ferramentas;
- exploits;
- contatos;
- infraestrutura;
- conhecimento real do jogador.

O jogo não deve apresentar constantemente menus como:

```text
A) solução web
B) engenharia social
C) comprar acesso
```

Ele apresenta o problema.

O jogador utiliza seu repertório para descobrir possibilidades.

A sensação desejada é:

> **"Eu tive essa ideia."**

---

# 18. Soluções não ensinadas

O sistema deve permitir uma categoria especial:

> **solução suportada, mas ainda não ensinada ao jogador.**

Se o jogador conhece a abordagem e o ambiente simulado suporta aquela interação, ela pode funcionar.

Isso diferencia uma simulação de uma árvore tradicional de habilidades.

Também cria oportunidades para:

- easter eggs;
- diálogos alternativos;
- sequence breaks controlados;
- reconhecimento por especialistas;
- conquistas;
- replayability.

---

# 19. Replayability

Na primeira campanha:

**jogador e protagonista aprendem juntos.**

Em campanhas posteriores:

**o protagonista começa novamente sem seus recursos, mas o jogador mantém seu conhecimento real.**

Isso permite:

- utilizar técnicas antecipadamente;
- encontrar soluções melhores;
- evitar soluções ruins;
- gerar diálogos especiais;
- descobrir caminhos não vistos;
- completar técnicas ausentes do Technical Journey;
- perseguir a conquista de completude técnica.

O jogo pode funcionar como um **New Game+ baseado em conhecimento**, sem precisar conceder poderes artificiais ao personagem.

---

# 20. Regra para o Mission Map

Ao retornar ao mapa completo de missões, cada missão importante deve possuir campos como:

| Campo | Função |
|---|---|
| ID | identificação |
| Sessão | posição na campanha |
| Tipo | Main / Character / Side / Guild / CTF / Bug Bounty / etc. |
| Doador | personagem/sistema |
| Pré-requisitos | dependências narrativas |
| Objetivo | resultado esperado |
| Rota canônica | solução garantida |
| Técnicas principais | técnicas aprofundadas |
| Técnicas praticadas | conhecimentos reutilizados |
| Alternativas reconhecidas | outras soluções previstas |
| Prior Knowledge | soluções antecipáveis |
| Ferramentas | recursos relevantes |
| Exploits/CVEs | quando aplicável |
| Wiki | documentação adquirida |
| Consequências | ruído, evidências, Heat, persistência |
| Technical Payoff | onde esse conhecimento pode retornar |
| Desbloqueios | contatos, ferramentas, infraestrutura, mercados etc. |

---

# 21. Filosofia final consolidada

**Game Hacker não é uma sequência de fases que desbloqueiam botões de hacking.**

É um ecossistema técnico persistente.

A Main Quest fornece **profundidade**.

As Side Quests fornecem **amplitude**.

As guildas fornecem **conhecimento coletivo, recursos e especialização**.

A Wiki fornece **documentação**.

Ferramentas, exploits, infraestrutura e contatos ampliam o que o personagem possui.

Mas o conhecimento real do jogador nunca é artificialmente bloqueado.

Quanto mais o jogador aprende, mais soluções consegue enxergar.

Quanto mais explora, maior se torna seu arsenal.

Quanto mais joga novamente, mais percebe que situações antigas poderiam ter sido resolvidas de maneiras completamente diferentes.

> **Main Quest = Profundidade**  
> **Side Quest = Amplitude**  
> **Guildas = Conhecimento coletivo**  
> **Technical Payoff = Recompensa pela exploração**  
> **Conhecimento do jogador = Liberdade**
