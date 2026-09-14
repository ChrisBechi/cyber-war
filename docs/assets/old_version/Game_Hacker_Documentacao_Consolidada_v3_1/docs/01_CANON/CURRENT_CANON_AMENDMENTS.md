# GAME HACKER — Emendas Canônicas Consolidadas (V1 + V2)

Este arquivo registra as decisões mais recentes e deve ser lido **junto**, e não no lugar, da documentação detalhada anterior. Quando houver conflito de continuidade, estas emendas prevalecem; o conteúdo antigo permanece preservado em `99_SOURCE_VERSIONS/` para que nenhuma informação seja perdida.

## 1. Tio
O tio permanece somente como personagem/easter egg da missão do pendrive. A recuperação revela a pasta protegida e a amante. **Não existe ligação posterior do tio com Deep Web, máfia, fóruns ou submundo.**

## 2. Gregory
Gregory é amigo e âncora humana, não fixer nem fornecedor recorrente de crimes. Ele participa do gatilho inicial e da missão da garota, depois se assusta com a transformação do protagonista. Nunca trai deliberadamente `[NICKNAME]`; fala demais por culpa/ingenuidade e mais tarde retorna para ajudar na fuga.

## 3. A Garota
A garota é uma pessoa comum e determinada. **Ela não entende hacking, não aprende hacking e não vira investigadora técnica.** Ela busca respostas por meios humanos: conversa, observa, correlaciona fatos cotidianos e extrai de Gregory informações sutis sobre o amigo que ela não conhece — confiabilidade, proximidade, onde vive, nickname e contexto. Quando chega a uma identidade candidata, precisa de alguém tecnicamente capaz: NULL.

## 4. NULL e o rastro
A primeira invasão da garota ocorre quando o protagonista ainda é inexperiente e deixa vestígios correlacionáveis. NULL, muito mais experiente, consegue puxar esse fio técnico. A informação humana da garota e o rastro técnico convergem. O honeypot **não revela magicamente dados pessoais**: serve apenas para confirmar, de forma exata, que a identidade candidata opera `[NICKNAME]`. O `Te peguei` é a confirmação.

## 5. BYTE e DarkMatter
BYTE é o “Woozie” estrutural: amplia o mundo econômico, clientes, mercado, negociação e oportunidades. DarkMatter não o substitui; DarkMatter funciona melhor como recrutador/ponte para o mundo das guildas.

## 6. Personagens secundários
O jogo deve possuir personagens de arco fora da história principal. Eles podem oferecer 2–5 trabalhos, crescer paralelamente, reaparecer, mudar de guilda, ser presos, desaparecer ou nunca mais voltar. Nem todo personagem secundário deve convergir para a trama principal.

## 7. Atividades criminosas opcionais
Além de contratos, o protagonista pode anunciar serviços e construir operações próprias dentro do universo fictício. Isso cria renda, reputação, contatos, evidências e heat. Exemplos narrativos incluem scams simulados, páginas/anúncios/promessas falsas, mercado de dados, access brokering, cracking e serviços underground. O foco é simulação de risco, persona, credibilidade, consequência e economia — não instrução operacional para fraude real.

## 8. Reconhecimento
WHOIS e pesquisa avançada/“Google Dorks” simulada passam a integrar a progressão de reconhecimento, junto com OSINT, footprinting, metadados, DNS, subdomínios, scanning e enumeração.

## 9. CVEs, exploits e atualização
Algumas CVEs históricas reais permanecem no design por valor educacional, especialmente para demonstrar patch management e a importância de atualização. Exploits reais podem inspirar puzzles e laboratórios fictícios, mantendo contexto, impacto e defesa sem transformar o produto em ferramenta contra sistemas externos.

## 10. Zero-day e bug bounty
Pesquisa de vulnerabilidades ganha arco próprio. O jogador pode encontrar vulnerabilidades desconhecidas no universo do jogo e encarar escolhas entre responsible disclosure/bug bounty, retenção ou mercado underground. Isso reforça o contraste moral com NULL.

## 11. Blockchain e máfia
Blockchain/Bitcoin não é um módulo solto: entra organicamente pela rota da máfia. A organização usa criptoativos; Irina conecta o jogador ao financeiro. A análise de fluxos, carteiras e relações ajuda `[NICKNAME]` a compreender a estrutura e preparar Project: EXIT.

## 12. Metodologia técnica
A gramática recorrente de gameplay é:
`Footprinting → Scanning → Enumeration → Vulnerability Analysis → Exploitation → Post-Exploitation → Evidence/Data Collection → Covering Tracks`.
No início o jogo orienta; no final apresenta apenas alvo e objetivo.

## 13. Novos domínios técnicos preservados
A documentação consolidada inclui: engenharia social, 2FA/recovery, VoIP, wireless avançado, Android/iOS, malware/rootkit/ransomware, forensics e malware analysis, APT como arco operacional, identidades/personas darknet, blockchain, ICS/SCADA e suporte remoto a physical security — sempre dentro da infraestrutura simulada do jogo.

## 14. Sistemas governamentais fictícios
Fases avançadas podem envolver equivalentes fictícios de educação, saúde, polícia/justiça e repositórios governamentais de vulnerabilidades. O peso é narrativo e sistêmico: demonstrar como acesso digital pode gerar mercados criminosos e consequências graves.

## 15. Regra de apresentação
Permanece absoluta: **o jogador nunca sai da tela do computador**. Eventos físicos chegam por mensagens, chamadas, CCTV, mapas, logs, notícias, arquivos, feeds e estado de conexão.

## 16. Confronto final com NULL — derrota, tentativa técnica e “Adeus”
Após o honeypot e o `Te peguei`, `[NICKNAME]` reage da forma que aprendeu durante toda a campanha: tenta atacar/invadir NULL novamente. NULL interrompe a tentativa e deixa claro que a disputa terminou: **“não adianta, acabou, essa guerra você já perdeu.”**

O protagonista, em desespero, pergunta **“quanto?”**, tentando comprar o silêncio de NULL. NULL se recusa a transformar a situação em negociação e demonstra que não irá se rebaixar. A conversa evolui para o blefe já canônico de NULL sobre entregar as informações à polícia. `[NICKNAME]` reage com ameaça, mas NULL não acredita que ele realmente atravessará essa linha. NULL encerra a relação com uma última mensagem — **“Adeus, [NICKNAME].”** — e fica offline.

Não há uma cadeia artificial de missões intermediárias entre o honeypot e a decisão extrema. A escalada acontece dentro do próprio confronto: **identidade confirmada → tentativa técnica fracassa → tentativa de comprar o silêncio fracassa → ameaça/blefe da polícia → “Adeus” → silêncio → paranoia/desespero → contratação da eliminação de NULL.** Para NULL, “Adeus” encerra a guerra; para o protagonista, o silêncio parece confirmar que sua vida está prestes a ser destruída.

## 17. Presença progressiva da Garota
A Garota continua sendo completamente leiga em tecnologia. Entre as Sessões 1 e 5, porém, sua determinação deixa rastros narrativos: perguntas em comunidades, pesquisas pelo nickname, conversas com pessoas e tentativas comuns de compreender quem poderia estar por trás da invasão. O protagonista pode encontrar alguns desses rastros sem saber que pertencem a ela. Isso funciona como foreshadowing e não como investigação técnica.

## 18. Gregory e a conversa completa
Quando a verdade sobre Gregory finalmente puder ser confrontada, existe espaço para ele mostrar ao protagonista a conversa completa com a Garota. O material deixa claro que Gregory falou demais e cometeu um erro grave, mas também que, ao perceber para onde as perguntas estavam levando, tentou defender o amigo e não quis entregá-lo. Isso preserva o cânone: **culpa e ingenuidade, não traição deliberada.**

## 19. Supply Chain como arco de operação
Ataques à cadeia de suprimentos podem existir como arcos avançados em ambientes totalmente simulados. A progressão narrativa é: **mapear ecossistema/fornecedores → identificar uma dependência → comprometer o fornecedor no ambiente fictício → afetar uma atualização/cadeia de confiança simulada → acompanhar a distribuição → perceber a escala inesperada.** O foco é a confiança entre organizações e as consequências, não instruções operacionais aplicáveis a fornecedores reais.

## 20. DDoS como capacidade estratégica
Botnet/DDoS não precisa existir apenas como uma sequência isolada de três missões. Dentro da simulação, a botnet pode se tornar um **ativo clandestino persistente** que cresce ao longo da campanha e possui capacidade, disponibilidade e risco. Em operações posteriores, DDoS pode ser empregado narrativamente como distração/cortina de fumaça para outro objetivo. O sistema permanece fictício e isolado.

## 21. Comandos fundamentais de rede
`ping`, `ipconfig` e `ifconfig` passam a fazer parte do vocabulário básico do jogo em ambientes simulados. `ping` introduz conectividade/latência; `ipconfig` reforça o passado do protagonista no Windows; `ifconfig` aparece na adaptação ao Kali/Linux. A troca acidental de `ipconfig` por `ifconfig` pode funcionar como humor no tutorial e, mais tarde, como callback em situação de estresse, lembrando que por trás de `[NICKNAME]` ainda existe o garoto que começou a campanha aprendendo o novo sistema.
