// A small authored timeline exercises the shared event engine before adding more brands.
export function worldEvents({ add, url, epoch }) {
  const paragraph = (text) => ({ kind: 'paragraph', text });
  const product = 'web-shopnow-produto-nexphone-x2';
  const productLink = {
    label: 'Consultar o NexPhone X2 na ShopNow',
    url: url('shopnow', '/produto/nexphone-x2'),
  };
  const release = add(
    'nexora',
    '/empresa/campanha-x2',
    'NexPhone X2 participa de campanha de distribuição',
    'A Nexora anuncia uma condição temporária para o X2 no catálogo da ShopNow, sujeita à disponibilidade do lote.',
    'Empresa',
    [
      paragraph(
        'A campanha mantém a configuração de 128 GB e não altera as especificações do aparelho. A condição comercial é publicada pela loja, que também informa a disponibilidade atual.',
      ),
      paragraph(
        'O lote é limitado. Uma alteração de preço ou estoque na loja não representa mudança de modelo; compare o código do produto ao consultar a documentação.',
      ),
    ],
    undefined,
    {
      publishedAt: epoch + 600,
      visual: 'phone',
      entities: ['nexora', 'nexphone-x2'],
      links: [
        productLink,
        { label: 'Especificações do X2', url: url('nexora', '/produtos/nexphone-x2') },
      ],
    },
  );
  const promotion = add(
    'shopnow',
    '/ofertas/nexphone-x2',
    'Condição especial do NexPhone X2',
    'O X2 de 128 GB está em campanha por R$ 1.799,00 enquanto houver unidades no lote.',
    'Tecnologia',
    [
      paragraph(
        'Este lote participa de uma condição temporária. Consulte a página do aparelho para conferir o preço vigente, a disponibilidade e as especificações antes de adicionar ao carrinho.',
      ),
      paragraph(
        'Itens no carrinho acompanham o preço atual da loja. A inclusão não reserva unidades nem congela o valor da oferta.',
      ),
    ],
    undefined,
    { publishedAt: epoch + 600, visual: 'phone', entities: ['nexphone-x2'], links: [productLink] },
  );
  const discussion = add(
    'redditor',
    '/t/x2-ultimas-unidades',
    'O X2 entrou em últimas unidades. Vocês chegaram a comparar?',
    'Vi a campanha na loja e voltei para ler a documentação. Agora a página indica poucas unidades; ainda estou comparando com meu aparelho atual.',
    'Tecnologia',
    [
      paragraph(
        'Meu celular ainda atende, então não quero decidir só pelo aviso de estoque. O que vocês colocam na comparação além do armazenamento? Separei os links de especificação e da loja abaixo.',
      ),
    ],
    { kind: 'thread', community: 'tecnologia', votes: 7, locked: false },
    {
      publishedAt: epoch + 1200,
      author: 'janela_aberta',
      visual: 'phone',
      entities: ['nexphone-x2'],
      links: [
        productLink,
        { label: 'Manual e especificações', url: url('nexora', '/produtos/nexphone-x2') },
      ],
      comments: [
        {
          id: 'x2-comparacao-1',
          author: 'manual_primeiro',
          text: 'Eu compararia o que está faltando no uso de hoje. Uma oferta curta não muda a necessidade de entender o produto.',
          publishedAt: epoch + 1260,
          rating: null,
        },
      ],
    },
  );
  const restock = add(
    'b1',
    '/noticias/reposicao-nexphone-x2',
    'ShopNow registra reposição do NexPhone X2',
    'O aparelho voltou ao catálogo disponível após o encerramento do lote promocional; a loja informa uma nova condição comercial.',
    'Tecnologia',
    [
      paragraph(
        'A página do NexPhone X2 voltou a aceitar inclusões no carrinho. O histórico da oferta registra as mudanças de preço e disponibilidade entre os dois lotes.',
      ),
      paragraph(
        'A reposição mantém a configuração de 128 GB. A Nexora conserva as especificações e os documentos de suporte no endereço do produto.',
      ),
    ],
    undefined,
    {
      publishedAt: epoch + 2400,
      visual: 'phone',
      entities: ['nexora', 'nexphone-x2'],
      links: [productLink],
    },
  );
  const event = (id, afterSeconds, publish, remove, priceCents, stock) => ({
    id,
    afterSeconds,
    requiredFlag: null,
    publish,
    remove,
    offers: [{ documentId: product, priceCents, stock }],
  });
  return [
    event('NEXPHONE_CAMPAIGN_OPEN', 600, [release.id, promotion.id], [], 179900, 'IN_STOCK'),
    event('NEXPHONE_CAMPAIGN_LOW_STOCK', 1200, [discussion.id], [], null, 'LOW_STOCK'),
    event('NEXPHONE_CAMPAIGN_CLOSED', 1800, [], [promotion.id], 189900, 'OUT_OF_STOCK'),
    event('NEXPHONE_RESTOCKED', 2400, [restock.id], [], 184900, 'IN_STOCK'),
  ];
}
