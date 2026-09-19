# M1C.4 — VFS events/watch e tail

A certificação, os resultados finais e o motivo de parada estão no
[relatório do milestone](../milestones/m1c4-report.md). Este documento descreve
os contratos e a arquitetura; não substitui a evidência executada.

## Eventos e identidade do VFS

`vfs/events.rs` fornece assinaturas tipadas por inode ou caminho. O `NodeTable`
emite eventos nos pontos centrais de inserção/remoção de entradas e alteração
de inode, compartilhados por handles, editores, programas e serviços.

Cada assinatura mantém uma revisão monotônica e no máximo um evento pendente.
O evento combina conteúdo, metadados e namespace e preserva o menor tamanho
observado em truncamentos. O consumidor decide como interpretar esse histórico:
`tail` compara o tamanho atual com seu offset, como demonstrado no GNU, inclusive
quando truncate + append ocorrem antes de ele voltar a executar.

Os índices por inode/caminho têm ordem determinística. Hardlinks compartilham
notificações. Mudanças nos ancestrais atingem os caminhos dependentes; leitura,
atime e bookkeeping de uma entrada irmã não acordam consumidores sem relação.
Confirmar uma revisão antiga não apaga um evento posterior.

Assinaturas não mantêm inodes vivos. Entradas e handles determinam sua duração.
O resolvedor de dependências usa o lookup POSIX compartilhado, incluindo symlinks,
`..`, componentes ausentes e falhas de permissão. Cada VFS admite 1024 watchers.

## Suspensão, transações e isolamento

`shell/wait.rs` combina revisões do VFS, vida de processos, entrada e timers
virtuais. O Control usa uma geração de notificações para evitar perder mudanças
entre observar o estado e armar a espera. A identidade do mundo filtra eventos
externos ao consumidor.

`shell/cooperative.rs` preserva a pilha do interpretador enquanto transfere o
mundo ao coordenador. Cada trecho executa sob a transação do serviço; a espera
libera seu mutex. O próximo trecho recebe o mundo atualizado, preservando os
contextos de terminal e as alterações confirmadas de outras sessões.

Notificações são publicadas após commit. A geração do serviço impede que um
worker antigo publique sobre um mundo carregado. Falhas de SQL cancelam o worker
e liberam somente recursos de runtime; não restauram conteúdo de snapshots
antigos. Sinais entre sessões pertencem ao estado virtual confirmado.

## Tempo virtual e save/load

Timers têm dono, prazo em ticks e cancelamento, com limite de 1024. Sua semântica
não consulta relógio host, FPS ou `thread::sleep`. Quando todos os stages estão
bloqueados, o scheduler pode avançar ao próximo prazo finito.

Arquivos regulares observáveis suspendem por eventos. Symlinks/retry que exigem
reverificação usam o timer compartilhado; intervalo e contador de unchanged
stats determinam o prazo. Os testes incluem um prazo virtual de quatro segundos
com `-s2` e `--max-unchanged-stats=1`, além de writers com intervalos distintos.

Handles, assinaturas, waits, timers e sinais pendentes não são serializados.
Reset/load limpa esses recursos; os comandos da aplicação cancelam os terminais
no carregamento/restauração. Cada mundo carregado recebe nova identidade.
Comandos bloqueados não são retomados a partir de um save.

## Contrato de tail

O handler usa o scheduler de streams compartilhado e o VFS virtual. Arquivos
regulares buscam sufixos a partir do fim; entradas sem seek usam retenção
incremental. A seleção `+N` mantém estado entre blocos e eventos. Os bytes não
sofrem conversão semântica para UTF-8.

A matriz cobre opções GNU, multiplicadores, overflow, sintaxe histórica, `-z`,
headers, quiet/verbose, erros, help/version, descritores, redirecionamentos,
pipelines, sinais e TTY. Descriptor follow conserva o inode aberto; name follow
resolve o nome novamente e reconhece substituição, remoção, recriação e
permissões. `--pid` observa escritores virtuais, incluindo término por sinal.

O stdout redirecionado usa a abstração compartilhada `shell/stdio.rs` para o
buffering observável da baseline Alpine/musl, incluindo autoappend controlado.
A baseline permanece GNU Coreutils 9.7, locale C e os mesmos pacotes fixados.

Limites intencionais: retenção de sufixo de 4 MiB, captura total de saída de
4 MiB (1024 bytes reservados ao diagnóstico) e arquivos binários de 32 MiB.
Os testes exercitam limite - 1, limite e limite + 1. Esses limites, os processos
e dispositivos virtuais definem o escopo; job control geral e novos dispositivos
permanecem fora deste milestone.

## Harness e evidência

A matriz contém 306 requests únicos: 268 do gerador base, 27 do gerador follow e
11 direcionados. O protocolo TTY é versão 1; o protocolo follow é versão 2.
O formato da captura canônica de tail é versão 4.

As interações incluem barreiras explícitas de bytes/espera, mutações estruturadas,
lotes de mutações com o consumidor suspenso e writers controlados. Timeout falha.
O schema rejeita operações incompletas, caminhos fora da fixture, barreiras
ambíguas e escritores inválidos. O adaptador DEV usa o mesmo handoff do jogo,
com observação binária independente da apresentação textual do terminal.

A referência exige dois runs idênticos e verificação independente. A comparação
inclui stdout/stderr, status/sinal, término, observações intermediárias, arquivos,
modos, caminhos removidos e relações de inode. Os fingerprints vinculam requests,
fontes compartilhadas, contratos e harnesses. Evidência obsoleta bloqueia promoção.

VFS.EVENTS e VFS.WATCH vinculam os 39 casos canônicos do protocolo follow. Sua
prontidão é calculada por evidência GNU atual, testes vinculados, contratos,
flags, ausência de lacunas required e Host Guard. A declaração permanece PARTIAL;
o pipeline calcula READY somente quando as condições passam.

O manifesto de tail também vincula PROCESS.LIFETIME, SIGNALS.STREAMS e
TTY.CANONICAL_IO às evidências de writers, sinais e TTY. São capacidades
específicas da implementação existente; os subsistemas gerais PROCESS, SIGNALS
e TTY permanecem PARTIAL. O vínculo conserva todos os requisitos VFS da família
Coreutils. Sem declaração específica, outros comandos continuam dependendo do
subsistema geral, conforme o teste de regressão do discovery.

O CI verifica GNU e strict tail, além dos seis comandos já certificados.
Tooling, probes e o ambiente GNU são exclusivos de desenvolvimento. Os probes
históricos M1C.3 foram preservados; resultados exploratórios não certificam a
entrega. Consulte o relatório para contagens derivadas, builds e próxima fila.
