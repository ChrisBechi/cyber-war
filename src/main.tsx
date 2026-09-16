import React from 'react';
import ReactDOM from 'react-dom/client';

import { App } from './App';
import './styles/global.css';
import './styles/kali.css';
import './styles/boot.css';
import './styles/investigation.css';
import './styles/install.css';

const root = ReactDOM.createRoot(document.getElementById('root')!);
if (import.meta.env.DEV && location.hash === '#archive/qa') {
  void import('./features/files/ArchiveQA').then(({ ArchiveQA }) => root.render(<ArchiveQA />));
} else if (import.meta.env.DEV && location.hash === '#opening/montage') {
  void import('./features/opening/OpeningMontage').then(({ OpeningMontage }) =>
    root.render(<OpeningMontage />),
  );
} else if (import.meta.env.DEV && location.hash === '#opening/edit') {
  void import('./features/opening/OpeningBookends').then(({ OpeningBookends }) =>
    root.render(<OpeningBookends />),
  );
} else if (import.meta.env.DEV && location.hash.startsWith('#opening/')) {
  void import('./features/opening/OpeningScenarioMode').then(({ OpeningScenarioMode }) =>
    root.render(<OpeningScenarioMode />),
  );
} else {
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}
