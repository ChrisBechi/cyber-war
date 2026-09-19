// Authored breadth pack. Each source has its own subject, voice, and local links.
// All places, organizations and events in these records belong to the campaign world.
export function editorialContent({ brands, entities, add, url, epoch }) {
  const p = (text) => ({ kind: 'paragraph', text });
  const h = (text) => ({ kind: 'heading', text });
  const link = (label, url) => ({ label, url });
  const sources = [
    [
      'trilhos',
      'Nos Trilhos',
      'nostrilhos.com',
      'EDITORIAL',
      'newspaper',
      '#294e69',
      'nt',
      'A cidade vista pela janela.',
      'Cidades',
      'office',
      [
        [
          'terminal',
          'Terminal da Estação reabre a biblioteca de viagem',
          'As estantes voltaram ao saguão leste depois da reforma do piso. A retirada dos livros acontece no balcão azul, sem interferir no embarque.',
          'O acervo reúne romances curtos, mapas antigos e relatos de moradores. A equipe registra a devolução em qualquer uma das três estações participantes.',
          'Quem precisa apenas consultar um título pode usar as mesas do saguão. O anúncio de plataformas continua audível nessa área.',
        ],
        [
          'horarios',
          'Como ler o quadro de partidas de Porto Claro',
          'O quadro separa horário previsto, plataforma e situação da viagem. Mudanças de plataforma aparecem na mesma linha, ao lado do destino.',
          'A linha Circular atende Estação, Mercado e Vila Norte. O ramal Aurora termina no Jardim das Oficinas e exige troca para chegar ao campus.',
          'Uma partida retirada do quadro não confirma o embarque. O histórico do atendimento deve ser consultado no balcão da estação.',
        ],
        [
          'oficina',
          'A oficina que conserva o primeiro bonde',
          'A restauração preservou marcas de reparos no piso de madeira. A equipe preferiu registrar cada intervenção a devolver ao veículo uma aparência sem uso.',
          'Cadernos antigos mostram substituições de bancos e ajustes nas portas. As peças retiradas ficam identificadas no depósito do museu.',
          'A visita passa por uma passarela lateral. O bonde permanece imobilizado e não participa da circulação regular.',
        ],
      ],
    ],
    [
      'quintal',
      'Quintal de Dentro',
      'quintaldedentro.com',
      'BLOG',
      'personal',
      '#567342',
      'qd',
      'Um caderno para observar o que cresce.',
      'Jardinagem',
      'house',
      [
        [
          'sombra',
          'A varanda que mudou de luz',
          'No inverno, o prédio vizinho faz sombra sobre quase toda a minha varanda. Passei uma semana anotando onde a luz chegava antes de trocar os vasos de lugar.',
          'As fotografias tiradas sempre no mesmo horário mostraram melhor a mudança do que minha memória. Marquei os pontos de manhã, perto do almoço e no fim da tarde.',
          'Não encontrei um lugar perfeito para tudo. Separei as plantas por necessidade de luz e mantive as mais sensíveis perto da janela interna.',
        ],
        [
          'etiquetas',
          'Etiquetas que sobreviveram à chuva',
          'Depois de perder três nomes de mudas, usei etiquetas reaproveitadas e escrevi a data no verso. O caderno guarda o nome completo e a origem de cada uma.',
          'As anotações não resolvem uma identificação incerta. Duas mudas continuam registradas pelo nome provisório até que apareçam características suficientes para compará-las.',
          'A etiqueta presa ao vaso acompanha a planta quando mudo a disposição da varanda. Colar só na prateleira era uma forma rápida de confundir tudo.',
        ],
        [
          'trocas',
          'A pequena feira de mudas da Vila Norte',
          'A feira começou com duas mesas e cresceu quando os vizinhos trouxeram histórias sobre os quintais antigos. Cada muda saiu com uma anotação de procedência.',
          'Uma participante levou fotografias da planta adulta, o que ajudou a explicar seu porte. Outra preferiu oferecer sementes acompanhadas da data da coleta.',
          'O grupo combinou uma nova troca no mesmo salão. A lista de interesse fica no mural, junto das perguntas que ainda precisam de resposta.',
        ],
      ],
    ],
    [
      'lente',
      'Lente Aberta',
      'lenteaberta.com',
      'EDITORIAL',
      'tech',
      '#794f35',
      'la',
      'Fotografia com tempo para olhar.',
      'Fotografia',
      'film',
      [
        [
          'janela',
          'Retratos com a luz da janela',
          'A oficina do Mercado usou uma janela lateral e uma cortina clara. Em vez de trocar a câmera, cada participante mudou a posição da cadeira e comparou as sombras.',
          'A luz próxima da janela marcou um lado do rosto; mais longe, o contraste diminuiu. Um cartão branco ajudou a devolver parte da luz ao lado mais escuro.',
          'As imagens foram comparadas com o mesmo enquadramento. Manter uma variável por vez tornou mais fácil explicar por que o resultado mudava.',
        ],
        [
          'arquivo',
          'Um arquivo de fotos que permite encontrar as pessoas',
          'O coletivo começou renomeando pastas por data e assunto. Os nomes das pessoas ficaram em uma planilha separada, junto da autorização para publicação.',
          'Fotos parecidas podem ter permissões diferentes. A equipe passou a conferir a autorização da imagem escolhida, sem presumir que uma sessão inteira estava liberada.',
          'As cópias para divulgação ficam em uma pasta própria. Os originais continuam preservados com os arquivos de edição.',
        ],
        [
          'contato',
          'Folhas de contato antes da seleção final',
          'Ver todas as tentativas juntas mudou a seleção do ensaio da estação. Uma fotografia isolada escondia a sequência de decisões que levou ao enquadramento final.',
          'A folha de contato reúne miniaturas com nomes de arquivo. Os participantes marcam perguntas e alternativas antes de discutir nitidez ou tratamento de cor.',
          'A seleção final ficou menor, mas mais coerente. Algumas imagens tecnicamente corretas repetiam informações que já apareciam melhor em outra fotografia.',
        ],
      ],
    ],
    [
      'palco',
      'Palco de Bairro',
      'palcodebairro.com',
      'EDITORIAL',
      'newspaper',
      '#873e57',
      'pb',
      'A cultura acontece perto.',
      'Cultura',
      'radio',
      [
        [
          'ensaio',
          'O coro da Vila Horizonte abre o ensaio',
          'O grupo convidou moradores para acompanhar o trabalho de preparação, com pausas para explicar como as vozes entram em momentos diferentes.',
          'O ensaio aberto acontece no salão da associação. As cadeiras ficam afastadas da área de passagem e o público pode chegar entre uma peça e outra.',
          'A apresentação final será anunciada depois da revisão do repertório. A coordenação preferiu não confundir o ensaio de trabalho com um concerto pronto.',
        ],
        [
          'cartazes',
          'Cartazes guardam a história do Teatro Aurora',
          'Uma coleção de cartazes revelou mudanças no nome do teatro, no preço do ingresso e no desenho das sessões. Alguns exemplares ainda trazem correções feitas à mão.',
          'A exposição apresenta reproduções ampliadas e mantém os originais em suportes protegidos. O catálogo informa a origem e as lacunas de cada registro.',
          'Datas repetidas nem sempre significam a mesma montagem. Programas e notícias de época ajudaram a separar temporadas e apresentações avulsas.',
        ],
        [
          'sonoplastia',
          'Os objetos que viram sons no palco',
          'Uma caixa de madeira, folhas secas e uma chapa metálica fazem parte da bancada de sonoplastia da companhia. O efeito depende tanto do gesto quanto do objeto.',
          'Durante a oficina, o público vê como a distância do microfone altera a gravação. Pequenas ações são repetidas para comparar textura e duração.',
          'A companhia guarda anotações sobre cada combinação. O registro permite reconstruir um efeito quando outra pessoa assume a operação.',
        ],
      ],
    ],
    [
      'mapas',
      'Atlas de Porto Claro',
      'atlasportoclaro.com',
      'ENCYCLOPEDIA',
      'reference',
      '#4b6884',
      'ap',
      'Lugares, percursos e memória.',
      'Lugares',
      'house',
      [
        [
          'mercado',
          'Mercado Central de Porto Claro',
          'O Mercado Central ocupa a quadra entre a Rua das Oficinas e a Praça da Estação. O pátio leste reúne alimentação; o corredor oeste concentra bancas e pequenos serviços.',
          'As entradas da praça e da Rua das Oficinas estão no mesmo nível do passeio. A ligação com o terminal passa pelo saguão da biblioteca de viagem.',
          'Reformas alteraram a numeração de algumas bancas. Cadastros antigos devem ser comparados com o corredor e o nome do estabelecimento.',
        ],
        [
          'vila-norte',
          'Vila Norte: o bairro além da linha',
          'Vila Norte cresceu em torno das oficinas ferroviárias. A associação de moradores ocupa o antigo armazém de ferramentas, próximo à parada Circular.',
          'A praça possui um mural com atividades do bairro e uma feira mensal de trocas. O salão recebe o grupo de jardinagem e oficinas de pequenos reparos.',
          'O bairro é separado do campus pelo canal. A passagem de pedestres fica ao lado da ponte nova, e não no antigo acesso de serviço.',
        ],
        [
          'santa-aurora',
          'Santa Aurora e o Jardim das Oficinas',
          'Santa Aurora abriga o Teatro Aurora e o Jardim das Oficinas, onde termina o ramal de passageiros. A praça do teatro liga as duas áreas por uma passagem coberta.',
          'As ruas próximas ao jardim conservaram nomes associados a antigas profissões. Fotografias municipais mostram ocupações anteriores nos mesmos terrenos.',
          'O campus fica fora do perímetro do jardim. Endereços que usam os dois nomes precisam informar também o número do edifício.',
        ],
      ],
    ],
  ];
  for (const [id, name, domain, platform, layout, accent, mark, tagline, category] of sources) {
    brands.push({
      id,
      name,
      domain,
      platform,
      layout,
      accent,
      mark,
      tagline,
      voice: `${name}: relatos específicos, fontes identificadas e vocabulário de ${category.toLowerCase()}.`,
      navigation: [],
    });
  }
  for (const [id, name, , , , , , tagline, category, visual, articles] of sources) {
    entities.push({
      id: `topic-${id}`,
      name: category,
      kind: 'Topic',
      aliases: [name],
      description: tagline,
      relations: [],
    });
    for (const [slug, title, summary, body, end] of articles) {
      add(
        id,
        `/leitura/${slug}`,
        title,
        summary,
        category,
        [p(summary), h('Mais de perto'), p(body), h('Para continuar'), p(end)],
        undefined,
        {
          author: `Redação ${name}`,
          entities: [`topic-${id}`],
          visual,
          publishedAt: epoch - 86400 * 2,
          links: [link('Explore a região no Atlas', url('mapas', '/leitura/mercado'))],
        },
      );
    }
  }
}
