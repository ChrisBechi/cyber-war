// Authored source for web-core. Runtime never invents campaign facts or calls a service.
import { conversations } from './web-conversations.mjs';
import { worldEvents } from './web-world-events.mjs';
import { communityContent } from './web-community.mjs';
import { editorialContent } from './web-editorial.mjs';
import { longTailContent } from './web-long-tail.mjs';
const paragraph = (text) => ({ kind: 'paragraph', text });
const heading = (text) => ({ kind: 'heading', text });
const list = (items, ordered = false) => ({ kind: 'list', items, ordered });
const link = (label, url) => ({ label, url });
const epoch = 1789257600;
const slug = (text) =>
  text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
const brandRows = [
  [
    'techbyte',
    'TechByte',
    'techbyte.com',
    'EDITORIAL',
    'tech',
    '#086b66',
    'TB',
    'Tecnologia sem ruído.',
    'Reviews|Guias',
    'Técnico, acessível e criterioso.',
  ],
  [
    'redditor',
    'Redditor',
    'redditor.com',
    'FORUM',
    'community',
    '#e65023',
    'r/',
    'Toda pergunta encontra uma conversa.',
    'Tecnologia|Vida cotidiana|Aprendizado',
    'Experiências pessoais, perguntas e contrapontos.',
  ],
  [
    'shopnow',
    'ShopNow',
    'shopnow.com',
    'COMMERCE',
    'shop',
    '#5843c4',
    'S',
    'Encontre o que faz parte do seu dia.',
    'Tecnologia|Casa|Brinquedos',
    'Especificações claras; sem superlativos vazios.',
  ],
  [
    'wipedia',
    'Wipédia',
    'wipedia.org',
    'ENCYCLOPEDIA',
    'reference',
    '#42648c',
    'W',
    'A enciclopédia aberta.',
    'Tecnologia|Cultura|Cotidiano',
    'Enciclopédico, descritivo e organizado.',
  ],
  [
    'cozinha',
    'Cozinha Fácil',
    'cozinhafacil.com',
    'RECIPE',
    'kitchen',
    '#a64c28',
    'cf',
    'Receitas que cabem na sua rotina.',
    'Bolos|Salgados|Dia a dia',
    'Próximo, prático e atento a medidas.',
  ],
  [
    'educa',
    'Educa+',
    'educamais.com',
    'EDUCATION',
    'learning',
    '#6044b5',
    'e+',
    'Um pouco por dia. Muito mais longe.',
    'Programação|Idiomas|Tecnologia',
    'Didático, com exercícios e exemplos.',
  ],
  [
    'nexora',
    'Nexora',
    'nexora.com',
    'CORPORATE',
    'corporate',
    '#3746b7',
    'N',
    'Tecnologia para continuar.',
    'Produtos|Documentação|Empresa',
    'Institucional e objetivo.',
  ],
  [
    'viewtube',
    'ViewTube',
    'viewtube.com',
    'VIDEO',
    'video',
    '#c43635',
    '▶',
    'Ideias que merecem ser vistas.',
    'Tecnologia|Culinária|Cultura',
    'Apresentação direta, capítulos e contexto.',
  ],
  [
    'blog',
    'Caderno da Lia',
    'cadernodalia.blog',
    'BLOG',
    'personal',
    '#7c554c',
    'L.',
    'Anotações de uma vida em construção.',
    'Cotidiano|Tecnologia|Cozinha',
    'Primeira pessoa, observações específicas e dúvidas.',
  ],
  [
    'b1',
    'B1',
    'b1.tech',
    'EDITORIAL',
    'newspaper',
    '#b52832',
    'b1',
    'Informação para entender o presente.',
    'Tecnologia|Cidades|Cultura',
    'Jornalístico, atribui fontes e distingue opinião.',
  ],
];

// Shared facts and aliases are referenced across brands, rather than rewritten independently.
export const topics = [
  [
    'rubber-duck',
    'Pato de borracha',
    'Cotidiano',
    'patinho de borracha|rubber duck|brinquedo de banho',
    'Um pato de borracha é um brinquedo flutuante associado ao banho. Modelos de peça única evitam a abertura por onde pode entrar água.',
    'Observe as dimensões, o material e a indicação etária. A cor amarela é tradicional, mas não determina a qualidade do brinquedo.',
    'A expressão rubber duck debugging descreve explicar um problema em voz alta, passo a passo, para perceber pressupostos que passaram despercebidos.',
    'duck',
  ],
  [
    'javascript',
    'JavaScript',
    'Tecnologia',
    'js|ecmascript|programação javascript',
    'JavaScript é uma linguagem de programação usada para criar comportamento em interfaces e processar dados. Variáveis, funções e objetos compõem seu vocabulário básico.',
    'Uma função recebe entradas e pode devolver um resultado. Separar a transformação de dados da interação com a tela facilita testar o programa.',
    'A execução assíncrona permite esperar operações sem bloquear todas as outras tarefas; o resultado ainda precisa de tratamento explícito de erro.',
    'code',
  ],
  [
    'gpu',
    'Placa de vídeo',
    'Tecnologia',
    'gpu|placas de vídeo|placa grafica',
    'Uma placa de vídeo processa tarefas gráficas e reúne processador, memória e interfaces de saída. Nem todo computador precisa de uma placa dedicada.',
    'Antes de trocar a placa, confira o espaço do gabinete, os conectores da fonte e a resolução usada no monitor.',
    'Memória de vídeo, sozinha, não resume desempenho. O programa, a arquitetura e o limite de energia também influenciam o resultado.',
    'chip',
  ],
  [
    'carrot-cake',
    'Bolo de cenoura',
    'Cotidiano',
    'receita bolo de cenoura|bolo cenoura',
    'O bolo de cenoura brasileiro costuma levar cenoura batida com ovos e óleo, incorporada depois à farinha. A cobertura de chocolate é uma combinação popular.',
    'Pesar a cenoura evita que o excesso de umidade deixe o centro pesado. Misturar a farinha apenas até incorporar ajuda a preservar a textura.',
    'O tempo de forno depende da forma e do equipamento. Verifique o centro e deixe amornar antes de desenformar.',
    'cake',
  ],
  [
    'phone',
    'Celular',
    'Tecnologia',
    'smartphone|telefone móvel|celulares',
    'O celular reúne comunicação, câmera, sensores e aplicativos em um dispositivo portátil. Autonomia e ergonomia variam conforme o uso.',
    'Avalie quanto tempo o fabricante oferece atualizações, a disponibilidade de assistência e o espaço necessário para fotos e aplicativos.',
    'O NexPhone X2 é o aparelho intermediário da Nexora: tela de 6,4 polegadas, armazenamento de 128 GB e bateria de 4.600 mAh.',
    'phone',
  ],
  [
    'wifi',
    'Wi-Fi',
    'Tecnologia',
    'wifi|wi fi|rede sem fio|wireless',
    'Wi-Fi conecta dispositivos a uma rede local por rádio. Estar conectado ao roteador não garante que o acesso à internet esteja disponível.',
    'Paredes, distância e interferência alteram o sinal. Posicionar o ponto de acesso em local aberto e central costuma ajudar.',
    'A banda de 2,4 GHz tende a alcançar distâncias maiores; 5 GHz oferece mais canais, mas depende das condições do ambiente.',
    'router',
  ],
  [
    'horror',
    'Filmes de terror',
    'Cultura',
    'filme de terror|cinema de horror|suspense',
    'O terror cria tensão a partir de ameaça, incerteza e expectativa. Algumas obras privilegiam atmosfera, enquanto outras recorrem ao choque visual.',
    'Fotografia, silêncio e desenho de som ajudam a construir o medo antes que a ameaça apareça.',
    'No filme fictício A Última Frequência, uma radialista recebe transmissões de uma estação fechada; a trama trabalha isolamento e memória.',
    'film',
  ],
  [
    'english',
    'Inglês',
    'Cultura',
    'curso inglês|curso de ingles|english',
    'Estudar inglês envolve compreender, falar, ler e escrever. Práticas curtas e frequentes ajudam a consolidar vocabulário em contexto.',
    'Uma apresentação simples combina nome, ocupação e interesse. Ler frases em voz alta permite notar ritmo e pronúncia.',
    'Em vez de memorizar listas enormes, monte pequenas conversas relacionadas a situações que você realmente deseja praticar.',
    'book',
  ],
  [
    'used-car',
    'Carro usado',
    'Cotidiano',
    'automóvel usado|carros usados|trocar pneu',
    'A avaliação de um carro usado considera conservação, histórico de manutenção, documentação e adequação ao uso. Quilometragem isolada não conta toda a história.',
    'Examine pneus, iluminação, ruídos e registros de serviços. Uma inspeção independente ajuda a identificar problemas que não aparecem nas fotografias.',
    'Para trocar um pneu, consulte os pontos de apoio e as instruções do manual do veículo; a superfície deve ser estável e o local protegido do trânsito.',
    'car',
  ],
  [
    'ai',
    'Inteligência artificial',
    'Tecnologia',
    'ia|ai|aprendizado de máquina',
    'Inteligência artificial é um campo de estudo de sistemas que realizam tarefas como reconhecer padrões, prever resultados e produzir respostas.',
    'Um modelo aprende relações a partir de dados. Qualidade do conjunto, objetivo e avaliação influenciam o que ele consegue fazer.',
    'Uma resposta plausível pode estar errada. Comparar com fontes e medir o comportamento em casos diferentes é parte da avaliação.',
    'chip',
  ],
  [
    'router',
    'Roteador',
    'Tecnologia',
    'roteadores|router|ponto de acesso',
    'Um roteador encaminha tráfego entre redes. Equipamentos domésticos frequentemente também funcionam como ponto de acesso Wi-Fi e switch.',
    'O número de antenas não substitui a avaliação da cobertura. Considere paredes, quantidade de dispositivos e posicionamento.',
    'O NexLink R4 possui quatro portas gigabit e duas bandas de rádio. A configuração local oferece rede de convidados separada.',
    'router',
  ],
  [
    'dog',
    'Cachorro',
    'Cotidiano',
    'cão|cachorros|cachorro pode comer banana',
    'Cães são animais domésticos com necessidades de alimentação, exercício, descanso e interação. Essas necessidades variam com idade e condição individual.',
    'Mudanças no comportamento e no apetite devem ser observadas com atenção. Um acompanhamento veterinário orienta cuidados específicos.',
    'Dúvidas sobre alimentos, como banana, merecem considerar quantidade, preparo e saúde individual; um texto geral não substitui a orientação do veterinário.',
    'paw',
  ],
  [
    'pizza',
    'Pizza',
    'Cotidiano',
    'receita pizza|massa de pizza|pizza caseira',
    'Pizza combina uma base de massa com cobertura e cocção em temperatura elevada. Na versão caseira, fermentação e forno fazem grande diferença.',
    'Uma massa simples começa com farinha, água, fermento e sal. Descansar a massa permite desenvolver estrutura e facilita abrir o disco.',
    'Cobertura em excesso dificulta assar o centro. Escorra ingredientes úmidos e pré-aqueça o forno antes de começar.',
    'pizza',
  ],
  [
    'linux',
    'Linux',
    'Tecnologia',
    'gnu linux|terminal linux|sistema operacional',
    'Linux é o núcleo de uma família de sistemas operacionais. Uma distribuição combina esse núcleo com ferramentas, gerenciador de pacotes e interface.',
    'No terminal, pwd mostra o diretório atual, ls lista entradas e cd muda de diretório. Caminhos relativos partem do diretório em que você está.',
    'Permissões distinguem leitura, escrita e execução. Antes de alterar um arquivo de configuração, entenda sua função e preserve uma cópia.',
    'code',
  ],
  [
    'nexora',
    'Nexora',
    'Tecnologia',
    'nexora corporate|fabricante nexora',
    'A Nexora é uma fabricante de dispositivos pessoais e equipamentos de rede. Seus produtos incluem o NexPhone X2 e o roteador NexLink R4.',
    'A empresa publica especificações e documentação local de suporte. A linha X2 prioriza manutenção simples e uso cotidiano.',
    'A sede de desenvolvimento fica em Porto Claro. A documentação técnica mantém instruções distintas por modelo e versão.',
    'office',
  ],
  [
    'chair',
    'Cadeira de escritório',
    'Cotidiano',
    'cadeira gamer|cadeira ergonômica',
    'Uma cadeira de escritório deve permitir postura confortável e ajustes compatíveis com a mesa. O formato externo não garante adaptação ao corpo.',
    'Altura do assento, apoio lombar e posição dos braços precisam ser considerados em conjunto. Alterne posição e faça pausas.',
    'Compare medidas do produto com seu espaço antes de comprar. Rodízios diferentes atendem pisos com necessidades distintas.',
    'chair',
  ],
  [
    'radio',
    'História do rádio',
    'Cultura',
    'rádio|radiodifusão|ondas de rádio',
    'O rádio transmite informação por ondas eletromagnéticas. A radiodifusão aproximou notícias, música e entretenimento de públicos distantes.',
    'A programação ao vivo desenvolveu formatos próprios, como entrevistas, radionovelas e boletins. Receptores portáteis mudaram hábitos de escuta.',
    'No arquivo cultural de Porto Claro, registros de programas comunitários ajudam a observar como a linguagem se transformou.',
    'radio',
  ],
  [
    'hotel',
    'Hospedagem econômica',
    'Cotidiano',
    'hotel barato|pousada|viagem',
    'Planejar hospedagem envolve localização, transporte, horários e serviços incluídos. O menor preço anunciado nem sempre corresponde ao menor custo total.',
    'Compare a distância dos lugares que pretende visitar e confira as condições descritas na reserva.',
    'Uma pousada perto de transporte público pode reduzir deslocamentos; uma cozinha compartilhada pode ajudar em estadias longas.',
    'house',
  ],
  [
    'coffee',
    'Café coado',
    'Cotidiano',
    'café|coador|preparo de café',
    'No café coado, água atravessa o café moído e um filtro separa a bebida do pó. Moagem, proporção e tempo alteram o sabor.',
    'Comece com uma proporção consistente, anote o resultado e ajuste uma variável por vez. Moagem muito fina pode alongar o escoamento.',
    'Um recipiente limpo e água de boa qualidade ajudam a perceber o sabor do grão. Não é necessário ferver repetidamente a água.',
    'cup',
  ],
  [
    'usb',
    'Cabos USB',
    'Tecnologia',
    'cabo usb|usb c|carregamento',
    'Cabos USB transportam energia e, dependendo do modelo, dados. O formato do conector não revela sozinho as capacidades do cabo.',
    'Confira comprimento, potência declarada e padrão de transferência. Um cabo adequado para carregar pode ser lento para copiar arquivos grandes.',
    'Evite dobrar o cabo junto ao conector. Para organizar a mesa, use uma curva suave e um comprimento adequado à distância.',
    'cable',
  ],
];

export function buildPack({ longTailLimit = 192 } = {}) {
  const brands = brandRows.map(
    ([id, name, domain, platform, layout, accent, mark, tagline, categories, voice]) => ({
      id,
      name,
      domain,
      platform,
      layout,
      accent,
      mark,
      tagline,
      voice,
      navigation: categories
        .split('|')
        .map((c) => link(c, `https://www.${domain}/?category=${encodeURIComponent(c)}`)),
    }),
  );
  const brand = (id) => brands.find((b) => b.id === id);
  const url = (id, path = '/') => `https://www.${brand(id).domain}${path}`;
  const docs = [];
  const entities = topics.map(([id, name, , aliases, description]) => ({
    id,
    name,
    kind: id === 'nexora' ? 'Company' : 'Topic',
    aliases: aliases.split('|'),
    description,
    relations: [],
  }));
  entities.push({
    id: 'nexphone-x2',
    name: 'NexPhone X2',
    kind: 'Device',
    aliases: ['X2', 'celular Nexora'],
    description: 'Tela de 6,4 polegadas; 128 GB; bateria de 4.600 mAh.',
    relations: [
      { kind: 'MANUFACTURED_BY', target: 'nexora' },
      { kind: 'RELATED_TO', target: 'phone' },
    ],
  });
  entities.push({
    id: 'nexlink-r4',
    name: 'NexLink R4',
    kind: 'Device',
    aliases: ['R4'],
    description: 'Roteador de duas bandas com quatro portas gigabit.',
    relations: [
      { kind: 'MANUFACTURED_BY', target: 'nexora' },
      { kind: 'RELATED_TO', target: 'router' },
    ],
  });
  const add = (
    b,
    path,
    title,
    summary,
    category,
    blocks,
    detail = { kind: 'article', readingMinutes: 4 },
    extra = {},
  ) => {
    const doc = {
      id: `web-${b}-${slug(path) || 'home'}`,
      brandId: b,
      path,
      title,
      summary,
      category,
      author:
        {
          techbyte: 'Renata Avelar',
          redditor: 'circuito_aberto',
          shopnow: 'Equipe ShopNow',
          wipedia: 'Comunidade Wipédia',
          cozinha: 'Dora Nascimento',
          educa: 'Iara Mendonça',
          nexora: 'Nexora',
          viewtube: 'Oficina Aberta',
          blog: 'Lia Moreira',
          b1: 'Redação B1',
        }[b] ?? brand(b).name,
      publishedAt: epoch - 86400 * 3,
      entities: [],
      keywords: [],
      blocks,
      detail,
      comments: [],
      links: [],
      visual: 'book',
      requiredFlags: [],
      ...extra,
    };
    docs.push(doc);
    return doc;
  };
  const comment = (id, author, text, rating = null) => ({
    id,
    author,
    text,
    rating,
    publishedAt: epoch - 86400,
  });
  const topicLinks = (id) => [
    link('Entenda o assunto na Wipédia', url('wipedia', `/wiki/${id}`)),
    link('Converse com a comunidade', url('redditor', `/t/${id}`)),
  ];
  for (const [id, name, category, aliases, definition, advice, context, visual] of topics) {
    const extra = { entities: [id], keywords: aliases.split('|'), visual };
    const [poster, question, situation, response] = conversations[id];
    add(
      'wipedia',
      `/wiki/${id}`,
      name,
      definition,
      category,
      [
        paragraph(definition),
        heading('Características'),
        paragraph(advice),
        heading('Contexto e aplicações'),
        paragraph(context),
        heading('Referências'),
        { kind: 'link', ...link('Guia da TechByte', url('techbyte', `/guias/${id}`)) },
      ],
      {
        kind: 'reference',
        facts: [
          ['Área', category],
          ['Termos relacionados', aliases.split('|').join(', ')],
        ],
      },
      { ...extra, links: topicLinks(id).slice(1) },
    );
    add(
      'techbyte',
      `/guias/${id}`,
      `${name}: o que observar antes de começar`,
      advice,
      category === 'Tecnologia' ? 'Guias' : 'Vida digital',
      [
        paragraph(definition),
        heading('Comece pelo essencial'),
        paragraph(advice),
        heading('O detalhe que muda a experiência'),
        paragraph(context),
        {
          kind: 'quote',
          text: 'Uma boa comparação começa definindo a necessidade, antes de escolher uma solução.',
          attribution: 'Renata Avelar, editora de guias',
        },
      ],
      undefined,
      {
        ...extra,
        links: topicLinks(id),
        comments: [
          comment(
            `${id}-tb`,
            'Pedro Arantes',
            `A parte sobre ${aliases.split('|')[0]} me ajudou a formular uma pergunta melhor. Vou comparar as opções usando esses critérios.`,
          ),
        ],
      },
    );
    add(
      'redditor',
      `/t/${id}`,
      question,
      situation,
      ['javascript', 'english', 'linux'].includes(id)
        ? 'Aprendizado'
        : category === 'Tecnologia'
          ? 'Tecnologia'
          : 'Vida cotidiana',
      [paragraph(situation)],
      {
        kind: 'thread',
        community: category === 'Tecnologia' ? 'tecnologia' : 'cotidiano',
        votes: 38 + id.length,
        locked: false,
      },
      {
        ...extra,
        author: poster,
        links: [
          link('O guia que motivou a conversa', url('techbyte', `/guias/${id}`)),
          link('Anotação da Lia', url('blog', `/notas/${id}`)),
        ],
        comments: [comment(`${id}-r1`, id.length % 2 ? 'samira_m' : 'duarte.c', response)],
      },
    );
    add(
      'blog',
      `/notas/${id}`,
      `No meu caderno: ${name.toLowerCase()}`,
      `Hoje separei um tempo para entender ${name.toLowerCase()} sem abrir vinte abas de uma vez.`,
      ['carrot-cake', 'pizza', 'coffee'].includes(id)
        ? 'Cozinha'
        : category === 'Tecnologia'
          ? 'Tecnologia'
          : 'Cotidiano',
      [
        paragraph(`Anotei esta ideia para não esquecer: ${advice}`),
        paragraph(`O ponto de partida foi simples. ${definition}`),
        heading('O que ficou comigo'),
        paragraph(`Depois de ler e conversar, ficou uma observação: ${context}`),
        paragraph(
          'Deixei as referências abaixo. Se você já passou por uma situação parecida, conta nos comentários; quero retomar esse assunto com mais exemplos.',
        ),
      ],
      undefined,
      {
        ...extra,
        links: topicLinks(id),
        comments: [
          comment(
            `${id}-bl`,
            'Nina Valença',
            `Gostei da pergunta sobre ${name.toLowerCase()}. Vou levar esse contexto para a conversa lá no fórum.`,
          ),
        ],
      },
    );
    add(
      'viewtube',
      `/watch/${id}`,
      `${name} em cinco minutos`,
      `${definition} Uma introdução com exemplos e referências para continuar estudando.`,
      ['carrot-cake', 'pizza', 'coffee'].includes(id) ? 'Culinária' : category,
      [paragraph(definition), paragraph(advice), paragraph(context)],
      {
        kind: 'video',
        seconds: 300 + id.length * 7,
        views: 1200 + id.length * 231,
        channel: link('Oficina Aberta', url('viewtube', '/canal/oficina-aberta')),
        chapters: [
          '00:00 · A pergunta inicial',
          '01:10 · Conceitos e exemplos',
          '03:30 · Cuidados e próximos passos',
        ],
      },
      { ...extra, links: topicLinks(id) },
    );
  }
  add(
    'viewtube',
    '/canal/oficina-aberta',
    'Oficina Aberta',
    'Conversas curtas sobre tecnologia, cultura e cotidiano.',
    'Canais',
    [
      paragraph(
        'Somos um pequeno canal interessado em explicar uma coisa por vez. Publicamos introduções com capítulos, referências e espaço para perguntas.',
      ),
    ],
    {
      kind: 'profile',
      bio: 'Apresentação de Beto Siqueira; pesquisa de Renata Avelar.',
      location: 'Porto Claro',
    },
    { links: topics.slice(0, 8).map(([id, name]) => link(name, url('viewtube', `/watch/${id}`))) },
  );
  add(
    'shopnow',
    '/vendedor/casa-do-dia',
    'Casa do Dia',
    'Peças úteis para a casa, com descrição e medidas verificáveis.',
    'Vendedores',
    [
      paragraph(
        'A Casa do Dia trabalha com acessórios de mesa, brinquedos de banho e organização. Consulte as medidas de cada versão antes de escolher.',
      ),
      paragraph(
        'Atendimento de segunda a sexta. O catálogo informa disponibilidade por versão; os comentários ficam reunidos na página de cada produto.',
      ),
    ],
    { kind: 'profile', bio: 'Loja de utilidades fundada em Porto Claro.', location: 'Porto Claro' },
    { visual: 'house', links: [link('Ver catálogo', url('shopnow'))] },
  );
  add(
    'educa',
    '/professores/iara-mendonca',
    'Iara Mendonça',
    'Professora de programação e tecnologias do cotidiano.',
    'Professores',
    [
      paragraph(
        'Iara ensina a decompor problemas e testar hipóteses com exemplos pequenos. Suas aulas combinam leitura, exercício e uma pergunta para reflexão.',
      ),
    ],
    {
      kind: 'profile',
      bio: 'Educadora e desenvolvedora de materiais didáticos.',
      location: 'Porto Claro',
    },
    {
      visual: 'book',
      links: [link('Cursos de programação', url('educa', '/?category=Programa%C3%A7%C3%A3o'))],
    },
  );
  const recipes = [
    [
      'bolo-de-cenoura',
      'Bolo de cenoura com cobertura',
      50,
      10,
      'Bolos',
      [
        '250 g de cenoura descascada',
        '3 ovos',
        '180 ml de óleo',
        '200 g de açúcar',
        '240 g de farinha de trigo',
        '12 g de fermento químico',
        '100 g de chocolate para a cobertura',
      ],
      [
        'Aqueça o forno a 180 °C e unte uma forma média.',
        'Bata cenoura, ovos, óleo e açúcar até ficar homogêneo.',
        'Incorpore a farinha com uma espátula e misture o fermento por último.',
        'Asse por cerca de 35 minutos; confira o centro antes de retirar.',
        'Deixe amornar e espalhe o chocolate derretido por cima.',
      ],
      'carrot-cake',
      'cake',
    ],
    [
      'pizza-caseira',
      'Pizza caseira de tomate e queijo',
      100,
      4,
      'Salgados',
      [
        '300 g de farinha de trigo',
        '190 ml de água',
        '5 g de fermento biológico seco',
        '6 g de sal',
        '15 ml de azeite',
        '120 g de molho de tomate',
        '150 g de queijo',
        '1 tomate fatiado',
      ],
      [
        'Misture farinha, água e fermento; junte sal e azeite.',
        'Sove até obter uma massa lisa e deixe crescer por aproximadamente uma hora.',
        'Abra dois discos, distribua molho, queijo e tomate sem exagerar na cobertura.',
        'Asse no forno previamente aquecido a 240 °C até dourar as bordas.',
      ],
      'pizza',
      'pizza',
    ],
    [
      'cafe-coado',
      'Café coado para duas pessoas',
      10,
      2,
      'Dia a dia',
      ['20 g de café moído', '300 ml de água filtrada'],
      [
        'Aqueça a água e escalde o filtro de papel.',
        'Coloque o café no filtro e umedeça todo o pó com um pouco de água.',
        'Despeje o restante aos poucos, em movimentos circulares.',
        'Espere escoar e sirva em xícaras aquecidas.',
      ],
      'coffee',
      'cup',
    ],
    [
      'arroz-solto',
      'Arroz soltinho do dia a dia',
      25,
      4,
      'Dia a dia',
      [
        '1 xícara de arroz',
        '2 xícaras de água quente',
        '1 colher de sopa de óleo',
        '1 dente de alho',
        'Sal a gosto',
      ],
      [
        'Refogue o alho no óleo sem deixar escurecer.',
        'Acrescente o arroz e mexa por um minuto.',
        'Junte água e sal. Cozinhe com a tampa entreaberta até secar.',
        'Desligue, tampe e aguarde cinco minutos antes de soltar com um garfo.',
      ],
      null,
      'bowl',
    ],
    [
      'panqueca-simples',
      'Panqueca simples de frigideira',
      20,
      3,
      'Dia a dia',
      [
        '1 ovo',
        '1 xícara de leite',
        '1 xícara de farinha',
        '1 colher de chá de fermento',
        '1 colher de sopa de açúcar',
      ],
      [
        'Misture ovo e leite; incorpore farinha e açúcar.',
        'Adicione fermento e misture delicadamente.',
        'Despeje pequenas porções em frigideira untada.',
        'Vire quando aparecerem bolhas e termine de dourar.',
      ],
      null,
      'cake',
    ],
  ];
  for (const [
    id,
    name,
    minutes,
    servings,
    category,
    ingredients,
    steps,
    entity,
    visual,
  ] of recipes) {
    const notes = {
      'bolo-de-cenoura': [
        'Corte a cenoura em pedaços pequenos antes de bater. Uma massa uniforme evita bolsões de farinha no miolo; incorpore o fermento por último.',
        'Ralei a cenoura antes de bater e o miolo ficou uniforme. Esperei amornar para desenformar e a cobertura não escorreu toda para o prato.',
      ],
      'pizza-caseira': [
        'Abra a massa depois do descanso e mantenha a cobertura leve. Tomate muito úmido pode deixar o centro mole; distribua as fatias sem acumular líquido.',
        'Escorri as fatias de tomate e a base assou melhor. Fiz dois discos pequenos para caber na minha assadeira.',
      ],
      'cafe-coado': [
        'Umedeça o pó por inteiro antes de completar a água. Despejar devagar facilita observar a passagem pelo filtro e repetir o preparo que você preferiu.',
        'Usei a mesma medida de água nos dois preparos e consegui perceber a diferença da moagem. Na próxima vez vou anotar também o tempo.',
      ],
      'arroz-solto': [
        'Depois de desligar, mantenha a panela tampada durante o descanso. Solte os grãos com um garfo, sem amassar; mexer o tempo todo durante a cocção altera a textura.',
        'O descanso no final fez diferença. Eu costumava mexer o arroz ainda com água e desta vez esperei para soltar com o garfo.',
      ],
      'panqueca-simples': [
        'Faça a primeira panqueca pequena para ajustar o fogo. Bolhas na superfície e bordas firmes ajudam a perceber quando virar sem rasgar a massa.',
        'A primeira ficou escura, então baixei o fogo. As seguintes douraram por igual e consegui virar sem dobrar.',
      ],
    }[id];
    add(
      'cozinha',
      `/receitas/${id}`,
      name,
      `${name} com medidas, preparo em ${minutes} minutos e rendimento de ${servings} porções.`,
      category,
      [
        paragraph(
          'Separe os ingredientes antes de começar e leia o modo de preparo até o fim. Os tempos são uma referência: observe a textura durante o preparo.',
        ),
        heading('Na cozinha da Dora'),
        paragraph(
          category === 'Bolos'
            ? 'Evite abrir o forno nos primeiros minutos. Deixe o bolo amornar para desenformar com mais facilidade.'
            : notes[0],
        ),
      ],
      { kind: 'recipe', minutes, servings, difficulty: 'Fácil', ingredients, steps },
      {
        entities: entity ? [entity] : [],
        visual,
        keywords: [name, 'receita'],
        comments: [comment(`${id}-recipe`, 'Clara Rios', notes[1], 5)],
        links: entity ? topicLinks(entity) : [],
      },
    );
  }

  const lessonThemes = [
    [
      'Objetivo',
      'Defina uma pergunta que você gostaria de responder com este assunto. Escreva uma situação concreta em que a resposta seria útil.',
    ],
    [
      'Vocabulário',
      'Identifique três termos do texto. Explique cada um usando suas palavras e confira se a explicação ainda combina com o exemplo.',
    ],
    [
      'Observação',
      'Descreva o que você observa antes de propor uma mudança. Separe o que foi observado daquilo que é apenas uma hipótese.',
    ],
    [
      'Primeiro exemplo',
      'Monte um exemplo pequeno. Escolha uma entrada ou situação e registre o resultado esperado antes de experimentar.',
    ],
    [
      'Comparação',
      'Compare dois casos mudando apenas uma característica. Anote o que permaneceu igual e o que mudou.',
    ],
    [
      'Limitações',
      'Encontre uma situação em que a regra geral não basta. Explique quais informações adicionais seriam necessárias.',
    ],
    [
      'Explicação',
      'Explique o conceito em voz alta para alguém que está começando. Reescreva o trecho em que você precisou voltar atrás.',
    ],
    [
      'Revisão',
      'Retome sua pergunta inicial. Use os conceitos do texto para melhorar a resposta e indique o que continua em aberto.',
    ],
  ];
  for (const [id, name, , aliases, definition, advice, context, visual] of topics.filter((t) =>
    [
      'javascript',
      'english',
      'linux',
      'wifi',
      'ai',
      'router',
      'gpu',
      'usb',
      'radio',
      'coffee',
    ].includes(t[0]),
  )) {
    const path = `/cursos/${id}`;
    const lessons = lessonThemes.map(([title], i) =>
      link(`${i + 1}. ${title}: ${name.toLowerCase()}`, url('educa', `${path}/aula-${i + 1}`)),
    );
    add(
      'educa',
      path,
      `Introdução a ${name}`,
      `Um percurso de oito aulas para compreender ${name.toLowerCase()}, praticar e organizar suas dúvidas.`,
      id === 'english'
        ? 'Idiomas'
        : id === 'javascript' || id === 'linux'
          ? 'Programação'
          : 'Tecnologia',
      [
        paragraph(definition),
        paragraph(advice),
        heading('O que você vai praticar'),
        list([
          'Explicar os conceitos com exemplos.',
          'Comparar situações e reconhecer limitações.',
          'Construir uma pequena revisão ao final.',
        ]),
      ],
      {
        kind: 'course',
        minutes: 120,
        level: 'Iniciante',
        teacher: link('Iara Mendonça', url('educa', '/professores/iara-mendonca')),
        lessons,
      },
      {
        entities: [id],
        keywords: [...aliases.split('|'), 'curso', 'aulas'],
        visual,
        links: topicLinks(id),
      },
    );
    lessonThemes.forEach(([theme, exercise], i) => {
      const example =
        id === 'javascript'
          ? {
              kind: 'code',
              language: 'javascript',
              text: [
                'const preco = 25;\nconst quantidade = 3;\nconst total = preco * quantidade;',
                'function dobro(valor) {\n  return valor * 2;\n}\ndobro(4); // 8',
                'const tarefas = ["ler", "praticar"];\nconst quantidade = tarefas.length;',
              ][i % 3],
            }
          : id === 'linux'
            ? {
                kind: 'code',
                language: 'shell',
                text: 'pwd\nls\nmkdir estudo\ncd estudo\necho "minha observação" > notas.txt\ncat notas.txt',
              }
            : paragraph(i % 2 ? advice : context);
      add(
        'educa',
        `${path}/aula-${i + 1}`,
        `${name} · Aula ${i + 1}: ${theme}`,
        `${theme} em ${name.toLowerCase()}: ${exercise}`,
        'Aulas',
        [
          paragraph(definition),
          heading(`${theme} na prática`),
          example,
          heading('Exercício'),
          paragraph(exercise),
          heading('Para conferir sua resposta'),
          paragraph(i % 2 ? context : advice),
        ],
        {
          kind: 'lesson',
          course: link(`Introdução a ${name}`, url('educa', path)),
          position: i + 1,
          next: lessons[i + 1] ?? null,
        },
        { entities: [id], keywords: aliases.split('|'), visual, links: topicLinks(id) },
      );
    });
  }

  const product = (path, title, summary, entity, visual, price, specs, stock = 'IN_STOCK') =>
    add(
      'shopnow',
      path,
      title,
      summary,
      entity === 'rubber-duck' ? 'Brinquedos' : entity === 'chair' ? 'Casa' : 'Tecnologia',
      [
        paragraph(summary),
        heading('Antes de escolher'),
        paragraph(
          topics.find((t) => t[0] === entity)?.[5] ??
            'Confira medidas e compatibilidade antes de escolher.',
        ),
        heading('Conteúdo da embalagem'),
        paragraph(
          `Uma unidade de ${title}. Consulte as especificações desta versão para conferir as medidas e os conectores.`,
        ),
      ],
      {
        kind: 'product',
        priceCents: price,
        stock,
        seller: link('Casa do Dia', url('shopnow', '/vendedor/casa-do-dia')),
        specifications: specs,
      },
      {
        entities: [entity],
        visual,
        keywords: [title, ...(topics.find((t) => t[0] === entity)?.[3].split('|') ?? [])],
        comments: [
          comment(
            `${slug(path)}-review`,
            'Marina Couto',
            `Escolhi esta versão pelas medidas (${specs[0][1]}). A descrição corresponde ao que eu precisava para o meu espaço.`,
            4,
          ),
        ],
        links: topicLinks(entity),
      },
    );
  product(
    '/produto/pato-de-borracha',
    'Pato de borracha clássico',
    'Patinho amarelo de peça única, com 8 cm de altura e base flutuante. Superfície sem abertura para entrada de água.',
    'rubber-duck',
    'duck',
    1890,
    [
      ['Altura', '8 cm'],
      ['Material', 'Elastômero flexível'],
      ['Cor', 'Amarelo'],
      ['Indicação', 'A partir de 3 anos'],
    ],
  );
  const x2 = product(
    '/produto/nexphone-x2',
    'NexPhone X2 · 128 GB',
    'O celular da Nexora para o dia a dia: tela de 6,4 polegadas, 128 GB e bateria de 4.600 mAh.',
    'phone',
    'phone',
    189900,
    [
      ['Armazenamento', '128 GB'],
      ['Tela', '6,4 polegadas'],
      ['Bateria', '4.600 mAh'],
      ['Cor', 'Grafite'],
    ],
  );
  x2.entities.push('nexphone-x2');
  x2.links.push(link('Conheça o fabricante', url('nexora', '/produtos/nexphone-x2')));
  product(
    '/produto/nexlink-r4',
    'NexLink R4',
    'Roteador de duas bandas com quatro portas gigabit e configuração local de rede de convidados.',
    'router',
    'router',
    32900,
    [
      ['Portas', '4 × gigabit'],
      ['Bandas', '2,4 GHz e 5 GHz'],
      ['Alimentação', '12 V'],
    ],
    'LOW_STOCK',
  );
  product(
    '/produto/vertex-v6',
    'Vertex V6 · 8 GB',
    'Placa gráfica de entrada para jogos leves e trabalho com duas telas. Confira os conectores e o espaço no gabinete.',
    'gpu',
    'chip',
    129900,
    [
      ['Memória', '8 GB'],
      ['Comprimento', '22 cm'],
      ['Saídas', '2 × DisplayPort; 1 × HDMI'],
    ],
  );
  product(
    '/produto/cadeira-cais',
    'Cadeira Cais',
    'Cadeira para escritório com altura regulável, apoio lombar e braços ajustáveis.',
    'chair',
    'chair',
    74900,
    [
      ['Altura do assento', '43 a 53 cm'],
      ['Largura', '49 cm'],
      ['Revestimento', 'Tecido cinza'],
    ],
  );
  // Long-tail inventory represents real SKU differences: connector, construction, length and color.
  // It does not manufacture extra news or campaign clues to reach a document count.
  const constructions = [
    ['Essencial', 'Revestimento flexível em PVC', 0],
    ['Trama', 'Revestimento trançado em tecido', 600],
    ['Flex', 'Revestimento macio em silicone', 900],
    ['Mesa', 'Reforço nos conectores para organização da mesa', 500],
  ];
  const connectors = [
    ['C–C', 'USB-C para USB-C', '60 W', '480 Mb/s', 1200],
    ['A–C', 'USB-A para USB-C', '15 W', '480 Mb/s', 900],
    ['A–Micro', 'USB-A para micro USB', '10 W', '480 Mb/s', 600],
  ];
  const colors = ['Grafite', 'Branco', 'Azul', 'Verde', 'Vermelho', 'Lilás', 'Areia', 'Laranja'];
  for (const [series, material, surcharge] of constructions)
    for (const [code, connector, power, speed, base] of connectors)
      for (const length of [0.25, 0.5, 1, 1.5, 2, 3, 4, 5])
        for (const color of colors) {
          const identity = `${series}-${code}-${length}-${color}`;
          const title = `Cabo ${series} ${code} · ${String(length).replace('.', ',')} m · ${color.toLowerCase()}`;
          const summary = `${connector}, ${String(length).replace('.', ',')} m, acabamento ${color.toLowerCase()}. ${material}; até ${power} e transferência de ${speed}.`;
          const doc = product(
            `/produto/${slug(identity)}`,
            title,
            summary,
            'usb',
            'cable',
            base + surcharge + Math.round(length * 280),
            [
              ['Comprimento', `${length} m`],
              ['Conectores', connector],
              ['Construção', material],
              ['Cor', color],
              ['Potência', power],
              ['Transferência', speed],
            ],
            length === 5 && color === 'Lilás' ? 'OUT_OF_STOCK' : 'IN_STOCK',
          );
          doc.comments = [];
        }
  for (const section of ['sobre', 'equipe', 'carreiras', 'suporte']) {
    const data = {
      sobre: [
        'Sobre a Nexora',
        'A Nexora desenvolve dispositivos pessoais e equipamentos de rede em Porto Claro.',
        'As equipes de produto e suporte compartilham a documentação de cada versão, para que especificações e instruções permaneçam consistentes.',
      ],
      equipe: [
        'Pessoas da Nexora',
        'Engenharia, design e suporte trabalham a partir dos mesmos requisitos de produto.',
        'Elisa Brandão coordena o desenvolvimento do NexPhone. Ravi Torres cuida da experiência de configuração do NexLink.',
      ],
      carreiras: [
        'Trabalhe na Nexora',
        'Buscamos pessoas curiosas sobre o que acontece entre o primeiro protótipo e o uso cotidiano.',
        'Áreas: suporte de produto, documentação técnica e testes de dispositivos. A página de cada vaga será publicada neste espaço.',
      ],
      suporte: [
        'Suporte e documentação',
        'Escolha o produto para consultar instruções e especificações.',
        'Confira o modelo na etiqueta do equipamento. Instruções de outra revisão podem apresentar menus diferentes.',
      ],
    }[section];
    add(
      'nexora',
      `/${section}`,
      data[0],
      data[1],
      'Empresa',
      [paragraph(data[1]), paragraph(data[2])],
      undefined,
      {
        entities: ['nexora'],
        visual: 'office',
        links: [
          link('NexPhone X2', url('nexora', '/produtos/nexphone-x2')),
          link('Documentação do NexLink R4', url('nexora', '/docs/nexlink-r4')),
        ],
      },
    );
  }
  add(
    'nexora',
    '/produtos/nexphone-x2',
    'NexPhone X2',
    'Espaço para o seu dia. Tela de 6,4 polegadas, 128 GB e bateria de 4.600 mAh.',
    'Produtos',
    [
      paragraph(
        'O X2 reúne comunicação, câmera e aplicativos em um corpo de acabamento grafite. A interface prioriza as tarefas mais frequentes.',
      ),
      heading('Especificações'),
      {
        kind: 'table',
        headings: ['Característica', 'NexPhone X2'],
        rows: [
          ['Tela', '6,4 polegadas'],
          ['Armazenamento', '128 GB'],
          ['Bateria', '4.600 mAh'],
        ],
      },
    ],
    {
      kind: 'reference',
      facts: [
        ['Fabricante', 'Nexora'],
        ['Linha', 'NexPhone'],
      ],
    },
    {
      entities: ['nexora', 'nexphone-x2', 'phone'],
      visual: 'phone',
      links: [
        link('Manual do X2', url('nexora', '/docs/nexphone-x2')),
        link('Leia a análise da TechByte', url('techbyte', '/reviews/nexphone-x2')),
        link('Encontrar na ShopNow', url('shopnow', '/produto/nexphone-x2')),
      ],
    },
  );
  for (const [id, title, body] of [
    [
      'nexphone-x2',
      'Manual do NexPhone X2',
      'Use Configurações > Armazenamento para verificar o espaço disponível. A bateria tem capacidade nominal de 4.600 mAh; a autonomia depende do uso.',
    ],
    [
      'nexlink-r4',
      'NexLink R4: configuração local',
      'Conecte o computador a uma porta LAN. Abra o painel local do equipamento, escolha um nome de rede e configure a rede de convidados separadamente.',
    ],
  ]) {
    add(
      'nexora',
      `/docs/${id}`,
      title,
      body,
      'Documentação',
      [
        heading('Versão 1.0'),
        paragraph(body),
        heading('Antes de alterar'),
        list([
          'Anote a configuração atual.',
          'Confirme o modelo do equipamento.',
          'Verifique a conexão depois de salvar.',
        ]),
        {
          kind: 'code',
          language: 'text',
          text: `Produto: ${id}\nDocumento: guia de uso\nRevisão: 1.0\nSuporte: nexora.com/suporte`,
        },
      ],
      {
        kind: 'reference',
        facts: [
          ['Revisão', '1.0'],
          ['Idioma', 'Português'],
        ],
      },
      {
        entities: ['nexora', id],
        visual: id === 'nexphone-x2' ? 'phone' : 'router',
        links: [
          link('Centro de suporte', url('nexora', '/suporte')),
          link('Guia de Wi-Fi', url('techbyte', '/guias/wifi')),
        ],
      },
    );
  }
  add(
    'techbyte',
    '/reviews/nexphone-x2',
    'NexPhone X2: o cotidiano como medida',
    'A proposta da Nexora é simples: tela confortável, espaço para arquivos e uma interface direta.',
    'Reviews',
    [
      paragraph(
        'O NexPhone X2 tem tela de 6,4 polegadas, 128 GB de armazenamento e bateria de 4.600 mAh. Esses números descrevem a configuração; não substituem a avaliação do uso.',
      ),
      heading('O que faz sentido'),
      paragraph(
        'O espaço atende quem prefere guardar parte das fotos e arquivos no aparelho. A tela favorece leitura, mas o tamanho pode incomodar quem procura um celular compacto.',
      ),
      heading('O que conferir'),
      paragraph(
        'Antes de escolher, compare o tamanho na mão, a disponibilidade de assistência e os recursos de que você realmente precisa.',
      ),
    ],
    undefined,
    {
      entities: ['nexphone-x2', 'nexora', 'phone'],
      visual: 'phone',
      links: [
        link('Especificações da fabricante', url('nexora', '/produtos/nexphone-x2')),
        link('Preço e disponibilidade', url('shopnow', '/produto/nexphone-x2')),
        link('Discussão da comunidade', url('redditor', '/t/phone')),
      ],
    },
  );
  const stories = [
    [
      'biblioteca',
      'Biblioteca de Porto Claro abre oficina de programação',
      'A atividade reúne iniciantes para explorar funções e pequenos programas.',
      'javascript',
    ],
    [
      'rede-bairro',
      'Centro comunitário reorganiza rede de acesso',
      'A equipe reposicionou pontos de acesso e separou a conexão destinada aos visitantes.',
      'wifi',
    ],
    [
      'radio-arquivo',
      'Acervo de rádio ganha catálogo público',
      'A coleção organiza programas por período, emissora e tema.',
      'radio',
    ],
    [
      'cinema',
      'Mostra local debate som e silêncio no terror',
      'A programação inclui A Última Frequência e uma conversa sobre desenho de som.',
      'horror',
    ],
    [
      'cozinha-bairro',
      'Feira de bairro reúne receitas de família',
      'Cadernos de cozinha e demonstrações de preparo fazem parte da programação.',
      'carrot-cake',
    ],
  ];
  for (const [id, title, summary, topic] of stories) {
    add(
      'b1',
      `/noticias/${id}`,
      title,
      summary,
      topic === 'horror' || topic === 'radio'
        ? 'Cultura'
        : topic === 'javascript'
          ? 'Tecnologia'
          : 'Cidades',
      [
        paragraph(summary),
        paragraph(topics.find((t) => t[0] === topic)[4]),
        heading('Entenda o contexto'),
        paragraph(topics.find((t) => t[0] === topic)[6]),
      ],
      undefined,
      {
        entities: [topic],
        visual: topics.find((t) => t[0] === topic)[7],
        links: topicLinks(topic),
      },
    );
  }
  const eventNews = add(
    'b1',
    '/noticias/revisao-de-acessos',
    'Orion confirma revisão de acessos',
    'A empresa informou que analisa registros de sua rede corporativa após relatos de tráfego incomum.',
    'Tecnologia',
    [
      paragraph(
        'A Orion confirmou uma revisão de acessos. A empresa não publicou uma conclusão sobre a origem dos registros.',
      ),
      paragraph(
        'Especialistas consultados pelo B1 recomendam preservar os arquivos originais e conferir a integridade das cópias antes de tirar conclusões.',
      ),
    ],
    undefined,
    {
      requiredFlags: ['SESSION_1_COMPLETE'],
      visual: 'office',
      links: [
        link('Discussão sobre preservação de registros', url('redditor', '/t/registros-orion')),
      ],
    },
  );
  const eventThread = add(
    'redditor',
    '/t/registros-orion',
    'Como preservar os registros citados pelo B1?',
    'A notícia fala em revisão de acessos; quais cuidados ajudam a evitar conclusões precipitadas?',
    'Tecnologia',
    [
      paragraph(
        'Li a confirmação da revisão e fiquei com uma dúvida sobre integridade. Comparar hashes ajuda a saber se duas cópias são iguais, mas não confirma quem criou o arquivo.',
      ),
    ],
    { kind: 'thread', community: 'tecnologia', votes: 18, locked: false },
    {
      requiredFlags: ['SESSION_1_COMPLETE'],
      visual: 'code',
      links: [link('Reportagem do B1', url('b1', '/noticias/revisao-de-acessos'))],
      comments: [
        comment(
          'registros-1',
          'arquivo_vivo',
          'Preserve o original e documente de onde veio a cópia. Integridade e autoria são perguntas diferentes.',
        ),
      ],
    },
  );
  eventThread.comments[0].publishedAt = eventThread.publishedAt + 60;
  const timeline = worldEvents({ add, url, epoch });
  communityContent({ brands, entities, docs, add, url, epoch });
  editorialContent({ brands, entities, add, url, epoch });
  longTailContent({ brands, entities, add, url, epoch, limit: longTailLimit });
  // Explicit homepage entries ensure navigational searches lead to a brand, not an arbitrary SKU.
  for (const b of brands) {
    add(
      b.id,
      '/',
      b.name,
      b.tagline,
      'Início',
      [paragraph(b.tagline)],
      { kind: 'reference', facts: [['Endereço', b.domain]] },
      {
        visual:
          b.id === 'shopnow'
            ? 'duck'
            : b.id === 'cozinha'
              ? 'cake'
              : b.id === 'nexora'
                ? 'phone'
                : 'book',
        keywords: [b.name, b.domain],
        entities: b.id === 'nexora' ? ['nexora'] : [],
      },
    );
  }
  const technical = new Set(topics.filter((t) => t[2] === 'Tecnologia').map((t) => t[0]));
  technical.add('rubber-duck');
  const removedGuides = new Map();
  for (const doc of docs) {
    if (
      doc.brandId === 'techbyte' &&
      doc.path.startsWith('/guias/') &&
      !technical.has(doc.entities[0])
    ) {
      removedGuides.set(url('techbyte', doc.path), url('blog', `/notas/${doc.entities[0]}`));
    }
  }
  for (let i = docs.length - 1; i >= 0; i--) {
    if (docs[i].brandId === 'techbyte' && removedGuides.has(url('techbyte', docs[i].path)))
      docs.splice(i, 1);
  }
  for (const doc of docs) {
    for (const item of [...doc.links, ...doc.blocks.filter((block) => block.kind === 'link')]) {
      if (removedGuides.has(item.url)) {
        item.url = removedGuides.get(item.url);
        item.label = 'Uma experiência no Caderno da Lia';
      }
    }
  }
  const headlines = {
    'rubber-duck': 'Por que um pato de borracha ajuda a depurar código?',
    javascript: 'JavaScript: da primeira função a uma interface que responde',
    gpu: 'Placa de vídeo: memória não é tudo',
    phone: 'Celular novo: seis critérios além da câmera',
    wifi: 'O Wi-Fi conecta, mas a internet não abre. E agora?',
    ai: 'Inteligência artificial: como avaliar uma resposta convincente',
    router: 'Mais antenas significam um roteador melhor?',
    linux: 'Linux: entenda o terminal antes de copiar um comando',
    nexora: 'Nexora: onde encontrar especificações e suporte',
    usb: 'USB-C por fora, capacidades diferentes por dentro',
  };
  for (const doc of docs.filter((d) => d.brandId === 'techbyte' && d.path.startsWith('/guias/'))) {
    doc.title = headlines[doc.entities[0]] ?? doc.title;
    if (doc.entities[0] === 'rubber-duck') {
      doc.category = 'Guias';
      doc.summary =
        'Explicar um problema em voz alta pode revelar o passo que ficou implícito. O brinquedo é apenas um ouvinte paciente.';
      doc.blocks = [
        heading('O problema, em voz alta'),
        paragraph(topics[0][6]),
        heading('Um passo de cada vez'),
        paragraph(
          'Comece dizendo o que o programa deveria fazer. Em seguida, descreva a entrada, cada transformação e o resultado observado. Se a explicação depender de “isso deve funcionar”, pare nesse ponto e confira a hipótese.',
        ),
        heading('Quando chamar outra pessoa'),
        paragraph(
          'O método ajuda a organizar o raciocínio, mas não substitui revisão, testes ou a experiência de outra pessoa. Anote o ponto exato em que a explicação deixa de coincidir com o resultado.',
        ),
      ];
    }
  }
  return {
    id: 'web-core',
    ads: [
      {
        id: 'x2-lancamento',
        advertiser: 'Nexora',
        documentId: 'web-shopnow-ofertas-nexphone-x2',
        title: 'NexPhone X2: conheça a oferta',
        description: '128 GB e bateria de 4.600 mAh. Consulte a disponibilidade na ShopNow.',
        keywords: ['celular', 'nexphone', 'nexora'],
        targeting: ['SEARCH', 'EDITORIAL', 'BLOG', 'SOCIAL', 'VIDEO', 'COMMERCE'],
        quality: 85,
        fraudRisk: 0,
        budgetCents: 5000,
        clickCostCents: 25,
        startsAt: 600,
        endsAt: 1800,
      },
      {
        id: 'aprenda-programacao',
        advertiser: 'Educa+',
        documentId: 'web-educa-home',
        title: 'Aprenda no seu ritmo com Educa+',
        description: 'Percursos gratuitos com aulas, exercícios e progresso salvo.',
        keywords: ['javascript', 'programacao', 'curso'],
        targeting: ['SEARCH', 'EDITORIAL', 'BLOG', 'SOCIAL', 'VIDEO', 'EDUCATION', 'FORUM'],
        quality: 90,
        fraudRisk: 0,
        budgetCents: 20000,
        clickCostCents: 10,
        startsAt: 0,
        endsAt: 864000,
      },
    ],
    version: 3,
    seed: 2317,
    brands,
    domains: [
      ...brands.map((b) => ({ host: b.domain, brandId: b.id, redirect: null })),
      { host: 'educa.com', brandId: 'educa', redirect: 'educamais.com' },
      { host: 'nexora.tech', brandId: 'nexora', redirect: 'nexora.com' },
    ],
    entities,
    documents: docs.map((d) =>
      d.brandId === 'shopnow' && d.entities.includes('usb') && d.detail.kind === 'product'
        ? { ...d, blocks: [], materializer: 'product' }
        : d,
    ),
    events: [
      {
        id: 'ORION_ACCESS_REVIEW',
        requiredFlag: 'SESSION_1_COMPLETE',
        afterSeconds: 0,
        publish: [eventNews.id, eventThread.id],
        remove: [],
        offers: [],
      },
      ...timeline,
    ],
  };
}
