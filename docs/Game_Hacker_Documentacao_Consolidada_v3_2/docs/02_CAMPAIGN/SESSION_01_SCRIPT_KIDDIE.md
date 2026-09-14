# GAME HACKER - Sessao 1 / SCRIPT KIDDIE

> Documento canonico da Sessao 1. O arquivo tecnico antigo no `archive` e apenas referencia historica quando divergir deste documento.

---

# GAME HACKER --- SESSÃO 1: SCRIPT KIDDIE

## Versão consolidada

> **"Você sabe fazer funcionar. Agora precisa descobrir se realmente
> sabe o que está fazendo."**

------------------------------------------------------------------------

## 1. Ponto de partida

A Sessão 1 começa imediatamente depois do **Tutorial / V1**.

No tutorial, o protagonista migrou para o novo sistema, baixou um jogo
crackeado por P2P, conheceu peers e seeds, utilizou um scanner de
memória, encontrou valores e endereços, criou a V1, compartilhou com o
amigo, viu o arquivo viralizar, chegou ao fórum e conheceu BYTE, R4ZOR,
NULL e outros usuários.

NULL inicialmente considera `[NICKNAME]` apenas mais um **script
kiddie**. O protagonista cria sua conta no fórum para responder às
provocações, não para reivindicar publicamente a autoria.

O tutorial termina quando a desenvolvedora atualiza o jogo:

``` text
SECURITY UPDATE AVAILABLE
```

A V1 deixa de funcionar:

``` text
INCOMPATIBLE VERSION
TARGET MEMORY STRUCTURE CHANGED
```

Tela preta.

# SESSÃO 1 --- SCRIPT KIDDIE

------------------------------------------------------------------------

## 2. Regra estrutural da sessão

A sessão segue:

**Abertura → Liberdade → Convergência → Missão Final**

A última missão de cada sessão é **fixa, obrigatória e canônica**. Ela
encerra a sessão e cria o gancho da próxima.

Na Sessão 1, a missão final é:

# A GAROTA

Para liberá-la, o jogador precisa obrigatoriamente concluir:

``` text
V2_COMPLETE          = TRUE
WIFI_COMPLETE        = TRUE
PENDRIVE_COMPLETE    = TRUE
FORUM_JOB_COMPLETE   = TRUE
VEX_MAIN_COMPLETE    = TRUE
```

CTFs, perguntas, jobs secundários e o incidente condicional de VEX não
substituem esses requisitos.

------------------------------------------------------------------------

## 3. Fluxo da Sessão 1

``` text
TUTORIAL / V1
      │
      ▼
MISSÃO 1 — V2
      │
      ▼
FÓRUM / CTFs / CONHECIMENTO
      │
      ▼
EVENTO — MÃE DESLIGA O ROTEADOR
      │
      ├───────────────────────┐
      ▼                       ▼
SEM INTERNET             O PENDRIVE
      │                       │
      ▼                       │
INTERNET RESTAURADA           │
      │                       │
      ▼                       │
NOVO PEDIDO NO FÓRUM          │
      │                       │
      ▼                       │
PRIMEIRO TRABALHO             │
      │                       │
      ▼                       │
VEX APARECE                   │
      │                       │
      ▼                       │
ARCO PRINCIPAL DE VEX         │
      │                       │
      └───────────┬───────────┘
                  ▼
       REQUISITOS CONCLUÍDOS
                  │
                  ▼
          MISSÃO FINAL
            A GAROTA
                  │
                  ▼
          FIM DA SESSÃO 1
```

O jogador pode escolher, por exemplo:

-   Wi-Fi → Fórum → trabalho → VEX → Pendrive;
-   Pendrive → Wi-Fi → Fórum → trabalho → VEX.

Entre os marcos, pode realizar CTFs, responder perguntas, fazer jobs e
melhorar equipamento.

------------------------------------------------------------------------

# 4. MISSÃO 1 --- PATCHED / V2

## Descrição

A atualização destruiu a V1. Os endereços e estruturas não se comportam
mais como antes.

NULL continua provocando:

> **NULL:** cadê o misterioso `[NICKNAME]`

> **PROTAGONISTA:** Aqui.

> **NULL:** resolve aí script kid

> **PROTAGONISTA:** Salva essa mensagem.

> **NULL:** printado

## Objetivo

**Construir uma nova versão do hack compatível com a atualização.**

## Conceitos

-   memória de processos;
-   endereçamento;
-   ASLR;
-   análise estática e dinâmica;
-   fluxo de execução;
-   assembly introdutório;
-   injeção no processo simulado;
-   JMP;
-   Code Cave;
-   arquivos de save;
-   solução funcional × solução eficiente.

## Subobjetivos

### 1. Descobrir por que a V1 quebrou

``` text
TARGET FOUND
MEMORY ADDRESS INVALID
```

Ao reiniciar, os endereços mudam novamente.

### 2. Reanalisar o processo

Localizar novamente dinheiro, vida e munição e perceber que um endereço
fixo não é mais suficiente.

### 3. Analisar o executável

O protagonista encontra uma rotina conceitualmente semelhante a:

``` asm
LOAD current_ammo
SUB  current_ammo, 1
STORE current_ammo
```

### 4. Construir a solução do protagonista

Ele cria uma V2 muito mais sofisticada do que a V1, envolvendo sua
própria lógica durante a execução simulada.

Compila, testa, enfrenta crashes e corrige erros.

``` text
BUILD SUCCESSFUL
```

### 5. Publicar

``` text
[RELEASE] [GAME] — V2

Voltou.

[DOWNLOAD]
```

A comunidade confirma que funciona.

### 6. NULL analisa

> **NULL:** funciona

> **PROTAGONISTA:** Cadê o script kid agora?

> **NULL:** aqui

> **NULL:** tô falando com ele

NULL admite que ficou bom, mas considera a solução excessiva.

> **NULL:** até que foi bem

> **NULL:** só deu a volta no planeta pra atravessar a rua

### 7. JMP + Code Cave

NULL mostra que determinada rotina poderia ser desviada de forma muito
mais simples:

``` text
LOAD
 ↓
JMP ─────────────► CODE CAVE
                       │
                       ├─ lógica alterada
                       └─ retorno
 ↓
CONTINUA EXECUÇÃO
```

> **NULL:** pula daqui

> **NULL:** joga pro cave

> **NULL:** faz o que precisa

> **NULL:** volta

> **NULL:** acabou

### 8. A solução ainda mais simples

NULL manda olhar o save:

``` text
/save/profile_01.dat
```

Alguns recursos persistentes podem ser alterados ali.

> **NULL:** parabéns pela estrutura toda

> **NULL:** ficou bonita

> **NULL:** eu mudaria o save

> **PROTAGONISTA:** Filho da puta.

> **NULL:** ❤️

Para funções em runtime, a solução do protagonista ainda possui mérito.

A partir daqui NULL sabe que `[NICKNAME]` realmente criou os hacks,
embora ainda o considere inexperiente.

------------------------------------------------------------------------

# 5. O FÓRUM

Depois da V2, o fórum passa a ser parte central da progressão.

## O fórum oferece

-   **Conhecimento:** perguntas, respostas, discussões e CTFs.
-   **Reputação:** reconhecimento do nickname.
-   **Dinheiro:** pequenos jobs públicos.
-   **Desbloqueios:** áreas, desafios e trabalhos maiores.
-   **Pessoas:** usuários que podem virar contatos.

O fórum **descobre pessoas**. Ele não é o aplicativo onde
relacionamentos profissionais recorrentes acontecem.

------------------------------------------------------------------------

# 6. CONTATOS

Os contatos funcionam como um aplicativo de mensagens semelhante a um
WhatsApp dentro do jogo.

Fluxo:

``` text
FÓRUM
  ↓
REPUTAÇÃO
  ↓
OPORTUNIDADE
  ↓
TRABALHO
  ↓
CONFIANÇA
  ↓
CONTATO
  ├── novas missões
  ├── novos clientes
  └── novos contatos
```

Depois que um personagem passa seu contato, seus próximos trabalhos
chegam pelo aplicativo de mensagens.

Um contato pode ter dois, três, quatro ou mais trabalhos e pode indicar
o protagonista para outras pessoas.

------------------------------------------------------------------------

# 7. EVENTO --- A MÃE DESLIGA O ROTEADOR

Depois de algum tempo no computador:

> **MÃE:** Chega por hoje.

Ela desliga o roteador.

> **PROTAGONISTA:** Mãe...

> **MÃE:** Você tá nisso desde cedo.

Antes de sair:

> **MÃE:** Ah, e aproveita.

> **MÃE:** Seu tio deixou um pendrive aí na sua mesa.

> **PROTAGONISTA:** Pra quê?

> **MÃE:** Ele apagou umas coisas sem querer. Disse que você talvez
> consiga recuperar.

> **PROTAGONISTA:** Tá.

> **MÃE:** E sai um pouco desse computador depois.

Na tela:

``` text
NETWORK
OFFLINE
```

Duas missões ficam disponíveis:

``` text
[ SEM INTERNET ]
Encontrar uma maneira de voltar à Internet.

[ O PENDRIVE ]
Tentar recuperar os arquivos do seu tio.
```

Não existe ordem obrigatória entre elas.

------------------------------------------------------------------------

# 8. MISSÃO --- SEM INTERNET

## Objetivo

**Encontrar uma maneira de voltar à Internet.**

## Conceitos

-   redes sem fio;
-   autenticação;
-   canais;
-   BSSID;
-   tráfego;
-   WEP;
-   IV;
-   fragilidades criptográficas históricas;
-   análise PTW simulada.

## Subobjetivos

### 1. Confirmar a queda

``` text
DEFAULT GATEWAY: UNREACHABLE
```

### 2. Procurar redes próximas

O protagonista utiliza um adaptador wireless dentro da simulação.

### 3. Encontrar uma rede antiga

Uma rede próxima utiliza WEP.

### 4. Pesquisar o protocolo

O jogador consulta documentação dentro do jogo.

### 5. Coletar tráfego

``` text
IVs: 481
IVs: 2,114
IVs: 9,873
```

### 6. Analisar

``` text
ANALYZING RC4 STATISTICAL BIASES...
```

Até:

``` text
KEY RECOVERED
```

### 7. Conectar

``` text
CONNECTED
INTERNET ACCESS: ONLINE
```

O protagonista percebe que entrou na rede de outra pessoa sem
autorização.

É a primeira transgressão deliberada.

## Consequência

Quando a Internet volta:

``` text
3 NEW NOTIFICATIONS
1 NEW FORUM THREAD
```

O Wi-Fi desbloqueia o **primeiro trabalho do fórum**.

------------------------------------------------------------------------

# 9. MISSÃO --- O PENDRIVE

Pode ser feita antes ou depois do Wi-Fi.

## Objetivo

**Recuperar os arquivos apagados do pendrive do tio.**

## Conceitos

-   dispositivos;
-   partições;
-   filesystem;
-   exclusão lógica;
-   imagem forense;
-   file carving;
-   hashes;
-   integridade;
-   arquivos protegidos;
-   privacidade.

## Subobjetivos

### 1. Identificar o dispositivo

``` text
/dev/sdb
```

### 2. Preservar o original

Criar uma imagem do dispositivo antes de trabalhar nos dados.

### 3. Recuperar arquivos

``` text
RECOVERED FILES: 18
RECOVERED FILES: 43
RECOVERED FILES: 87
```

### 4. Encontrar um arquivo protegido

``` text
pessoal.zip
PASSWORD REQUIRED
```

O jogador já pode concluir o objetivo principal sem abrir esse arquivo.

### 5. Investigação opcional

Se a curiosidade vencer, o protagonista tenta descobrir a senha dentro
da simulação.

Ao abrir, encontra fotos, reservas, recibos e conversas.

> **MULHER:** Quando você vai contar pra ela?

> **TIO:** Não posso.

> **MULHER:** Você fala isso há três anos.

Ele percebe que o tio tem uma amante.

Não existe conspiração. É apenas um segredo humano que ele não deveria
conhecer.

O jogo registra o que o protagonista decide fazer com a informação.

Tema:

> **Ter acesso não significa ter direito de olhar.**

------------------------------------------------------------------------

# 10. MISSÃO --- PRIMEIRO TRABALHO DO FÓRUM

Só aparece depois do Wi-Fi.

## Contexto

Um usuário publica:

> **USER:** alguém consegue me ajudar com um servidor?

> **USER:** fiz merda aqui e não sei o que aconteceu

É um trabalho autorizado.

## Objetivo

**Descobrir por que um serviço parou e restaurá-lo.**

## Conceitos

-   serviços;
-   processos;
-   permissões;
-   logs;
-   troubleshooting;
-   configuração Linux.

## Subobjetivos

1.  Receber acesso ao servidor simulado.
2.  Verificar processos, serviços, armazenamento e logs.
3.  Encontrar uma configuração/permissão incorreta.
4.  Corrigir.
5.  Testar o serviço.

> **USER:** CARALHO

> **USER:** voltou

> **PROTAGONISTA:** era permissão

> **USER:** quanto?

> **PROTAGONISTA:** ?

> **USER:** pra te pagar

O protagonista ganha dinheiro e reputação.

Mais importante: outros usuários veem sua competência.

Um deles é VEX.

------------------------------------------------------------------------

# 11. VEX --- ENTRADA NA HISTÓRIA PRINCIPAL

VEX é obrigatório na Sessão 1.

Sua função é mostrar que **o fórum desbloqueia pessoas e pessoas
desbloqueiam missões maiores**.

Depois do trabalho anterior:

> **VEX:** vi o que você resolveu naquele post

> **PROTAGONISTA:** blz

> **VEX:** você manja de linux mesmo ou foi sorte?

> **PROTAGONISTA:** quer descobrir?

> **VEX:** kkkkk

> **VEX:** tô configurando um servidor do zero

> **VEX:** pago se quiser fazer

------------------------------------------------------------------------

# 12. VEX 1 --- DO ZERO

## Objetivo

**Configurar um servidor Linux funcional a partir de uma instalação
limpa.**

## Conceitos

-   usuários e grupos;
-   filesystem;
-   permissões;
-   pacotes;
-   serviços;
-   SSH;
-   HTTP;
-   firewall;
-   logs;
-   processos.

## Subobjetivos

1.  Verificar CPU, RAM, disco, interfaces e distribuição.
2.  Atualizar pacotes e dependências.
3.  Criar usuários e grupos.
4.  Configurar permissões.
5.  Configurar SSH.
6.  Instalar/configurar servidor web.
7.  Configurar firewall.
8.  Provocar um erro controlado.
9.  Usar logs para diagnosticar.
10. Restaurar o serviço.

Ao final:

> **VEX:** salva meu contato

``` text
NEW CONTACT
VEX
ONLINE
```

> **VEX:** esse fórum é uma zona

> **VEX:** quando tiver livre chama

> **VEX:** sempre tenho alguma coisa aparecendo

A partir daqui, VEX deixa de entregar trabalhos pelo fórum.

------------------------------------------------------------------------

# 13. VEX 2 --- PRODUÇÃO

Chega pelo aplicativo de contatos.

> **VEX:** tá ocupado?

> **PROTAGONISTA:** depende

> **VEX:** dinheiro

> **PROTAGONISTA:** tô livre

## Objetivo

**Preparar o servidor para produção.**

## Conceitos

-   hardening;
-   menor privilégio;
-   serviços desnecessários;
-   autenticação;
-   permissões;
-   firewall;
-   monitoramento;
-   atualização;
-   backup e restauração.

## Subobjetivos

1.  Auditar serviços.
2.  Revisar usuários e privilégios.
3.  Revisar permissões.
4.  Definir políticas de acesso.
5.  Configurar firewall.
6.  Configurar monitoramento.
7.  Identificar dados importantes.
8.  Criar backup.
9.  Verificar integridade.
10. Testar restauração.

O jogo salva a qualidade das decisões:

``` text
ROOT_LOGIN        = ENABLED / DISABLED
PASSWORD_POLICY   = WEAK / STRONG
UNUSED_SERVICES   = ACTIVE / DISABLED
FIREWALL          = PARTIAL / STRICT
BACKUP            = NONE / VALID
PERMISSIONS       = WEAK / SECURE
```

Esses valores podem permanecer ocultos.

Ao concluir VEX 2:

``` text
VEX_MAIN_COMPLETE = TRUE
```

------------------------------------------------------------------------

# 14. VEX --- INCIDENTE CONDICIONAL

Se o jogador configurou corretamente, não é obrigado a sofrer um ataque
só para cumprir roteiro.

VEX pode apenas avisar:

> **VEX:** tentaram alguma coisa ontem

> **PROTAGONISTA:** E?

> **VEX:** não conseguiram

> **PROTAGONISTA:** 😎

> **VEX:** não começa

Se uma configuração ruim relevante ficou aberta:

> **VEX:** tá aí?

> **VEX:** responde

> **VEX:** deu merda

Surge uma cadeia condicional.

## ALGUMA COISA ESTÁ ERRADA

### Objetivo

**Investigar o comprometimento.**

### Subobjetivos

1.  Verificar CPU/RAM.
2.  Identificar processos desconhecidos.
3.  Revisar usuários.
4.  Correlacionar autenticações e horários.
5.  Analisar logs.
6.  Comparar arquivos com backups.
7.  Descobrir o vetor de entrada.
8.  Perceber, se aplicável, que a origem foi uma decisão anterior do
    próprio jogador.

Depois:

``` text
SERVER COMPROMISED
```

## TAKE BACK CONTROL

### Objetivo

**Retomar o controle e restaurar um estado confiável.**

### Subobjetivos

1.  Conter.
2.  Preservar evidências.
3.  Identificar acessos indevidos.
4.  Remover persistências/contas indevidas dentro da simulação.
5.  Trocar credenciais.
6.  Restaurar arquivos.
7.  Corrigir a origem.
8.  Validar integridade.
9.  Voltar os serviços para produção.

Se o jogador havia configurado backup corretamente, a recuperação é mais
fácil.

Se não havia, sofre as consequências.

Essas missões são consequência, não requisito para liberar a missão
final.

------------------------------------------------------------------------

# 15. CTFs E CONTEÚDO OPCIONAL

CTFs permanecem disponíveis durante a sessão.

Categorias:

-   Web;
-   Crypto;
-   Forensics;
-   Reverse Engineering;
-   Linux;
-   Networking.

NULL aparece no ranking:

``` text
REDSHIFT CTF

1. NULL          9850
2. 0xVOID        8120
3. BYTE          7440
...
47. [NICKNAME]    380
```

> **PROTAGONISTA:** alguma dica?

> **NULL:** sim

> **PROTAGONISTA:** ?

> **NULL:** resolve

> **PROTAGONISTA:** vai tomar no cu

> **NULL:** ❤️

CTFs fornecem conhecimento, reputação, prática e desbloqueios, mas não
substituem a história principal.

------------------------------------------------------------------------

# 16. LIBERAÇÃO DA MISSÃO FINAL

Quando:

``` text
V2_COMPLETE          = TRUE
WIFI_COMPLETE        = TRUE
PENDRIVE_COMPLETE    = TRUE
FORUM_JOB_COMPLETE   = TRUE
VEX_MAIN_COMPLETE    = TRUE
```

a missão final fica disponível.

O jogo não mostra "FINAL MISSION UNLOCKED".

O protagonista está usando o computador normalmente.

Então:

``` text
NEW MESSAGE
AMIGO
```

> **AMIGO:** mano

> **AMIGO:** tá ocupado?

> **PROTAGONISTA:** pq

> **AMIGO:** preciso de um favor

# MISSÃO FINAL --- A GAROTA

------------------------------------------------------------------------

# 17. MISSÃO FINAL --- A GAROTA

## Objetivo

# DESCUBRA A VERDADE.

O amigo acredita que a namorada está traindo.

O jogo não diz qual ferramenta usar. O jogador precisa aplicar o que
aprendeu.

## Conceitos testados

-   investigação;
-   OSINT;
-   autenticação;
-   credenciais;
-   engenharia social simulada;
-   sessões;
-   arquivos;
-   metadados;
-   OPSEC.

## Subobjetivos

### 1. Investigar a garota

Reunir informações públicas dentro da Internet fictícia.

### 2. Identificar superfícies

Contas, serviços, dispositivos simulados, hábitos e informações
disponíveis.

### 3. Escolher uma abordagem

Existem múltiplas rotas dentro da sandbox.

### 4. Conseguir acesso

``` text
ACCESS GRANTED
```

### 5. Encontrar as mensagens

A traição é confirmada.

### 6. Contar ao amigo

> **PROTAGONISTA:** Você tava certo.

O amigo pede o acesso.

O protagonista hesita, mas entrega.

### 7. O amigo ultrapassa a linha

Ele entra na conta e publica:

> **SOU UMA PIRANHA TRAIDORA.**

> **PROTAGONISTA:** MANO

> **PROTAGONISTA:** QUE PORRA VOCÊ TÁ FAZENDO?

> **AMIGO:** Ela merece.

> **PROTAGONISTA:** Apaga isso!

> **AMIGO:** Por quê?

> **PROTAGONISTA:** PORQUE AGORA ELA SABE QUE ALGUÉM ENTROU

### 8. O rastro

O protagonista acredita que o episódio acabou.

Não acabou.

Por ainda ser inexperiente, deixou vestígios correlacionáveis durante a
invasão.

Nada acontece imediatamente.

Nenhuma revelação.

Nenhum ataque.

Internamente:

``` text
EVIDENCE_CREATED = TRUE
```

Esse rastro será encontrado muito mais tarde por NULL, que conseguirá
puxar o fio técnico da primeira invasão.

------------------------------------------------------------------------

# 18. FIM DA SESSÃO 1

A garota sabe que foi invadida, mas ainda não sabe quem fez.

O protagonista termina a sessão tendo aprendido a:

-   modificar software;
-   analisar memória;
-   compreender fluxo de execução;
-   administrar servidores;
-   trabalhar com usuários e permissões;
-   analisar logs;
-   configurar backups;
-   recuperar arquivos;
-   trabalhar com redes;
-   participar de CTFs;
-   ganhar reputação;
-   transformar fórum em oportunidades;
-   transformar oportunidades em contatos;
-   ganhar dinheiro com conhecimento técnico;
-   ultrapassar limites de privacidade.

Ele ainda não se considera criminoso.

A progressão psicológica da sessão foi:

``` text
CURIOSIDADE
    ↓
COMPETÊNCIA
    ↓
RECONHECIMENTO
    ↓
OPORTUNIDADE
    ↓
PRIMEIRO DINHEIRO
    ↓
PRIMEIRA TRANSGRESSÃO
    ↓
INVASÃO DE PRIVACIDADE
```

A sessão começou com um garoto tentando fazer seu cheat voltar a
funcionar.

Termina com uma pessoa real prejudicada por aquilo que ele aprendeu.

# FIM DA SESSÃO 1

------------------------------------------------------------------------

# 19. REGRAS CONSOLIDADAS PARA AS PRÓXIMAS SESSÕES

## Missão final

Toda sessão possui uma missão final:

-   fixa;
-   obrigatória;
-   canônica;
-   responsável por encerrar a fase atual;
-   responsável por criar o gancho da próxima sessão.

## Não linearidade

A liberdade existe no miolo da sessão:

-   contatos;
-   jobs;
-   CTFs;
-   fórum;
-   conhecimento;
-   reputação;
-   dinheiro;
-   upgrades.

## Fórum

**Conhecimento + Reputação + Descoberta + Oportunidades**

## Contatos

**Relacionamentos + Trabalhos recorrentes + Dinheiro + Novos
personagens**

## Consequências

O jogo não deve provocar artificialmente um problema que o jogador
evitou por competência.

Configurações e escolhas anteriores podem criar ou remover
acontecimentos futuros.

## Evolução da ajuda ao jogador

``` text
TUTORIAL
"Faça isso."

↓
INÍCIO
"Você provavelmente precisa fazer algo assim."

↓
INTERMEDIÁRIO
"Descubra como resolver."

↓
AVANÇADO
"Este é o alvo. Este é o objetivo."

↓
FINAL
"Se vira."
```

## Princípio técnico

> **Máximo realismo na aparência e na experiência. Máximo isolamento na
> execução.**

Tudo acontece exclusivamente dentro da infraestrutura virtual e fictícia
do Game Hacker.


---

# Integracao de continuidade - Mae

Durante a Sessao 1, inserir uma pequena mensagem cotidiana em momento de baixa tensao:

> **MAE:** filho vou trabalhar  
> tem comida na geladeira ❤️

O protagonista responde normalmente. A funcao e criar uma linha de base emocional para o afastamento progressivo das sessoes seguintes.
