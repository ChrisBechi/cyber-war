import type { BrowserPage } from '../../lib/api';
import { useWindows } from '../../lib/window-store';

export function VigiliaDownload({
  busy,
  onDownload,
}: {
  page?: BrowserPage;
  busy: boolean;
  onDownload: () => void;
}) {
  const openGame = () => useWindows.getState().open('vigilia');
  return (
    <div className="vigilia-download-page">
      <header className="vigilia-download-hero">
        <div>
          <span className="vigilia-download-kicker">SECTOR IX / RELEASES</span>
          <h1>SECTOR IX — Protocolo Zero</h1>
          <p>
            Um shooter 2D de infiltração, com três operações, arsenal persistente e progresso salvo
            dentro da instalação do Cyber War.
          </p>
        </div>
        <div className="vigilia-download-mark">
          VZ<span>0</span>
        </div>
      </header>

      <section className="vigilia-download-grid">
        <article>
          <span className="vigilia-download-label">PACOTE LINUX</span>
          <h2>Baixe pelo terminal</h2>
          <p>
            O jogo usa o download virtual do sistema. Os comandos abaixo funcionam no terminal do
            Cyber War e deixam o instalador em <code>~/Downloads</code>.
          </p>
          <pre>{`cd ~/Downloads
wget -O sector-ix-linux.sh https://www.vigilia.org/download/sector-ix-linux.sh
chmod +x sector-ix-linux.sh
bash sector-ix-linux.sh
sector-ix`}</pre>
          <button className="vigilia-download-primary" disabled={busy} onClick={onDownload}>
            {busy ? 'Preparando pacote…' : 'Baixar pacote no Linux virtual'}
          </button>
        </article>

        <aside>
          <span className="vigilia-download-label">INFORMAÇÕES</span>
          <dl>
            <div>
              <dt>Versão</dt>
              <dd>1.0.0</dd>
            </div>
            <div>
              <dt>Formato</dt>
              <dd>Shell + runtime interno</dd>
            </div>
            <div>
              <dt>Modo inicial</dt>
              <dd>Janela</dd>
            </div>
            <div>
              <dt>Fullscreen</dt>
              <dd>Disponível no jogo</dd>
            </div>
          </dl>
          <button className="vigilia-download-secondary" onClick={openGame}>
            Jogar agora no Cyber War →
          </button>
        </aside>
      </section>

      <section className="vigilia-download-install">
        <span className="vigilia-download-label">COMO INSTALAR E RODAR</span>
        <div className="vigilia-download-steps">
          <div>
            <b>01</b>
            <h3>Baixe</h3>
            <p>
              Use <code>wget</code> para salvar o instalador na pasta de downloads.
            </p>
          </div>
          <div>
            <b>02</b>
            <h3>Permita a execução</h3>
            <p>
              <code>chmod +x</code> aplica a permissão de execução do Linux.
            </p>
          </div>
          <div>
            <b>03</b>
            <h3>Instale e execute</h3>
            <p>
              Rode o script e depois digite <code>sector-ix</code> para abrir a operação.
            </p>
          </div>
        </div>
      </section>

      <footer>
        <span>SECTOR IX / PROTOCOLO ZERO</span>
        <span>Conteúdo virtual da campanha · sem acesso ao host</span>
      </footer>
    </div>
  );
}
