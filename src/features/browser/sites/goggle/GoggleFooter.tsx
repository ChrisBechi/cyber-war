import { GOGGLE_HOME } from './goggle-model';

export function GoggleFooter({ navigate }: { navigate: (address: string) => void }) {
  return (
    <footer className="goggle-footer">
      <div className="goggle-region">Brasil</div>
      <div className="goggle-footer-links">
        <nav aria-label="Sobre a pesquisa">
          <button onClick={() => navigate(`${GOGGLE_HOME}/about`)}>Sobre</button>
          <button onClick={() => navigate(`${GOGGLE_HOME}/how-search-works`)}>
            Como funciona a Pesquisa
          </button>
        </nav>
        <nav aria-label="Políticas do Goggle">
          <button onClick={() => navigate(`${GOGGLE_HOME}/privacy`)}>Privacidade</button>
          <button onClick={() => navigate(`${GOGGLE_HOME}/terms`)}>Termos</button>
        </nav>
      </div>
    </footer>
  );
}
