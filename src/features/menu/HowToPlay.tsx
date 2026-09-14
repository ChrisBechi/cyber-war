const cards = [
  [
    '01',
    'Terminal',
    'Digite help para consultar os comandos. Use Tab para completar e ↑ / ↓ para acessar o histórico. Investigue cada pista com calma.',
  ],
  [
    '02',
    'Arquivos',
    'Abra pastas, leia documentos e conecte as informações. Os arquivos do computador virtual guardam pistas da história.',
  ],
  [
    '03',
    'Missões',
    'Acompanhe as mensagens e os objetivos. Suas ações e escolhas alteram a investigação e as consequências da campanha.',
  ],
  [
    '04',
    'Save',
    'Use o aplicativo Saves para salvar manualmente. Ao sair no meio de uma missão, a tentativa é descartada; ao concluir, o resultado e as próximas missões ficam disponíveis no slot.',
  ],
  [
    '05',
    'Sandbox',
    'Computadores, redes e ferramentas fazem parte de uma simulação local. Explore o universo do jogo: os comandos não atuam no seu sistema real.',
  ],
];
export function HowToPlay() {
  return (
    <div className="front-help">
      <p className="front-kicker">ANTES DA PRIMEIRA CONEXÃO</p>
      <h1>Como jogar</h1>
      <p>Curiosidade é sua primeira ferramenta.</p>
      <div className="front-help-grid">
        {cards.map(([number, title, description]) => (
          <article key={number}>
            <span>{number} /</span>
            <h2>{title}</h2>
            <p>{description}</p>
          </article>
        ))}
      </div>
    </div>
  );
}
