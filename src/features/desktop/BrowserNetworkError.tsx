import { useId, useState } from 'react';
import './browser-network-error.css';

export function BrowserNetworkError({
  address,
  error,
  retry,
}: {
  address: string;
  error: string;
  retry: () => void;
}) {
  const [showHelp, setShowHelp] = useState(false);
  const helpId = useId();
  let hostname: string;
  try {
    hostname = new URL(address).hostname;
  } catch {
    hostname = address.replace(/^https?:\/\//i, '').split(/[/?#]/)[0];
  }

  return (
    <div className="browser-network-error">
      <section className="browser-network-error-layout" aria-label="Erro de conexão">
        <img src="/assets/browser/no-connection.svg" alt="" width="244" height="173" />
        <div className="browser-network-error-content">
          <div role="alert">
            <h1>Servidor não encontrado</h1>
            <p className="browser-network-error-address">
              O Firefox não conseguiu se conectar ao servidor em <strong>{hostname}.</strong>
            </p>
          </div>
          <h2>O que você pode fazer a respeito?</h2>
          <ul>
            <li>
              Verifique se você digitou o endereço do site corretamente e tente novamente em
              instantes.
            </li>
            <li>Verifique sua conexão de rede.</li>
            <li>
              Verificar se o Firefox tem permissão para acessar a web (a conexão pode estar limitada
              por um firewall).
            </li>
          </ul>
          <button
            type="button"
            className="browser-network-error-help"
            aria-expanded={showHelp}
            aria-controls={helpId}
            onClick={() => setShowHelp((visible) => !visible)}
          >
            Saiba mais...
          </button>
          <div id={helpId} className="browser-network-error-details" hidden={!showHelp}>
            <p>Confira a conexão no painel de rede e tente abrir um dos seus favoritos.</p>
            <p>{error.replace(/^Error:\s*/, '')}</p>
          </div>
          <div className="browser-network-error-actions">
            <button type="button" className="browser-network-error-retry" onClick={retry}>
              Tentar novamente
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
