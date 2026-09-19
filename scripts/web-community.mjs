// Authored people recur across the professional network and local classifieds.
export function communityContent({ brands, entities, docs, add, url, epoch }) {
  const link = (label, url) => ({ label, url });
  const p = (text) => ({ kind: 'paragraph', text });
  for (const [id, name, domain, platform, layout, accent, mark, tagline] of [
    [
      'linkup',
      'LinkUp',
      'linkup.com',
      'SOCIAL',
      'social',
      '#156c74',
      'in',
      'Conexões que têm história.',
    ],
    [
      'feiralivre',
      'FeiraLivre',
      'feiralivre.com',
      'CLASSIFIEDS',
      'classifieds',
      '#9a4d21',
      'fl',
      'Perto de você. Pronto para outra história.',
    ],
    [
      'memoria',
      'Memória Web',
      'memoria.web',
      'ARCHIVE',
      'archive',
      '#665847',
      'mw',
      'O que a rede guardou.',
    ],
  ])
    brands.push({
      id,
      name,
      domain,
      platform,
      layout,
      accent,
      mark,
      tagline,
      voice: 'Pessoal, objetivo e atento ao contexto.',
      navigation: [],
    });
  const people = [
    [
      'nara-campos',
      'Nara Campos',
      'engenharia de redes',
      'Porto Claro',
      'nexora',
      'router',
      'Após três meses revisando o manual do R4, percebi que a pergunta mais útil vinha de quem instalava o primeiro roteador. Reorganizamos a documentação começando pela posição do equipamento, antes das opções avançadas.',
    ],
    [
      'renata-avelar',
      'Renata Avelar',
      'jornalismo de tecnologia',
      'Santa Aurora',
      'techbyte',
      'phone',
      'Uma bateria que dura no laboratório nem sempre acompanha um dia de trabalho. Na próxima análise quero mostrar as condições de cada teste: brilho, sinal e tarefas realizadas. Sem isso, comparar números é comparar rotinas diferentes.',
    ],
    [
      'iara-mendonca',
      'Iara Mendonça',
      'educação e programação',
      'Vila Horizonte',
      'educa',
      'javascript',
      'Hoje uma aluna resolveu um exercício descrevendo primeiro a entrada e a saída. Só depois escreveu a função. Guardei esse percurso para a próxima aula: explicar a transformação costuma ser mais útil do que decorar a sintaxe.',
    ],
    [
      'lia-moreira',
      'Lia Moreira',
      'pesquisa e escrita',
      'Porto Claro',
      'blog',
      'radio',
      'Encontrei um rádio antigo numa feira. O vendedor lembrava do programa que ouvia com o pai; não lembrava o ano de fabricação. Anotei as duas coisas separadamente. Memória afetiva e ficha técnica contam histórias que podem se encontrar sem se substituir.',
    ],
    [
      'dora-nascimento',
      'Dora Nascimento',
      'cozinha e oficinas',
      'Vila Horizonte',
      'cozinha',
      'pizza',
      'Fizemos a mesma massa em dois dias de temperaturas diferentes. A segunda cresceu muito antes do horário previsto. Atualizei minhas anotações para descrever o aspecto da massa, além do tempo de descanso.',
    ],
    [
      'caio-fontes',
      'Caio Fontes',
      'documentação técnica',
      'Santa Aurora',
      'nexora',
      'linux',
      'Documentação também precisa de revisão por alguém que não participou do projeto. Um caminho que parece óbvio para quem escreveu pode ser a primeira dúvida de quem acabou de chegar.',
    ],
  ];
  for (const [index, [slug, name, role, location, company, topic, text]] of people.entries()) {
    const entityId = `person-${slug}`;
    entities.push({
      id: entityId,
      name,
      kind: 'Person',
      aliases: [name],
      description: `${name} trabalha com ${role} em ${location}.`,
      relations: company === 'nexora' ? [{ kind: 'WORKS_AT', target: 'nexora' }] : [],
    });
    const profileId = `web-linkup-pessoas-${slug}`;
    add(
      'linkup',
      `/pessoas/${slug}`,
      name,
      `${role} · ${location}`,
      'Pessoas',
      [p(`${name} compartilha projetos e experiências de ${role}.`)],
      {
        kind: 'socialProfile',
        bio: `Meu trabalho envolve ${role}. Aqui reúno projetos públicos, perguntas e aprendizados do cotidiano.`,
        location,
        role,
        employer: link(brands.find((b) => b.id === company).name, url(company)),
        following: [`web-linkup-pessoas-${people[(index + 1) % people.length][0]}`],
      },
      {
        author: name,
        entities: [entityId, topic],
        links: [link('Projetos e publicações', url(company))],
        publishedAt: epoch - 86400 * 3,
      },
    );
    add(
      'linkup',
      `/publicacoes/${slug}-caderno`,
      `${name}: anotações do trabalho`,
      text,
      'Publicações',
      [p(text)],
      { kind: 'socialPost', profileId, likes: 12 + index * 9 },
      {
        author: name,
        entities: [entityId, topic],
        links: [
          link(`Perfil de ${name}`, url('linkup', `/pessoas/${slug}`)),
          link('Leituras relacionadas', url(company)),
        ],
        publishedAt: epoch - 7200 + index * 60,
        comments: [
          {
            id: `reply-${slug}`,
            author: people[(index + 1) % people.length][1],
            text: [
              'Esse cuidado com a ordem das instruções fez diferença quando preparei uma oficina.',
              'Gostaria de ver também como o sinal fraco altera o resultado.',
              'Descrever a entrada antes de abrir o editor funcionou comigo.',
              'O relato do vendedor merece ficar junto das fotos da peça.',
              'O ponto da massa conta mais do que o cronômetro em dias quentes.',
              'Testei o roteiro com alguém de outra equipe e encontramos duas etapas ausentes.',
            ][index],
            publishedAt: epoch - 3600 + index * 60,
            rating: null,
          },
        ],
      },
    );
  }
  for (const [slug, name] of people) {
    for (const doc of docs.filter(
      (d) => d.brandId !== 'linkup' && d.author === name && d.publishedAt >= epoch - 86400 * 3,
    )) {
      doc.links.push(link(`Perfil profissional de ${name}`, url('linkup', `/pessoas/${slug}`)));
      if (!doc.entities.includes(`person-${slug}`)) doc.entities.push(`person-${slug}`);
    }
  }
  const about = docs.find((d) => d.brandId === 'nexora' && d.path === '/sobre');
  if (about) about.links.push(link('Equipe: Nara Campos', url('linkup', '/pessoas/nara-campos')));
  const listings = [
    [
      'radio-aurora',
      'Rádio Aurora de mesa',
      8500,
      'radio',
      'lia-moreira',
      'Porto Claro',
      'usado',
      'Rádio de mesa com marcas na madeira e seletor funcionando. A antena telescópica foi substituída. Pode ser testado na retirada.',
      'AVAILABLE',
    ],
    [
      'cadeira-oficina',
      'Cadeira da oficina de Iara',
      24000,
      'chair',
      'iara-mendonca',
      'Vila Horizonte',
      'usado',
      'Cadeira com altura regulável. Tecido limpo, pequeno desgaste no braço direito e rodízios trocados no último semestre.',
      'AVAILABLE',
    ],
    [
      'roteador-reserva',
      'NexLink R4 de reserva',
      19000,
      'router',
      'nara-campos',
      'Porto Claro',
      'usado',
      'Roteador usado em bancada, restaurado para as configurações de fábrica. Acompanha fonte e cabo de rede de um metro.',
      'AVAILABLE',
    ],
    [
      'livros-aula',
      'Livros de introdução à programação',
      6000,
      'book',
      'caio-fontes',
      'Santa Aurora',
      'usado',
      'Dois livros com exercícios resolvidos a lápis nas páginas finais. Índices fotografados na conversa para conferir o conteúdo antes da retirada.',
      'SOLD',
    ],
    [
      'forma-pizza',
      'Forma de pizza de 30 cm',
      3500,
      'pizza',
      'dora-nascimento',
      'Vila Horizonte',
      'novo',
      'Forma de alumínio sem uso, com 30 cm de diâmetro. Comprei duas para uma oficina e uma ficou na embalagem.',
      'AVAILABLE',
    ],
    [
      'apoio-notebook',
      'Suporte dobrável para notebook',
      4500,
      'chip',
      'renata-avelar',
      'Santa Aurora',
      'usado',
      'Suporte metálico dobrável com apoios de borracha. Cabe na mochila e mantém a base ventilada. Não acompanha computador.',
      'AVAILABLE',
    ],
  ];
  for (const [
    slug,
    title,
    price,
    visual,
    person,
    location,
    condition,
    description,
    status,
  ] of listings) {
    add(
      'feiralivre',
      `/anuncios/${slug}`,
      title,
      description,
      'Anúncios',
      [
        p(description),
        p(
          'A retirada é combinada na conversa. Confira o item antes de confirmar a entrega e registre qualquer diferença em relação à descrição.',
        ),
      ],
      {
        kind: 'listing',
        priceCents: price,
        minimumCents: Math.round(price * 0.85),
        condition,
        location,
        status,
        seller: link(people.find((p) => p[0] === person)[1], url('linkup', `/pessoas/${person}`)),
      },
      {
        author: people.find((p) => p[0] === person)[1],
        entities: [`person-${person}`],
        visual,
        publishedAt: epoch - 3600,
      },
    );
  }
  add(
    'memoria',
    '/sobre',
    'Sobre as capturas da Memória Web',
    'Como consultar registros de páginas que deixaram de estar disponíveis.',
    'Ajuda',
    [
      p(
        'As capturas preservam o texto publicado e a data do registro. Links podem apontar para páginas que mudaram desde então. Uma captura não garante que uma oferta continue válida.',
      ),
    ],
    undefined,
    { author: 'Equipe Memória Web' },
  );
}
