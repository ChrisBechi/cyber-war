import { useCallback, useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { request } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import { useWindows } from '../../lib/window-store';
import { audioManager } from '../../lib/audio-manager';
import { packetSchema, signalEvidenceSchema, wirelessSchema } from '../../lib/investigation';
import type { Packet, SignalEvidence, WirelessNetwork } from '../../lib/investigation';
import { KaliIcon } from './KaliIcon';
import { softwareCatalog } from '../../lib/software-catalog';

const wireshark = softwareCatalog.entries.find((entry) => entry.package === 'wireshark')!;

export function WirelessInspector() {
  const [networks, setNetworks] = useState<WirelessNetwork[]>([]);
  const [selected, setSelected] = useState('');
  const [evidence, setEvidence] = useState<SignalEvidence | null>(null);
  const [error, setError] = useState('');
  const busy = useGame((state) => state.busy);
  const scan = useCallback((withSound = true) => {
    if (withSound) {
      audioManager.play('ui-click');
    }
    void request('wireless_scan', {}, z.array(wirelessSchema))
      .then((value) => {
        setNetworks(value);
        setError('');
      })
      .catch((reason: unknown) => setError(String(reason)));
  }, []);
  useEffect(() => {
    scan(false);
  }, [scan]);
  const inspect = () => {
    audioManager.play('ui-click');
    void perform('wireless_inspect', { bssid: selected }, signalEvidenceSchema)
      .then((value) => {
        setEvidence(value);
        audioManager.play('ui-download');
      })
      .catch((reason: unknown) => setError(String(reason)));
  };
  return (
    <div className="instrument-app">
      <header className="instrument-toolbar">
        <KaliIcon name="kali-aircrack-ng" size={24} />
        <strong>Redes sem fio</strong>
        <span>wlan0</span>
        <button onClick={() => scan()}>Atualizar redes</button>
      </header>
      <div className="app-scroll">
        <table className="instrument-table">
          <thead>
            <tr>
              <th>SSID</th>
              <th>BSSID</th>
              <th>Canal</th>
              <th>Sinal</th>
              <th>Proteção</th>
              <th>Estações</th>
            </tr>
          </thead>
          <tbody>
            {networks.map((network) => (
              <tr
                key={network.bssid}
                className={selected === network.bssid ? 'selected' : ''}
                onClick={() => {
                  setSelected(network.bssid);
                  setEvidence(null);
                }}
              >
                <td>
                  <button
                    onClick={() => {
                      setSelected(network.bssid);
                      setEvidence(null);
                    }}
                    aria-pressed={selected === network.bssid}
                  >
                    {network.ssid}
                  </button>
                </td>
                <td>{network.bssid}</td>
                <td>{network.channel}</td>
                <td>{network.signal} dBm</td>
                <td>{network.encryption}</td>
                <td>{network.clients}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {!networks.length && (
          <p className="instrument-empty">
            Atualize os sinais da interface para consultar as redes próximas.
          </p>
        )}
        <div className="toolbar">
          <button disabled={!selected || busy} onClick={inspect}>
            Inspecionar sinal
          </button>
        </div>
        {evidence && (
          <section className="signal-evidence">
            <h3>{evidence.organization}</h3>
            <p>{evidence.detail}</p>
            {evidence.capturePath && (
              <>
                <p className="instrument-path">{evidence.capturePath}</p>
                <button
                  onClick={() =>
                    useWindows
                      .getState()
                      .open(`tool:${wireshark.id}`, evidence.capturePath ?? undefined)
                  }
                >
                  Abrir captura
                </button>{' '}
                <button onClick={() => useWindows.getState().open('files', '/home/kali/Downloads')}>
                  Ver nos Downloads
                </button>
              </>
            )}
          </section>
        )}
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
      </div>
      <footer className="app-status">
        {networks.length} redes <span>Interface virtual · monitoramento local</span>
      </footer>
    </div>
  );
}

export function TrafficInspector({ initialPath }: { initialPath?: string }) {
  const [path, setPath] = useState(initialPath ?? '/home/kali/Downloads/orion.capture.json');
  const [packets, setPackets] = useState<Packet[]>([]);
  const [selected, setSelected] = useState<Packet | null>(null);
  const [conversation, setConversation] = useState<Packet[]>([]);
  const [visible, setVisible] = useState(0);
  const [filter, setFilter] = useState('');
  const [error, setError] = useState('');
  const busy = useGame((state) => state.busy);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    if (initialPath) {
      setPath(initialPath);
    }
  }, [initialPath]);
  useEffect(() => {
    if (!conversation.length) {
      return;
    }
    setVisible(0);
    const timers = conversation.map((_, index) =>
      window.setTimeout(() => {
        setVisible(index + 1);
        audioManager.play(index === conversation.length - 1 ? 'ui-alert' : 'ui-terminal');
      }, index * 1150),
    );
    return () => timers.forEach((timer) => window.clearTimeout(timer));
  }, [conversation]);
  const open = () => {
    audioManager.play('ui-click');
    void request('traffic_read', { path }, z.array(packetSchema))
      .then((value) => {
        if (mounted.current) {
          setPackets(value);
          setSelected(value[0] ?? null);
          setConversation([]);
          setError('');
        }
      })
      .catch((reason: unknown) => setError(String(reason)));
  };
  const follow = () => {
    if (!selected) {
      return;
    }
    void perform('traffic_follow', { path, stream: selected.stream }, z.array(packetSchema))
      .then((value) => {
        if (mounted.current) {
          setConversation(value);
        }
      })
      .catch((reason: unknown) => setError(String(reason)));
  };
  const filtered = packets.filter((packet) =>
    `${packet.source} ${packet.destination} ${packet.protocol} ${packet.text}`
      .toLowerCase()
      .includes(filter.toLowerCase()),
  );
  return (
    <div className="instrument-app traffic-inspector">
      <form
        className="instrument-toolbar"
        onSubmit={(event) => {
          event.preventDefault();
          open();
        }}
      >
        <KaliIcon name={wireshark.icon} size={24} />
        <input
          aria-label="Arquivo de captura"
          value={path}
          onChange={(event) => setPath(event.target.value)}
        />
        <button>Abrir captura</button>
      </form>
      <div className="toolbar">
        <label>
          Filtro
          <input
            aria-label="Filtro de pacotes"
            placeholder="Protocolo, endereço ou texto"
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
          />
        </label>
        <button disabled={!selected || busy} onClick={follow}>
          Seguir fluxo {selected?.stream ?? ''}
        </button>
      </div>
      <div className="app-scroll">
        <table className="instrument-table">
          <thead>
            <tr>
              <th>No.</th>
              <th>Tempo</th>
              <th>Origem</th>
              <th>Destino</th>
              <th>Protocolo</th>
              <th>Informação</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((packet) => (
              <tr
                key={packet.number}
                className={selected?.number === packet.number ? 'selected' : ''}
                onClick={() => setSelected(packet)}
              >
                <td>
                  <button onClick={() => setSelected(packet)}>{packet.number}</button>
                </td>
                <td>{packet.time}</td>
                <td>{packet.source}</td>
                <td>{packet.destination}</td>
                <td>{packet.protocol}</td>
                <td>
                  {packet.text.length} bytes · fluxo {packet.stream}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {!packets.length && (
          <p className="instrument-empty">Abra uma captura preservada no computador virtual.</p>
        )}
        {conversation.length > 0 ? (
          <section className="stream-view">
            <header>Fluxo {conversation[0].stream} · comunicação reconstruída</header>
            <div role="log" aria-label="Conteúdo do fluxo">
              {conversation.slice(0, visible).map((packet) => (
                <p
                  key={packet.number}
                  className={packet.source.endsWith('.30') ? 'stream-reply' : ''}
                >
                  <small>
                    {packet.time} · {packet.source}
                  </small>
                  {packet.text}
                </p>
              ))}
            </div>
          </section>
        ) : (
          selected && (
            <pre className="packet-detail">{`Frame ${selected.number}\n${selected.source} → ${selected.destination}\n${selected.protocol} · fluxo ${selected.stream}\n\n${selected.text}`}</pre>
          )
        )}
        {error && (
          <p role="alert" className="error">
            {error}
          </p>
        )}
      </div>
      <footer className="app-status">
        {packets.length} pacotes{' '}
        <span>
          {conversation.length
            ? `${visible}/${conversation.length} fragmentos reconstruídos`
            : 'Captura virtual · leitura de arquivo'}
        </span>
      </footer>
    </div>
  );
}
