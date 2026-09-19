import { appAddress, apps } from './goggle-model';
import { GoggleIcon } from './GoggleIcon';

export function GoggleAppsMenu({ navigate }: { navigate: (address: string) => void }) {
  return (
    <div className="goggle-apps-menu" aria-label="Aplicativos Goggle">
      {apps.map((app) => (
        <button key={app.id} onClick={() => navigate(appAddress(app.id))}>
          <span className={`goggle-app-symbol goggle-app-${app.id}`}>
            <GoggleIcon name={app.icon} />
          </span>
          <span>{app.name}</span>
          <small>{app.status === 'IMPLEMENTED' ? 'Disponível' : 'Em breve'}</small>
        </button>
      ))}
    </div>
  );
}
