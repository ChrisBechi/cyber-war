# GUILDA BREACH

> Arco consolidado --- identidade, filosofia, 4 missões, final e saída
> do protagonista

BREACH é uma guilda especializada em intrusão e segurança ofensiva. Seu
arco é não linear e pode coexistir com LEAK e outras guildas: o
protagonista não precisa concluir SEM ROSTO, O PREÇO ou qualquer outro
arco específico antes de começar BREACH. A guilda reage dinamicamente ao
histórico já realizado no save.

## 1. Identidade da guilda

BREACH reúne especialistas em intrusão, pentest, bug bounty e operações
ofensivas. Nem todas as missões são clandestinas: algumas possuem
autorização e escopo; outras são trabalhos cinzentos ou operações
underground.

> "Entrar não prova nada. O que você faz depois que entra, sim."

A diferença para LEAK é estrutural. LEAK procura informação e decide o
que fazer com ela. BREACH trata o acesso como ferramenta: onde está a
entrada, até onde ela leva, qual é o objetivo e qual é o impacto
necessário para concluí-lo.

### Princípios

-   Acesso é meio, não objetivo.

-   Não cause impacto desnecessário apenas para provar capacidade.

-   Em bug bounty, demonstre o impacto mínimo suficiente e reporte.

-   Em pentest, respeite o objetivo e o escopo contratado.

-   Em operações clandestinas, a ausência de autorização muda
    completamente o risco.

-   Abortar é uma decisão tática: abandonar uma tentativa comprometida,
    não abandonar a missão.

-   O jogador precisa concluir aquilo que começou; pode recuar,
    reorganizar e tentar outra abordagem.

-   A guilda valoriza competência, disciplina, compreensão do ambiente e
    controle de impacto.

### Personalidade

BREACH gosta de sistemas difíceis e de problemas que outras pessoas
consideram impossíveis. O perigo moral da guilda é a obsessão técnica:
em algum momento, a capacidade de entrar pode começar a ser confundida
com justificativa para entrar.

> "Se responde, tem uma porta."

A guilda não é ativista como LEAK. Também não é uma organização
puramente criminosa. Ela ocupa uma zona híbrida entre segurança ofensiva
profissional, desafios técnicos, contratos underground e operações de
intrusão.

## 2. Personagens principais

### GHOST --- líder / arquiteto

Calmo, metódico e pouco impressionável. Planeja operações, observa como
os membros reagem sob pressão e insiste que objetivo e impacto importam
tanto quanto conseguir acesso. Não é um mestre onisciente; pode errar
moralmente mesmo quando está tecnicamente correto.

### HEX --- exploração

Competitivo, provocador e fascinado por sistemas difíceis. É o membro
que mais facilmente transforma 'ninguém conseguiu' em motivo para
tentar.

### TRACE --- reconhecimento

Paciente e metódico. Sua filosofia é simples: você não invade aquilo que
não entende.

### ROOT --- pós-acesso

Sempre pensa no alcance de uma entrada: 'Entramos onde?' e depois 'O que
isso alcança?'.

### MUTE --- stealth

Paranoico com detecção, mudanças de comportamento, alertas e sinais de
que alguém percebeu a operação. É frequentemente o primeiro a recomendar
recuo.

## 3. Estrutura não linear e reação dinâmica

BREACH não depende de SEM ROSTO para existir. Se o protagonista já
trabalhou com membros da guilda em outra operação, eles reconhecem isso.
Se chega cedo, é tratado como alguém promissor. Se já possui grande
reputação, a guilda questiona se a técnica acompanha a fama. Se acumulou
detecções ou decisões imprudentes, MUTE e GHOST reagem a esse histórico.

A mesma missão pode ter introduções e falas diferentes conforme o save,
sem alterar seu objetivo central.

### Exemplos de reação

> GHOST, para um jogador pouco conhecido: "Quero ver até onde você
> consegue entrar."

> GHOST, para alguém já respeitado: "Seu nome circula. Aqui isso não
> abre porta nenhuma."

> MUTE, diante de histórico imprudente: "Eu não quero ele perto desse
> ambiente."

SEM ROSTO pode inclusive ocorrer antes ou depois do contato com BREACH.
Se o protagonista já conhece a guilda, pode sugeri-la quando LEAK
precisar de especialistas; caso contrário, o contato pode acontecer por
outra rota do mundo.

## 4. Estrutura do arco

| ID \| Missão \| Contexto \| Lição / conflito \|

| --- \| --- \| --- \| --- \|

| B1 \| DEFACE \| Prova inicial de intrusão \| Entrar não justifica
  vandalizar. \|

| B2 \| BUG BOUNTY \| Programa autorizado \| Provar impacto sem exceder
  o necessário. \|

| B3 \| PENTEST \| Teste contratado \| Cumprir objetivo, escopo e
  adaptar a estratégia. \|

| B4 \| INTRUSÃO REAL \| Operação sem autorização \| Aplicar a
  disciplina sem a proteção de um contrato. \|

| BF \| SALA DE EVIDÊNCIAS \| Infraestrutura policial fictícia \|
  Conseguir exatamente um arquivo e encarar o peso do que foi entregue.
  \|

## B1 --- DEFACE

Função: apresentar a filosofia da BREACH por meio de um erro do
protagonista.

GHOST entrega um alvo relativamente simples e pede que o protagonista
demonstre até onde consegue chegar. A instrução é deliberadamente curta.
O jogador encontra uma entrada, consegue acesso ao servidor web e decide
provar sua presença alterando a página.

    OWNED BY [NICK]

Ele publica a prova esperando impressionar a guilda. A reação é o
oposto.

> GHOST: O que você fez? PROTAGONISTA: Entrei. GHOST: Eu vi.
> PROTAGONISTA: Então? GHOST: Por que alterou o site?

> GHOST: Você encontrou uma porta aberta e, em vez de descobrir até onde
> ela levava, escreveu seu nome nela.

BREACH considera o acesso tecnicamente válido, mas a operação ruim. O
protagonista causou impacto desnecessário, alertou o administrador e
destruiu a discrição da própria entrada.

> PROTAGONISTA: Mas eu consegui entrar. GHOST: Essa era a parte fácil.

B1 estabelece que a guilda não mede capacidade pelo espetáculo. A
repreensão é real; não é um elogio disfarçado.

## B2 --- BUG BOUNTY

Função: mostrar o extremo oposto do deface. Desta vez existe
autorização, regras e um programa de recompensa.

O protagonista recebe um alvo dentro de um programa fictício de bug
bounty. O objetivo não é comprometer o máximo possível, mas localizar
uma vulnerabilidade válida, compreender o impacto e produzir uma
demonstração mínima que permita à empresa reproduzir e corrigir o
problema.

> GHOST: Qual é o objetivo? PROTAGONISTA: Encontrar a falha. GHOST: Não.
> Encontrar, provar e parar.

A missão ensina que uma vulnerabilidade séria não exige roubar uma base
inteira para ser demonstrada. O jogador pode ser tentado a avançar além
do necessário, mas isso reduz a qualidade da operação e pode violar o
escopo fictício.

Ao final, o relatório é enviado e a recompensa depende não apenas da
severidade, mas também da qualidade da demonstração e do respeito ao
escopo.

## B3 --- PENTEST

Função: transformar intrusão em trabalho profissional com objetivo
definido.

Uma empresa fictícia contrata BREACH para testar uma parte de sua
infraestrutura. Ela acredita que determinadas barreiras impedem que um
invasor externo alcance um ativo interno específico.

O trabalho tem objetivo e escopo claros. O protagonista precisa
reconhecer o ambiente, escolher uma rota e demonstrar se o ativo pode ou
não ser alcançado.

Durante a operação, uma abordagem pode começar a apresentar sinais de
detecção. Surge a opção de ABORTAR.

    ABORT ATTEMPT

Abortar encerra aquela tentativa, não a missão. O jogador preserva a
operação, reorganiza informações e busca outra abordagem.

> GHOST: Eu não disse para desistir. Disse para sair.

A missão só termina quando o objetivo contratado é concluído ou quando a
conclusão técnica prevista pelo cenário é demonstrada. O foco é
adaptação, não insistência cega.

## B4 --- INTRUSÃO REAL

Função: retirar a proteção dos contextos profissionais e testar se a
disciplina permanece quando ninguém autorizou a presença da guilda.

BREACH recebe um trabalho underground com um objetivo operacional
específico. Não existe interesse em publicar documentos como LEAK, nem
um contrato formal de pentest protegendo a equipe. O valor da missão
está em alcançar um recurso definido sem transformar a operação numa
pilhagem indiscriminada.

O protagonista precisa aplicar tudo que aprendeu: reconhecimento,
objetivo, controle de impacto, decisão de recuar quando uma rota fica
comprometida e conclusão obrigatória do trabalho por outro caminho.

B4 prepara o jogador para a final: a guilda já demonstrou que consegue
entrar. Agora o problema passa a ser compreender o que significa aceitar
certos trabalhos.

## BF --- SALA DE EVIDÊNCIAS

Categoria: intrusão institucional. Escala: alta. Esta é a final do
primeiro arco da BREACH. A instituição policial é fictícia e inspirada
apenas na ideia geral de uma polícia federal.

### O cliente

Um contato underground procura BREACH com um pedido estranhamente
específico. Meses antes, um notebook foi apreendido numa grande operação
policial. Dentro dele existia um arquivo criptografado chamado
ledger.enc.

O cliente não quer acesso permanente, banco de dados ou sabotagem. Quer
uma cópia daquele arquivo exatamente como foi apreendido.

> GHOST: O que tem nele? CLIENTE: Não importa. GHOST: Para mim importa.
> CLIENTE: Eu não quero que vocês abram.

A proposta financeira é grande e o objetivo parece perfeitamente
delimitado: um arquivo.

### Localizar a evidência

O primeiro problema é descobrir onde a evidência digital foi armazenada.
O protagonista parte de informações sobre apreensão, data, procedimento,
equipamento e cadeia de custódia. Diferentes sistemas fictícios
registram etapas diferentes do caminho da evidência.

A missão não apresenta um diretório conveniente. O jogador reconstrói o
percurso até encontrar o registro correto.

    EVIDENCE ITEM
    DEVICE: NOTEBOOK
    FORENSIC IMAGE: CREATED
    STATUS: ARCHIVED
    ACCESS: RESTRICTED

Encontrar o registro não significa possuir o arquivo. O armazenamento de
evidências está mais fundo.

### O repositório

A operação progride por camadas abstratas de intrusão até o repositório
fictício de evidências digitais.

    CASE ███████

    IMAGE_01
    IMAGE_02
    EXTRACTED_FILES
    METADATA

Ali está ledger.enc.

Mas também existem evidências de inúmeras outras investigações. A
tentação final é justamente perceber o quanto poderia ser levado.

> GHOST: Achou? PROTAGONISTA: Sim. GHOST: Então acabou.

O terminal permanece aberto. O jogador pode obedecer e copiar apenas o
objetivo ou explorar além do necessário, aumentando risco e
ultrapassando o escopo que a própria BREACH estabeleceu.

    [ PEGAR ledger.enc ]
    [ CONTINUAR EXPLORANDO ]

### Conclusão técnica

Na execução mais disciplinada:

    ledger.enc
    TRANSFER COMPLETE

    FILES COPIED: 1
    FILES MODIFIED: 0

> GHOST: Sai.

A saída é parte da operação. Quando termina:

    SESSION CLOSED
    OBJECTIVE: COMPLETE

> GHOST: Um arquivo. PROTAGONISTA: Um arquivo.

A BREACH considera a operação um enorme sucesso técnico.

## 5. Consequência --- FORA DO ESCOPO

Alguns dias depois, enquanto o protagonista já pode estar envolvido em
qualquer outro arco do jogo, aparece uma notícia: uma pessoa ligada à
investigação da qual ledger.enc fazia parte foi encontrada morta. A
polícia investiga possível relação com uma organização criminosa.

O protagonista procura o cliente.

    USER NOT FOUND
    ACCOUNT DELETED

O pagamento continua em sua carteira. Não existe confirmação de que o
arquivo causou a morte, mas a coincidência é impossível de ignorar.

### Confronto com BREACH

> PROTAGONISTA: Vocês viram as notícias? HEX: Vi. PROTAGONISTA: E? HEX:
> E o quê?

GHOST reconhece a preocupação, mas insiste que não há prova de relação.

> GHOST: Você quer que eu diga que matamos aquele homem? PROTAGONISTA:
> Não. GHOST: Porque não sabemos. PROTAGONISTA: Também não sabemos que
> não.

Então surge a frase que muda o significado de todo o arco:

> GHOST: O conteúdo estava fora do escopo.

Durante as missões anteriores, 'escopo' significava disciplina. Agora o
protagonista percebe sua limitação moral: ninguém perguntou por que um
desconhecido estava disposto a pagar tanto por uma única evidência
policial.

### A discussão

> PROTAGONISTA: A gente invadiu a Federal por um cara que nem conhecia.
> GHOST: Sim. PROTAGONISTA: Pegamos uma evidência de uma investigação
> que nem sabíamos do que era. GHOST: Sim. PROTAGONISTA: E entregamos
> porque ele pagou.

> GHOST: Nós cumprimos o trabalho. PROTAGONISTA: Esse é o problema.

BREACH não vira vilã. MUTE pode argumentar que o cliente deveria ter
sido verificado melhor. ROOT questiona o que mudaria. HEX lembra que o
trabalho foi tecnicamente extraordinário. A guilda permanece dividida
apenas sobre a interpretação --- não se desfaz.

## 6. Saída do protagonista

O acontecimento faz o protagonista decidir que não quer continuar
aceitando trabalhos sob aquela filosofia.

> PROTAGONISTA: Estou fora. HEX: Fora do quê? PROTAGONISTA: Da BREACH.

Ghost pergunta se ele realmente está saindo por causa de uma morte que
talvez nem tenha relação com a operação.

> PROTAGONISTA: Não. GHOST: Então por quê? PROTAGONISTA: Porque eu
> entrei na Polícia Federal sem saber para quem estava trabalhando.

A conversa retorna à primeira missão.

> GHOST: Quando chegou aqui você alterou um site só para mostrar que
> conseguia entrar. PROTAGONISTA: Eu lembro. GHOST: Eu perguntei por
> quê. PROTAGONISTA: Eu não tinha resposta. GHOST: Agora tem?
> PROTAGONISTA: Tenho. GHOST: Qual? PROTAGONISTA: É por isso que estou
> saindo.

Ghost aceita. Não há expulsão, vingança nem destruição da guilda.

> GHOST: Certo. HEX: Só isso? GHOST: O que você quer que eu faça? A
> porta está aberta.

### Último momento

> HEX: Ei. PROTAGONISTA: Quê? HEX: Seu primeiro deface ainda foi uma
> merda. PROTAGONISTA: Vai tomar no cu. HEX: ❤️

O encerramento evita tom de formatura. O protagonista não sai porque
'aprendeu tudo'; sai porque um acontecimento concreto revelou uma
incompatibilidade entre a forma como BREACH delimita responsabilidade e
a forma como ele passa a enxergá-la.

## 7. Estado pós-arco

    BREACH
    STATUS: LEFT
    RELATION: NEUTRAL

BREACH continua existindo e operando no mundo. GHOST, HEX, TRACE, ROOT e
MUTE podem continuar aparecendo em fóruns, notícias, diálogos e
operações de outras guildas. Os contatos individuais permanecem
disponíveis quando fizer sentido narrativo.

Não surgem novas missões próprias da BREACH para o protagonista. Isso
não impede futuras colaborações pontuais.

> HEX: Achei que estava fora. PROTAGONISTA: Estou. HEX: Então quanto vai
> pagar?

Esse estado permite que BREACH participe de acontecimentos muito mais
avançados posteriormente sem reabrir seu arco de guilda.

## 8. Conteúdo reservado para mais tarde

A ideia de uma operação envolvendo uma ferramenta de vigilância
extremamente poderosa, zero-click/zero-day ou equivalente ao Pegasus NÃO
pertence a este arco. Ela fica reservada para uma etapa muito posterior
da história, quando esse nível de capacidade tiver peso proporcional ao
risco.

A final atual é SALA DE EVIDÊNCIAS justamente para evitar queimar esse
cartucho cedo.

## 9. Curva temática

-   B1 --- espetáculo: o protagonista confunde acesso com prova e faz um
    deface desnecessário.

-   B2 --- contenção: aprende que provar uma falha pode exigir
    pouquíssimo impacto.

-   B3 --- disciplina: aprende a trabalhar com objetivo, escopo e recuo
    tático.

-   B4 --- responsabilidade operacional: aplica a disciplina sem a
    proteção de autorização formal.

-   BF --- consequência: executa uma intrusão tecnicamente excelente,
    mas percebe que limitar o 'escopo técnico' não elimina
    responsabilidade sobre para quem se trabalha.

-   FORA DO ESCOPO --- escolha: BREACH continua; o protagonista decide
    sair.
