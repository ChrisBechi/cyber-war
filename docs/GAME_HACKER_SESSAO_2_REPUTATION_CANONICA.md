# GAME HACKER — SESSÃO 2: REPUTATION

## Status
**CANÔNICA — Main Story Spine**

## História Principal

Depois do incidente com a garota, a vida aparentemente volta ao normal. Gregory e `[NICKNAME]` fizeram as pazes e nenhum dos dois acredita que aquilo terá consequências maiores.

A mãe religa o modem, mas avisa que ele deve controlar o horário e lembra que suas notas estão ruins. Isso resolve a situação da Internet após a Sessão 1 e planta desde o início o problema escolar que culminará no final da sessão.

No fórum, a V2, os trabalhos anteriores e a relação com VEX começam a produzir reputação. Pela primeira vez alguém pergunta diretamente a `[NICKNAME]`: **“quanto você cobra?”**

Ele começa com serviços pequenos. Descobre que informação possui valor, aceita trabalhos cuja justificativa não consegue confirmar e gradualmente percebe que pode ser pago não apenas para resolver problemas, mas também para prejudicar pessoas e negócios.

Gregory ainda está próximo e inicialmente acha interessante o amigo ganhar dinheiro com informática. O afastamento começa discretamente: convites para jogar recebem respostas como “depois, tô terminando um negócio”.

NULL continua provocando, mas começa a reconhecer que `[NICKNAME]` realmente possui talento. O dinheiro alimenta o protagonista, porém o reconhecimento de NULL começa a alimentar algo ainda mais perigoso: seu ego.

Paralelamente, a garota continua leiga e determinada. Ela não conhece o nickname. Apenas procura entender, como uma pessoa comum, o que aconteceu com sua rede social, fazendo perguntas sobre sessões, dispositivos, histórico de login e recuperação de conta. Nada disso ainda identifica `[NICKNAME]`.

Enquanto sua capacidade técnica cresce, sua vida escolar piora. Notificações do portal mostram notas baixas, Gregory brinca que ele vai repetir e a mãe demonstra preocupação. No final, `[NICKNAME]` percebe que realmente será reprovado.

Depois de passar a sessão alterando sistemas para outras pessoas, surge a conclusão perigosa: **se consegue mudar sistemas dos outros, por que aceitar que um sistema decida o próprio futuro?**

Ele invade o sistema educacional fictício e tenta alterar sua situação acadêmica. O resultado não é necessariamente sucesso: uma alteração discreta pode passar despercebida; ganância, alterações absurdas, manipulação de muitos alunos, reclamações ou OPSEC ruim podem provocar auditoria, restauração dos dados e reprovação.

Independentemente do resultado escolar, a decisão foi tomada: `[NICKNAME]` utilizou hacking para manipular diretamente a própria realidade.

Sua reputação também chama a atenção de uma pequena guilda hacker. Um convite privado chega. Ele aceita.

**SESSION COMPLETE — REPUTATION**

**SESSION 3 — BLACK HAT**

---

# Tema

> **“Se eu consigo fazer isso, quanto vale o que eu sei?”**

Progressão:

**Conhecimento → dinheiro → tolerância moral → crime como serviço → reconhecimento → ego → vantagem pessoal.**

---

# Abertura — Modem e mãe

> **MÃE:** Liguei o modem de novo.  
> **MÃE:** Mas fica de olho no horário.  
> **MÃE:** Não quero você virando a noite nesse computador.  
>
> **[NICKNAME]:** tá  
>
> **MÃE:** “Tá” não. Você tem aula amanhã.  
> **MÃE:** E suas notas já não estão aquelas coisas.

A mãe deve continuar aparecendo ocasionalmente durante a campanha como ligação de `[NICKNAME]` com a vida comum.

---

# MAIN QUESTS

## MISSÃO 2.1 — QUANTO VOCÊ COBRA?

**Objetivo:** realizar o primeiro serviço contratado diretamente pela reputação de `[NICKNAME]`.

**Função narrativa:** transformar reputação em dinheiro.

**Conceitos:** OSINT básico, contas e serviços, autenticação, credenciais e correlação de informações.

**Subobjetivos:**
1. Receber contato indicado por VEX.
2. Entender o problema.
3. Levantar informações públicas.
4. Identificar serviços associados.
5. Encontrar uma solução válida na simulação.
6. Entregar o resultado.
7. Receber pagamento.

**Consequências:** `PAID_JOBS_UNLOCKED = TRUE`; expansão dos Jobs independentes.

---

## MISSÃO 2.2 — QUEM ESTÁ POR TRÁS?

**Objetivo:** descobrir quem provavelmente administra determinado pequeno site/plataforma.

**Função narrativa:** mostrar que informação também possui valor.

**Conceitos:** Footprinting, WHOIS, DNS, subdomínios, pesquisa pública, correlação e Google Dorks introdutórios.

**Subobjetivos:**
1. Receber o domínio.
2. Pesquisar informações públicas.
3. Examinar infraestrutura exposta.
4. Identificar domínios/subdomínios relacionados.
5. Correlacionar registros.
6. Determinar o provável responsável.
7. Produzir relatório.
8. Receber pagamento.

---

## MISSÃO 2.3 — CAIXA DE ENTRADA

**Objetivo:** obter o acesso solicitado por um cliente cuja história não pode ser confirmada.

**Função narrativa:** `[NICKNAME]` percebe que não saber a verdade pode ser conveniente.

**Conceitos:** OSINT, autenticação, reutilização de credenciais, engenharia social e análise de serviços.

**Subobjetivos:**
1. Receber a justificativa.
2. Investigar o alvo.
3. Descobrir serviços utilizados.
4. Identificar caminhos.
5. Escolher abordagem.
6. Conseguir o acesso necessário.
7. Localizar a informação.
8. Entregar.
9. Receber pagamento maior.

**Design:** múltiplas soluções suportadas pela simulação são válidas.

---

## MISSÃO 2.4 — PORTA ABERTA

**Objetivo:** mapear tecnicamente uma pequena organização.

**Função narrativa:** preparar a primeira invasão organizacional detalhada.

**Conceitos:** Footprinting, scanning, enumeração, serviços, versões, vulnerabilidades, CVEs e superfície de ataque.

**Subobjetivos:**
1. Identificar infraestrutura.
2. Descobrir hosts relevantes.
3. Enumerar serviços.
4. Identificar versões.
5. Pesquisar vulnerabilidades conhecidas.
6. Encontrar possíveis vetores.
7. Produzir mapa técnico.

O objetivo é reconhecer, não comprometer.

Ao final:

> **CLIENTE:** perfeito.  
> **CLIENTE:** consegue entrar?

---

## MISSÃO 2.5 — ACESSO INICIAL

**Objetivo:** obter acesso à organização reconhecida anteriormente e recuperar os dados solicitados.

**Função narrativa:** primeira invasão organizacional detalhada.

**Cadeia conceitual:** `RECON → ACCESS → CONTROL → OBJECTIVE → EXIT`.

**Subobjetivos:**
1. Revisar o reconhecimento.
2. Selecionar ponto de entrada.
3. Obter acesso inicial.
4. Entender o ambiente.
5. Identificar permissões.
6. Obter acesso suficiente.
7. Localizar dados.
8. Coletar.
9. Transferir.
10. Avaliar rastros.
11. Sair.

Existe rota canônica, mas conhecimentos opcionais podem abrir alternativas.

---

## MISSÃO 2.6 — OFFLINE

**Objetivo:** tornar temporariamente indisponível um pequeno serviço.

**Função narrativa:** primeira sabotagem paga de disponibilidade.

**Conceitos:** disponibilidade, serviços, DoS no ambiente simulado, monitoramento, ruído e reação do alvo.

**Subobjetivos:**
1. Receber alvo.
2. Reconhecer infraestrutura.
3. Identificar serviço.
4. Preparar operação.
5. Tornar serviço indisponível.
6. Confirmar.
7. Encerrar.
8. Receber pagamento.

`ping` reaparece naturalmente como conhecimento já adquirido. A missão também introduz formalmente a persistência/reação dos alvos.

---

## MISSÃO 2.7 — HORÁRIO DE PICO

**Premissa:** um empresário paga para que o delivery concorrente fique indisponível durante seu período de maior movimento.

**Objetivo:** interromper temporariamente o serviço concorrente durante o horário contratado.

**Função narrativa:** `[NICKNAME]` passa a vender sua capacidade para gerar vantagem comercial ilícita.

**Conceitos:** Slowloris representado dentro da simulação, DoS, comportamento HTTP/Apache, conexões, headers, disponibilidade, monitoramento, impacto e ruído.

**Subobjetivos:**
1. Identificar o serviço web.
2. Analisar seu comportamento.
3. Preparar a operação simulada.
4. Aguardar o horário contratado.
5. Executar a indisponibilidade.
6. Monitorar.
7. Confirmar o impacto.
8. Encerrar no período combinado.
9. Avaliar rastros.

Exagerar no impacto, prolongar a indisponibilidade ou gerar ruído excessivo pode aumentar Heat, evidências e reação do alvo.

**Consequência:** `EVIDENCE #007`.

---

## MISSÃO 2.8 — TIRA DO AR

**Objetivo:** interferir deliberadamente em um perfil indicado por um cliente.

**Função narrativa:** primeiro trabalho claramente malicioso contra uma pessoa por dinheiro.

Momento central:

> **[NICKNAME]:** por quê?  
> **CLIENTE:** importa?

Ele aceita.

**Conceitos:** OSINT, engenharia social, autenticação/sessão e mecanismos da plataforma, conforme a abordagem.

**Subobjetivos:**
1. Receber proposta.
2. Investigar alvo.
3. Escolher abordagem.
4. Obter capacidade de interferência.
5. Tirar o perfil do ar.
6. Confirmar.
7. Receber pagamento.

---

## MISSÃO 2.9 — NÃO TÁ RUIM

**Objetivo:** resolver um problema técnico publicado no fórum antes ou em paralelo com NULL.

**Função narrativa:** NULL passa a reconhecer o talento de `[NICKNAME]`.

> **NULL:** não tá ruim.  
> **[NICKNAME]:** só isso?  
> **NULL:** quer um certificado?

Se o jogador antecipar a solução que NULL recomendaria, ocorre **Prior Knowledge Recognition**:

> **NULL:** ...  
> **NULL:** olha só.  
> **NULL:** parece que você já sabia o que eu ia falar, né?

**Função emocional:** dinheiro alimenta `[NICKNAME]`; respeito de NULL alimenta o ego.

---

## MISSÃO 2.10 — VAI REPETIR

**Objetivo:** descobrir que a situação escolar chegou ao limite.

Exemplo:

```text
SITUAÇÃO ACADÊMICA

Média necessária: 6.0
Média atual: 4.3

Situação:
REPROVAÇÃO
```

> **GREGORY:** fudeu  
> **[NICKNAME]:** percebi  
> **GREGORY:** tua mãe vai te matar  
> **[NICKNAME]:** valeu pela ajuda  
> **GREGORY:** você invade metade da internet mas vai repetir matemática

Gregory não sugere a invasão. A frase apenas planta a ideia.

`[NICKNAME]` olha para o Portal do Aluno e abre o terminal.

---

## MISSÃO 2.11 — APROVADO
### MISSÃO FINAL CANÔNICA

**Objetivo:** tentar alterar a própria situação acadêmica para evitar a reprovação.

**Função narrativa:** primeira operação complexa executada inteiramente para vantagem pessoal.

O sistema educacional é fictício e simulado.

**Conceitos possíveis:** footprinting, reconhecimento web, aplicações/APIs, autenticação/autorização, SQLi ou outra falha web adequada ao sistema fictício, IDOR/BOLA, escalada de privilégios, bancos de dados, registros, OPSEC e rastros.

A vulnerabilidade canônica será definida durante o detalhamento técnico.

**Subobjetivos:**
1. Identificar o portal.
2. Mapear a infraestrutura fictícia.
3. Descobrir aplicações/serviços.
4. Entender a relação entre alunos, professores e notas.
5. Analisar a superfície.
6. Encontrar vetor válido.
7. Obter acesso.
8. Obter permissões necessárias.
9. Localizar o próprio registro.
10. Decidir o que alterar.
11. Confirmar.
12. Avaliar rastros.
13. Sair.

### Liberdade
O banco contém outros estudantes. O simulador não impede artificialmente que o jogador consulte ou altere outros registros.

### Sucesso técnico ≠ sucesso da missão
Uma alteração pode retornar `UPDATE SUCCESSFUL` e ainda produzir consequências posteriores.

### Solução discreta
Alterar apenas o necessário e de forma coerente produz menor anomalia.

### Ganância
Uma mudança como `4.3 → 10.0` pode funcionar tecnicamente, mas despertar atenção. A escola pode auditar, restaurar os dados e reprovar `[NICKNAME]`.

### Alterações em massa
Mexer em Gregory ou em vários alunos aumenta o número de pessoas afetadas. Reclamações podem revelar a invasão, provocar auditoria e restaurar todas as notas.

### Tipos de erro
- ganância;
- escala;
- ruído técnico;
- impacto humano;
- OPSEC ruim.

### Falha persistente
Esta missão introduz:

`MISSION FAILED → CONSEQUENCE → STORY CONTINUES`

O evento canônico é a decisão de invadir e manipular a própria situação. Aprovação ou reprovação é consequência variável.

---

# Reação da mãe

## Se funcionar

> **MÃE:** Vi sua nota.  
> **MÃE:** Parabéns.  
> **MÃE:** Eu sabia que você conseguia quando resolvesse estudar.  
>
> **[NICKNAME]:** valeu mãe

Ela está orgulhosa de algo que não aconteceu como imagina.

## Se for revertido

> **MÃE:** A escola ligou.  
> **MÃE:** Você repetiu.  
> **MÃE:** Eu trabalho o dia inteiro.  
> **MÃE:** Só queria que você levasse a escola um pouco a sério.

Ela não precisa descobrir a invasão.

---

# Encerramento — convite da guilda

O convite não depende do resultado escolar. Ele vem da reputação construída durante a sessão.

```text
PRIVATE INVITATION

We've been watching your work.

Interested?
```

Não são Anonymous. É um grupo pequeno, mas revela uma camada acima dos fóruns e clientes: guildas, operações coletivas e hierarquia.

`[NICKNAME]` aceita.

```text
MEMBERSHIP REQUEST ACCEPTED
```

**SESSION COMPLETE — REPUTATION**

**SESSION 3 — BLACK HAT**

---

# Estado final

No início:

> **“Será que alguém pagaria pelo que eu sei?”**

No final:

> **“Se consigo alterar os sistemas dos outros, por que deveria aceitar que um sistema decida o meu futuro?”**

Transformação:

`CURIOSIDADE → REPUTAÇÃO → DINHEIRO → CRIME COMO SERVIÇO → RECONHECIMENTO DE NULL → EGO → VANTAGEM PESSOAL → BLACK HAT`

---

# Decisão reservada para a Sessão 3

**A Vingança Digital** foi removida da Sessão 2 e reservada para **Sessão 3 — BLACK HAT**, onde a evolução envolvendo phishing avançado, exploração de vulnerabilidade client-side/RCE, payload e pós-exploração fará sentido com a maturidade técnica e criminal do protagonista.
