import { act, useGame } from '../../lib/game-store';
import { toggleFullscreen } from '../../lib/fullscreen';

const startupApps = [
  ['terminal', 'Terminal'],
  ['files', 'Arquivos'],
  ['editor', 'HackPad'],
  ['browser', 'Navegador'],
  ['messages', 'Mensagens'],
] as const;

function readAutostart(value: string | undefined): string[] {
  if (!value) {
    return [];
  }
  try {
    const parsed: unknown = JSON.parse(value);
    return Array.isArray(parsed)
      ? parsed.filter((item): item is string => typeof item === 'string')
      : [];
  } catch {
    return [];
  }
}

export function Settings({ onShowKeyboardHelp }: { onShowKeyboardHelp?: () => void } = {}) {
  const settings = useGame((s) => s.world?.settings);
  const network = useGame((s) => s.world?.network);
  const autostart = readAutostart(settings?.autostartApps);
  const updateAutostart = (id: string, enabled: boolean) => {
    const next = enabled
      ? [...new Set([...autostart, id])]
      : autostart.filter((current) => current !== id);
    act('setting_update', { key: 'autostartApps', value: JSON.stringify(next) });
  };
  return (
    <div className="app-scroll settings">
      <div className="section-heading">
        <div className="eyebrow">PERSONALIZE SEU ESPAÇO</div>
        <h2>Aparência</h2>
      </div>
      <h3>Exibição</h3>
      <button onClick={toggleFullscreen}>Alternar tela cheia · F11</button>
      {onShowKeyboardHelp && <button onClick={onShowKeyboardHelp}>Atalhos de teclado · F1</button>}
      <h3>Rede virtual</h3>
      <label>
        Conexão cabeada · eth0
        <select
          aria-label="Conexão da rede virtual"
          value={String(network?.connected ?? true)}
          onChange={(event) =>
            act('setting_update', { key: 'networkEnabled', value: event.target.value })
          }
        >
          <option value="true">Conectada</option>
          <option value="false">Desconectada</option>
        </select>
      </label>
      <p className="muted">
        Gateway: {network?.gateway}. Os serviços e endereços pertencem ao mundo do jogo.
      </p>
      <h3>Inicialização da sessão</h3>
      <p className="muted">
        A sessão começa limpa. Marque apenas os aplicativos que devem abrir após o login.
      </p>
      <div className="settings-check-list">
        {startupApps.map(([id, label]) => (
          <label key={id}>
            <input
              type="checkbox"
              checked={autostart.includes(id)}
              onChange={(event) => updateAutostart(id, event.target.checked)}
            />{' '}
            {label}
          </label>
        ))}
      </div>
      <p className="muted">
        Para serviços, use no terminal: <code>sudo systemctl enable SERVIÇO</code> ou{' '}
        <code>disable</code>.
      </p>
      <label>
        Tamanho da fonte do terminal{' '}
        <select
          value={settings?.fontSize ?? '14'}
          onChange={(e) => act('setting_update', { key: 'fontSize', value: e.target.value })}
        >
          {[12, 14, 16, 18, 20, 24].map((n) => (
            <option key={n}>{n}</option>
          ))}
        </select>
      </label>
      <h3>Papel de parede</h3>
      <div className="wallpapers">
        {['maze', 'waves'].map((wallpaper) => (
          <button
            key={wallpaper}
            className={settings?.wallpaper === wallpaper ? 'selected' : ''}
            onClick={() => act('setting_update', { key: 'wallpaper', value: wallpaper })}
          >
            <img
              alt={wallpaper === 'maze' ? 'Kali Maze' : 'Ondas azuis'}
              src={`/assets/kali-${wallpaper}.${wallpaper === 'waves' ? 'png' : 'jpg'}`}
            />
          </button>
        ))}
      </div>
      <p className="muted">
        Ctrl+Alt+T abre o terminal. Alt+PageDown troca de janela. F1 mostra todos os atalhos do
        sistema.
      </p>
    </div>
  );
}
