import { useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { useAppSettings } from '../../lib/app-settings';
import { KaliInstallerBoot } from './KaliInstallerBoot';
import { KaliSystemBoot } from './KaliSystemBoot';
import { InstallerList, InstallerProgress } from './InstallerWidgets';
import {
  capacity,
  createPartitions,
  disks,
  formatSize,
  partitionMethods,
  partitionSchemes,
  validatePartitions,
} from './installer-model';
import type { Partition, SystemSetupValues } from './installer-model';
import { help, progressStages, titles } from './installer-pages';
import type { Stage } from './installer-pages';
import { saveInstallerCapture } from './installer-capture';

export type { SystemSetupValues } from './installer-model';

export function NewSystemSetup({
  initialHostname,
  initialUsername,
  onCancel,
  onComplete,
}: {
  initialHostname: string;
  initialUsername: string;
  onCancel: () => void;
  onComplete: (values: SystemSetupValues) => void;
}) {
  const [boot, setBoot] = useState<'entry' | 'install' | 'finish'>('entry');
  const [stage, setStage] = useState<Stage>('language');
  const [history, setHistory] = useState<Stage[]>([]);
  const [error, setError] = useState('');
  const [showHelp, setShowHelp] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [confirmPassword, setConfirmPassword] = useState('');
  const [secret, setSecret] = useState('');
  const [confirmSecret, setConfirmSecret] = useState('');
  const [selectedAction, setSelectedAction] = useState('finish');
  const [editing, setEditing] = useState(-1);
  const [draft, setDraft] = useState<Partition>({ size: 1, fs: 'ext4', mount: '' });
  const [software, setSoftware] = useState({
    desktop: true,
    xfce: true,
    gnome: false,
    kde: false,
    tools: true,
    top10: true,
    recommended: true,
  });
  const screen = useRef<HTMLElement>(null);
  const appSettings = useAppSettings.getState().settings;
  const [values, setValues] = useState<SystemSetupValues>({
    language: 'pt-BR',
    location: 'Brasil',
    keyboard: 'br-abnt2',
    network: 'eth0',
    networkSsid: 'Casa',
    networkEnabled: true,
    hostname: initialHostname || 'lifeos',
    domain: 'localdomain',
    fullName: '',
    username: initialUsername || 'kali',
    password: '',
    timezone: 'America/Sao_Paulo',
    disk: 'nvme0n1',
    partition: 'guided-largest',
    partitionScheme: 'all',
    partitions: [],
    storage: 'plain',
    writeChanges: false,
    softwareProfile: 'standard',
    desktopEnvironment: 'xfce',
    softwareTools: 'top10,default',
    displayMode: appSettings.fullscreen ? 'fullscreen' : 'windowed',
    resolution: appSettings.resolution,
  });
  const update = <K extends keyof SystemSetupValues>(key: K, value: SystemSetupValues[K]) => {
    setValues((current) => ({ ...current, [key]: value }));
    setError('');
  };
  const go = (next: Stage) => {
    if (!progressStages[stage]) {
      setHistory((current) => [...current, stage]);
    }
    setStage(next);
    setError('');
    setShowHelp(false);
  };
  const previous = () => {
    if (!history.length) {
      setBoot('entry');
      return;
    }
    setStage(history[history.length - 1]);
    setHistory(history.slice(0, -1));
    setError('');
    setShowHelp(false);
  };
  useEffect(() => {
    if (boot === 'install' && !progressStages[stage]) {
      screen.current
        ?.querySelector<HTMLElement>(
          '.installer-body input, .installer-body [role="listbox"], .installer-continue',
        )
        ?.focus();
    }
  }, [stage, boot]);
  const prepare = (nextValues = values) => {
    setValues({ ...nextValues, partitions: createPartitions(nextValues), writeChanges: false });
    setSelectedAction('finish');
    go('partitioning');
  };
  const openPartition = (index: number) => {
    setEditing(index);
    setDraft(
      index < 0
        ? {
            size: Math.max(
              0.1,
              capacity(values) - values.partitions.reduce((sum, p) => sum + p.size, 0),
            ),
            fs: 'ext4',
            mount: '',
          }
        : { ...values.partitions[index] },
    );
    go('edit');
  };
  const overviewAction = (action: string) => {
    if (action === 'disk-heading') {
      go('disk');
    } else if (action === 'other-disk' || action === 'other-part') {
      update('disk', values.disk === 'nvme0n1' ? 'sda' : 'nvme0n1');
      go('disk');
    } else if (action === 'storage-heading') {
      go('storage');
    } else if (action === 'existing') {
      setError(
        'Esta partição será preservada. Para usar todo o disco, volte ao particionamento guiado.',
      );
    } else if (action === 'guided') {
      go('method');
    } else if (action === 'undo') {
      update('partitions', []);
      go('method');
    } else if (action === 'free') {
      if (capacity(values) - values.partitions.reduce((sum, p) => sum + p.size, 0) < 0.1) {
        setError(
          'Não há espaço livre suficiente. Reduza ou exclua uma partição para liberar espaço.',
        );
        return;
      }
      openPartition(-1);
    } else if (action.startsWith('part-')) {
      openPartition(Number(action.slice(5)));
    } else if (['raid1', 'lvm', 'encrypted', 'iscsi'].includes(action)) {
      setValues((current) => ({
        ...current,
        partition: 'guided-disk',
        storage: action as SystemSetupValues['storage'],
      }));
      go('storage');
    } else if (action === 'finish') {
      const issue = validatePartitions(values);
      if (issue) {
        setError(issue);
        return;
      }
      update('writeChanges', false);
      go('confirm');
    }
  };
  const next = () => {
    switch (stage) {
      case 'language':
        go('location');
        break;
      case 'location':
        go('keyboard');
        break;
      case 'keyboard':
        go('media');
        break;
      case 'network':
        update('networkEnabled', true);
        go(values.network === 'wlan0' ? 'wifi' : 'link');
        break;
      case 'wifi':
        if (values.networkSsid !== 'Casa' && values.networkEnabled) {
          setError(
            'A rede selecionada está protegida. Selecione Casa ou continue sem configurar a rede.',
          );
          return;
        }
        go(values.networkEnabled ? 'link' : 'hostname');
        break;
      case 'hostname':
        if (!/^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,22}[a-zA-Z0-9])?$/.test(values.hostname)) {
          setError('Use um nome de até 24 caracteres, com letras, números e hífen, sem espaços.');
          return;
        }
        go('domain');
        break;
      case 'domain':
        if (!/^[a-zA-Z0-9.-]{1,48}$/.test(values.domain)) {
          setError('Informe um domínio válido, como localdomain.');
          return;
        }
        go('fullname');
        break;
      case 'fullname':
        if (!values.fullName.trim() || values.fullName.length > 80) {
          setError('Informe o nome completo do usuário, com até 80 caracteres.');
          return;
        }
        go('username');
        break;
      case 'username':
        if (!/^[a-z][a-z0-9_-]{0,23}$/.test(values.username)) {
          setError('O usuário deve começar com uma letra minúscula e ter até 24 caracteres.');
          return;
        }
        go('password');
        break;
      case 'password':
        if (
          values.password.length < 4 ||
          values.password.length > 128 ||
          values.password !== confirmPassword
        ) {
          setError('As senhas devem ser iguais e ter de 4 a 128 caracteres.');
          return;
        }
        go('timezone');
        break;
      case 'timezone':
        go('method');
        break;
      case 'method':
        update(
          'storage',
          values.partition === 'guided-lvm'
            ? 'lvm'
            : values.partition === 'guided-encrypted'
              ? 'encrypted'
              : 'plain',
        );
        go('disk');
        break;
      case 'disk':
        if (values.partition === 'manual') {
          update('partitions', []);
          setSelectedAction('free');
          go('overview');
        } else {
          go('scheme');
        }
        break;
      case 'scheme':
        if (values.storage === 'encrypted') {
          go('encryption');
        } else {
          prepare();
        }
        break;
      case 'encryption':
        if (secret.length < 8 || secret !== confirmSecret) {
          setError('A frase secreta deve ter pelo menos 8 caracteres e ser igual nos dois campos.');
          return;
        }
        setSecret('');
        setConfirmSecret('');
        prepare();
        break;
      case 'overview':
        overviewAction(selectedAction);
        break;
      case 'edit': {
        if (
          !Number.isFinite(draft.size) ||
          draft.size <= 0 ||
          !draft.mount ||
          (draft.fs !== 'swap' && !draft.mount.startsWith('/'))
        ) {
          setError('Informe tamanho e ponto de montagem válidos.');
          return;
        }
        const parts = [...values.partitions];
        if (editing < 0) {
          parts.push(draft);
        } else {
          parts[editing] = draft;
        }
        if (parts.reduce((sum, p) => sum + p.size, 0) > capacity(values) + 0.001) {
          setError('O tamanho ultrapassa o espaço disponível.');
          return;
        }
        update('partitions', parts);
        setSelectedAction('finish');
        go('overview');
        break;
      }
      case 'storage':
        if (values.storage === 'encrypted') {
          go('encryption');
        } else {
          prepare();
        }
        break;
      case 'confirm':
        if (!values.writeChanges) {
          go('overview');
        } else {
          go('format');
        }
        break;
      case 'software':
        setValues((current) => ({
          ...current,
          desktopEnvironment: [
            software.xfce ? 'xfce' : '',
            software.gnome ? 'gnome' : '',
            software.kde ? 'kde' : '',
          ]
            .filter(Boolean)
            .join(','),
          softwareTools: [software.top10 ? 'top10' : '', software.recommended ? 'default' : '']
            .filter(Boolean)
            .join(','),
          softwareProfile: software.recommended ? 'standard' : 'minimal',
        }));
        go('packages');
        break;
      case 'complete':
        go('finishing');
        break;
      default:
        break;
    }
  };
  const field = (
    label: string,
    key: 'hostname' | 'domain' | 'fullName' | 'username' | 'password',
    type = 'text',
  ) => (
    <label className="installer-field">
      <span>{label}</span>
      <input
        type={type}
        value={values[key]}
        onChange={(event) => update(key, event.target.value)}
        autoComplete="off"
        spellCheck={false}
      />
    </label>
  );
  const list = <
    K extends
      | 'language'
      | 'location'
      | 'keyboard'
      | 'network'
      | 'networkSsid'
      | 'timezone'
      | 'partition'
      | 'disk'
      | 'partitionScheme'
      | 'storage',
  >(
    key: K,
    label: string,
    options: readonly (readonly [SystemSetupValues[K], string])[],
  ) => (
    <InstallerList
      label={label}
      options={options}
      value={values[key]}
      onChange={(value) => update(key, value)}
    />
  );
  let content: ReactNode;
  switch (stage) {
    case 'language':
      content = (
        <>
          <p>
            Escolha o idioma a ser usado no processo de instalação. O idioma selecionado também será
            o idioma padrão do sistema instalado.
          </p>
          {list('language', 'Idioma:', [['pt-BR', 'Português (Brasil) — Português do Brasil']])}
        </>
      );
      break;
    case 'location':
      content = (
        <>
          <p>
            A localização selecionada será usada para definir seu fuso horário e ajudar a escolher
            as configurações regionais do sistema. Normalmente, este deve ser o país onde você mora.
          </p>
          <p>Esta é uma lista baseada no idioma selecionado.</p>
          {list('location', 'País, território ou área:', [['Brasil', 'Brasil']])}
        </>
      );
      break;
    case 'keyboard':
      content = list('keyboard', 'Mapa de teclado a usar:', [
        ['br-abnt2', 'Brasileiro (ABNT2)'],
        ['br-abnt', 'Brasileiro (ABNT)'],
        ['us-intl', 'Inglês americano (internacional)'],
      ]);
      break;
    case 'network':
      content = (
        <>
          <p>
            Seu sistema possui várias interfaces de rede. Escolha a interface principal a ser usada
            durante a instalação. Se possível, a primeira interface conectada encontrada foi
            selecionada.
          </p>
          {list('network', 'Interface de rede principal:', [
            ['eth0', 'eth0: Intel Corporation Ethernet Connection (7) I219-V'],
            ['wlan0', 'wlan0: Rede sem fio (802.11x)'],
          ])}
        </>
      );
      break;
    case 'wifi':
      content = (
        <>
          <p>Selecione a rede sem fio a ser usada durante a instalação.</p>
          {list('networkSsid', 'Nome da rede sem fio (ESSID):', [
            ['Casa', 'Casa — WPA2 — sinal forte'],
            ['Oficina antiga', 'Oficina antiga — WEP — sinal médio'],
            ['Vizinho_5G', 'Vizinho_5G — WPA2 — sinal fraco'],
          ])}
          <label className="installer-check">
            <input
              type="checkbox"
              checked={!values.networkEnabled}
              onChange={(event) => update('networkEnabled', !event.target.checked)}
            />
            Não configurar a rede agora
          </label>
        </>
      );
      break;
    case 'hostname':
      content = (
        <>
          <p>Por favor, informe o nome deste computador.</p>
          <p>
            O nome do computador identifica o sistema. Pode ser uma palavra que forneça um nome
            único na rede local ou um nome de domínio completo (FQDN), que combina o nome do
            computador e o domínio, separados por pontos.
          </p>
          <p>
            Se você não souber qual nome usar, consulte o administrador da rede. Em uma rede
            doméstica, você pode escolher o nome.
          </p>
          {field('Nome do computador:', 'hostname')}
        </>
      );
      break;
    case 'domain':
      content = (
        <>
          <p>
            O nome de domínio é a parte do seu endereço na Internet à direita do nome do computador.
            Geralmente termina em .com, .net, .edu ou .org. Se estiver configurando uma rede
            doméstica, você pode inventar um nome, mas use o mesmo domínio em todos os computadores.
          </p>
          {field('Nome de domínio:', 'domain')}
        </>
      );
      break;
    case 'fullname':
      content = (
        <>
          <p>
            Uma conta de usuário será criada para você usar no lugar da conta root nas atividades
            que não exigem privilégios administrativos.
          </p>
          <p>
            Informe o nome real deste usuário. Esta informação será usada, por exemplo, como
            identificação padrão em e-mails enviados por este usuário e em programas que exibem ou
            utilizam seu nome real. Seu nome completo é uma boa escolha.
          </p>
          {field('Nome completo do novo usuário:', 'fullName')}
        </>
      );
      break;
    case 'username':
      content = (
        <>
          <p>
            Escolha um nome de usuário para a nova conta. Seu primeiro nome é uma boa escolha. O
            nome deve começar com uma letra minúscula, que pode ser seguida por números e outras
            letras minúsculas.
          </p>
          {field('Nome de usuário para sua conta:', 'username')}
        </>
      );
      break;
    case 'password':
      content = (
        <>
          <p>Certifique-se de escolher uma senha forte que não possa ser adivinhada.</p>
          {field(
            'Escolha uma senha para o novo usuário:',
            'password',
            showPassword ? 'text' : 'password',
          )}
          <label className="installer-check">
            <input
              type="checkbox"
              checked={showPassword}
              onChange={(event) => setShowPassword(event.target.checked)}
            />
            Mostrar senha em texto claro
          </label>
          <p>Informe novamente a mesma senha para verificar se foi digitada corretamente.</p>
          <label className="installer-field">
            <span>Digite a senha novamente para verificação:</span>
            <input
              type={showConfirm ? 'text' : 'password'}
              value={confirmPassword}
              onChange={(event) => setConfirmPassword(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label className="installer-check">
            <input
              type="checkbox"
              checked={showConfirm}
              onChange={(event) => setShowConfirm(event.target.checked)}
            />
            Mostrar confirmação em texto claro
          </label>
        </>
      );
      break;
    case 'timezone':
      content = (
        <>
          <p>
            Se o fuso horário desejado não estiver listado, volte à etapa de localização e selecione
            o país que utiliza o fuso desejado (o país onde você mora ou está localizado).
          </p>
          {list('timezone', 'Selecione seu fuso horário:', [
            ['America/Sao_Paulo', 'São Paulo — horário de Brasília'],
            ['America/Manaus', 'Manaus — horário do Amazonas'],
            ['America/Belem', 'Belém — horário do Pará'],
          ])}
        </>
      );
      break;
    case 'method':
      content = (
        <>
          <p>
            O instalador pode guiá-lo no particionamento de um disco (usando diferentes esquemas
            padrão) ou, se preferir, você pode fazê-lo manualmente. Com o particionamento guiado,
            você ainda poderá revisar e personalizar os resultados.
          </p>
          <p>
            Se escolher o particionamento guiado de um disco inteiro, a próxima etapa perguntará
            qual disco deve ser usado.
          </p>
          {list('partition', 'Método de particionamento:', partitionMethods)}
        </>
      );
      break;
    case 'disk':
      content = (
        <>
          <p>
            {values.partition === 'guided-largest'
              ? 'Somente o maior espaço livre contínuo será usado. Os dados nas partições existentes serão mantidos.'
              : 'Observe que todos os dados no disco selecionado serão apagados, mas somente depois de você confirmar que realmente deseja fazer as alterações.'}
          </p>
          {list('disk', 'Selecione o disco a particionar:', disks)}
        </>
      );
      break;
    case 'scheme':
      content = (
        <>
          <p>Selecionado para particionamento:</p>
          <p>{disks.find(([key]) => key === values.disk)?.[1]}</p>
          <p>
            O disco pode ser particionado usando um dos seguintes esquemas. Se estiver em dúvida,
            escolha o primeiro.
          </p>
          {list('partitionScheme', 'Esquema de particionamento:', partitionSchemes)}
        </>
      );
      break;
    case 'encryption':
      content = (
        <>
          <p>
            Escolha uma frase secreta para o volume criptografado. Digite a mesma frase nos dois
            campos para confirmar.
          </p>
          <label className="installer-field">
            <span>Frase secreta de criptografia:</span>
            <input
              type="password"
              value={secret}
              onChange={(event) => setSecret(event.target.value)}
            />
          </label>
          <label className="installer-field">
            <span>Repita a frase secreta:</span>
            <input
              type="password"
              value={confirmSecret}
              onChange={(event) => setConfirmSecret(event.target.value)}
            />
          </label>
        </>
      );
      break;
    case 'overview': {
      const free = Math.max(
        0,
        capacity(values) - values.partitions.reduce((sum, p) => sum + p.size, 0),
      );
      const rows: [string, string][] = [
        ['guided', '    Particionamento guiado'],
        ['raid1', '    Configurar RAID por software'],
        ['lvm', '    Configurar o Gerenciador de Volumes Lógicos'],
        ['encrypted', '    Configurar volumes criptografados'],
        ['iscsi', '    Configurar volumes iSCSI'],
        ['disk-heading', `\n▾  ${disks.find(([key]) => key === values.disk)?.[1] ?? ''}`],
      ];
      if (values.partition === 'guided-largest') {
        rows.push([
          'existing',
          `       ▸     ${values.disk === 'nvme0n1' ? '300,1' : '32,2'} GB     ntfs     Dados existentes (manter)`,
        ]);
      }
      if (values.storage !== 'plain') {
        rows.push([
          'storage-heading',
          `       ▾     ${values.storage === 'lvm' ? 'Grupo de volumes kali-vg' : values.storage === 'encrypted' ? 'Volume criptografado / LVM kali-vg' : values.storage === 'raid1' ? 'RAID 1 /dev/md0' : 'Disco iSCSI do laboratório'}`,
        ]);
      }
      values.partitions.forEach((p, index) =>
        rows.push([
          `part-${index}`,
          `       ▸     #${index + 1}       ${formatSize(p.size).padStart(10)}       f    ${p.fs.padEnd(6)}    ${p.mount}`,
        ]),
      );
      rows.push(
        ['free', `       ▸     ${formatSize(free)}       ESPAÇO LIVRE`],
        ['other-disk', `▾  ${disks.find(([key]) => key !== values.disk)?.[1] ?? ''}`],
        [
          'other-part',
          `       ▸     #1       ${values.disk === 'nvme0n1' ? '64,2' : '500,1'} GB       fat32`,
        ],
        ['undo', '\n    Desfazer as alterações nas partições'],
        ['finish', '    Finalizar o particionamento e escrever as mudanças no disco'],
      );
      content = (
        <>
          <p className="installer-explanation">
            Esta é uma visão geral das partições e pontos de montagem configurados. Selecione uma
            partição para modificar suas configurações (sistema de arquivos, ponto de montagem etc.)
            ou um espaço livre para criar partições.
          </p>
          <InstallerList
            label="Partições e pontos de montagem"
            options={rows}
            value={selectedAction}
            onChange={setSelectedAction}
            onActivate={overviewAction}
          />
        </>
      );
      break;
    }
    case 'edit':
      content = (
        <>
          <p>
            {editing < 0
              ? 'Criar uma nova partição no espaço livre.'
              : `Configurar a partição #${editing + 1}.`}
          </p>
          <label className="installer-field">
            <span>Tamanho da partição (GB):</span>
            <input
              type="number"
              min="0.1"
              step="any"
              value={draft.size}
              onChange={(event) => setDraft({ ...draft, size: Number(event.target.value) })}
            />
          </label>
          <label className="installer-field">
            <span>Usar como:</span>
            <select
              value={draft.fs}
              onChange={(event) =>
                setDraft({
                  ...draft,
                  fs: event.target.value,
                  mount:
                    event.target.value === 'ESP'
                      ? '/boot/efi'
                      : event.target.value === 'swap'
                        ? 'swap'
                        : draft.mount === 'swap'
                          ? ''
                          : draft.mount,
                })
              }
            >
              {['ext4', 'ext3', 'ext2', 'btrfs', 'xfs', 'FAT32', 'FAT16', 'swap', 'ESP'].map(
                (fs) => (
                  <option key={fs}>{fs}</option>
                ),
              )}
            </select>
          </label>
          <label className="installer-field">
            <span>Ponto de montagem:</span>
            <input
              value={draft.mount}
              onChange={(event) => setDraft({ ...draft, mount: event.target.value })}
            />
          </label>
          {editing >= 0 && (
            <button
              type="button"
              onClick={() => {
                update(
                  'partitions',
                  values.partitions.filter((_, i) => i !== editing),
                );
                go('overview');
              }}
            >
              Excluir a partição
            </button>
          )}
        </>
      );
      break;
    case 'storage':
      content = (
        <>
          <p>
            Selecione o modo de armazenamento. As partições propostas serão recalculadas antes da
            confirmação.
          </p>
          {list('storage', 'Modo de armazenamento:', [
            ['plain', 'Partições comuns'],
            ['lvm', 'Gerenciador de Volumes Lógicos (LVM)'],
            ['encrypted', 'Volumes criptografados com LVM'],
            ['raid1', 'RAID 1 — espelhamento dos dois discos'],
            ['iscsi', 'iSCSI — disco de laboratório'],
          ])}
        </>
      );
      break;
    case 'confirm':
      content = (
        <>
          <p>
            Se continuar, as alterações listadas abaixo serão gravadas nos discos. Caso contrário,
            você poderá fazer outras alterações manualmente.
          </p>
          <p>
            AVISO: isto destruirá todos os dados nas partições removidas e nas partições que serão
            formatadas.
          </p>
          <p>
            As tabelas de partições dos seguintes dispositivos serão alteradas:
            <br />
            {'  '}/dev/{values.disk}
          </p>
          <p className="installer-format-list">
            As seguintes partições serão formatadas:
            <br />
            {values.partitions.map((p, i) => (
              <span key={i}>
                {'  '}partição #{i + 1} de /dev/{values.disk} como {p.fs}
                <br />
              </span>
            ))}
          </p>
          <span className="installer-label">Escrever as alterações nos discos?</span>
          {[false, true].map((yes) => (
            <label className="installer-radio" key={String(yes)}>
              <input
                type="radio"
                name="write"
                checked={values.writeChanges === yes}
                onChange={() => update('writeChanges', yes)}
              />
              {yes ? 'Sim' : 'Não'}
            </label>
          ))}
        </>
      );
      break;
    case 'software':
      content = (
        <>
          <p>
            No momento, apenas o núcleo do sistema está instalado. As seleções padrão abaixo
            instalarão o Kali Linux com seu ambiente de área de trabalho e suas ferramentas padrão.
          </p>
          <p>
            Você pode personalizá-lo escolhendo outro ambiente de área de trabalho ou outra coleção
            de ferramentas.
          </p>
          <span className="installer-label">Escolha o software a instalar:</span>
          <div className="installer-list installer-software">
            {(
              [
                ['desktop', 'Ambiente de área de trabalho [selecionar este item não tem efeito]'],
                ['xfce', '… Xfce (ambiente de área de trabalho padrão do Kali)'],
                ['gnome', '… GNOME'],
                ['kde', '… KDE Plasma'],
                ['tools', 'Coleção de ferramentas [selecionar este item não tem efeito]'],
                ['top10', '… top10 — as 10 ferramentas mais populares'],
                [
                  'recommended',
                  '… default — ferramentas recomendadas (disponíveis no sistema live)',
                ],
              ] as const
            ).map(([key, label]) => (
              <label className="installer-check" key={key}>
                <input
                  type="checkbox"
                  checked={software[key]}
                  onChange={(event) => setSoftware({ ...software, [key]: event.target.checked })}
                />
                {label}
              </label>
            ))}
          </div>
        </>
      );
      break;
    case 'complete':
      content = (
        <div className="installer-complete">
          <span className="installer-info" aria-hidden="true">
            i
          </span>
          <div>
            <span className="installer-label">Instalação concluída</span>
            <p>
              A instalação foi concluída. Agora é hora de iniciar seu novo sistema. Certifique-se de
              remover a mídia de instalação para iniciar o novo sistema, em vez de reiniciar a
              instalação.
            </p>
            <p>Selecione &lt;Continuar&gt; para reiniciar.</p>
          </div>
        </div>
      );
      break;
    default:
      content = null;
  }
  if (boot === 'entry') {
    return <KaliInstallerBoot onContinue={() => setBoot('install')} onCancel={onCancel} />;
  }
  if (boot === 'finish') {
    return <KaliSystemBoot onContinue={() => onComplete(values)} />;
  }
  const progress = progressStages[stage];
  return (
    <section
      ref={screen}
      className="kali-setup"
      aria-label="Instalador do Kali Linux"
      data-stage={stage}
    >
      <header className="installer-banner" aria-label="Kali Linux" />
      <h2 className="installer-title">{titles[stage]}</h2>
      {progress ? (
        <InstallerProgress
          key={stage}
          label={
            stage === 'link'
              ? `Detectando a conexão em ${values.network}; aguarde…`
              : progress.label
          }
          details={progress.details}
          onComplete={() => {
            if (stage === 'finishing') {
              setBoot('finish');
            } else {
              go(progress.next);
            }
          }}
        />
      ) : (
        <form
          id="kali-setup-form"
          className="installer-body"
          onSubmit={(event) => {
            event.preventDefault();
            next();
          }}
          key={stage}
        >
          {content}
          {error && (
            <p className="installer-error" role="alert">
              {error}
            </p>
          )}
        </form>
      )}
      <footer className="installer-footer">
        {!progress && (
          <>
            <button
              type="button"
              aria-label="Capturar tela"
              title="Capturar tela"
              onClick={() => {
                if (screen.current) {
                  void saveInstallerCapture(screen.current).catch(() =>
                    setError('Não foi possível salvar a captura de tela.'),
                  );
                }
              }}
            >
              Captura
            </button>
            {help[stage] && (
              <button type="button" onClick={() => setShowHelp(true)}>
                Ajuda
              </button>
            )}
          </>
        )}
        <span />
        {stage === 'link' && (
          <button type="button" onClick={() => go('network')}>
            Cancelar
          </button>
        )}
        {!progress && (
          <>
            {!['confirm', 'software'].includes(stage) && (
              <button type="button" onClick={previous}>
                Voltar
              </button>
            )}
            <button type="submit" form="kali-setup-form" className="installer-continue">
              Continuar
            </button>
          </>
        )}
      </footer>
      {showHelp && (
        <div
          className="installer-help-backdrop"
          onKeyDown={(event) => {
            if (event.key === 'Escape') {
              setShowHelp(false);
            }
          }}
        >
          <section
            role="dialog"
            aria-modal="true"
            aria-label="Ajuda do instalador"
            className="installer-help"
          >
            <h3>Ajuda — {titles[stage]}</h3>
            <dl>
              {help[stage]?.map(([term, description]) => (
                <div key={term}>
                  <dt>{term}</dt>
                  <dd>{description}</dd>
                </div>
              ))}
            </dl>
            <button type="button" autoFocus onClick={() => setShowHelp(false)}>
              Fechar
            </button>
          </section>
        </div>
      )}
    </section>
  );
}
