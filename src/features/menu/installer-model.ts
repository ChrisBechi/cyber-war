import type { AppSettings } from '../../lib/app-settings';

export type Partition = { size: number; fs: string; mount: string };
export type SystemSetupValues = {
  language: 'pt-BR';
  location: 'Brasil';
  keyboard: 'br-abnt2' | 'br-abnt' | 'us-intl';
  network: 'eth0' | 'wlan0';
  networkSsid: string;
  networkEnabled: boolean;
  hostname: string;
  domain: string;
  fullName: string;
  username: string;
  password: string;
  timezone: 'America/Sao_Paulo' | 'America/Manaus' | 'America/Belem';
  disk: 'nvme0n1' | 'sda';
  partition: 'guided-largest' | 'guided-disk' | 'guided-lvm' | 'guided-encrypted' | 'manual';
  partitionScheme: 'all' | 'home' | 'var-tmp' | 'server' | 'small';
  partitions: Partition[];
  storage: 'plain' | 'lvm' | 'encrypted' | 'raid1' | 'iscsi';
  writeChanges: boolean;
  softwareProfile: 'standard' | 'minimal';
  desktopEnvironment: string;
  softwareTools: string;
  displayMode: 'fullscreen' | 'windowed';
  resolution: AppSettings['resolution'];
};

export const disks = [
  ['nvme0n1', '/dev/nvme0n1 — 500,1 GB Samsung SSD 970 EVO Plus 500GB'],
  ['sda', 'SCSI8 (0,0,0) (sda) — 64,2 GB Samsung Flash Drive'],
] as const;
export const partitionMethods = [
  ['guided-largest', 'Guiado — usar o maior espaço livre contínuo'],
  ['guided-disk', 'Guiado — usar o disco inteiro'],
  ['guided-lvm', 'Guiado — usar o disco inteiro e configurar LVM'],
  ['guided-encrypted', 'Guiado — usar o disco inteiro e configurar LVM criptografado'],
  ['manual', 'Manual'],
] as const;
export const partitionSchemes = [
  ['all', 'Todos os arquivos em uma partição (recomendado para novos usuários)'],
  ['home', 'Partição /home separada'],
  ['var-tmp', 'Partições /home, /var e /tmp separadas'],
  ['server', 'Partições /var e /srv separadas, swap < 1 GB (para servidores)'],
  ['small', 'Esquema de particionamento para disco pequeno (< 10 GB)'],
] as const;
export const capacity = (values: SystemSetupValues) => {
  if (values.storage === 'raid1') {
    return 64.2;
  }
  if (values.partition === 'guided-largest') {
    return values.disk === 'nvme0n1' ? 200 : 32;
  }
  return values.disk === 'nvme0n1' ? 500.1 : 64.2;
};
export function createPartitions(values: SystemSetupValues): Partition[] {
  const total = capacity(values);
  const swap =
    values.partitionScheme === 'server'
      ? 0.5
      : values.partitionScheme === 'small'
        ? 1
        : total > 100
          ? 17
          : 4;
  const available = total - 1 - swap;
  const root =
    values.partitionScheme === 'all'
      ? available
      : values.partitionScheme === 'small'
        ? 7
        : Math.min(40, available * 0.45);
  const partitions: Partition[] = [
    { size: 1, fs: 'ESP', mount: '/boot/efi' },
    { size: root, fs: 'ext4', mount: '/' },
  ];
  const rest = available - root;
  if (values.partitionScheme === 'home') {
    partitions.push({ size: rest, fs: 'ext4', mount: '/home' });
  }
  if (values.partitionScheme === 'var-tmp') {
    partitions.push(
      { size: rest * 0.7, fs: 'ext4', mount: '/home' },
      { size: rest * 0.2, fs: 'ext4', mount: '/var' },
      { size: rest * 0.1, fs: 'ext4', mount: '/tmp' },
    );
  }
  if (values.partitionScheme === 'server') {
    partitions.push(
      { size: rest * 0.4, fs: 'ext4', mount: '/var' },
      { size: rest * 0.6, fs: 'ext4', mount: '/srv' },
    );
  }
  partitions.push({ size: swap, fs: 'swap', mount: 'swap' });
  return partitions.map((p) => ({ ...p, size: Math.floor(p.size * 1000) / 1000 }));
}
export function validatePartitions(values: SystemSetupValues) {
  const parts = values.partitions;
  if (
    !parts.some(
      (p) =>
        p.mount === '/' && ['ext4', 'ext3', 'ext2', 'btrfs', 'xfs'].includes(p.fs) && p.size >= 7,
    )
  ) {
    return 'Crie uma partição raiz (/) com pelo menos 7 GB e um sistema de arquivos Linux.';
  }
  if (!parts.some((p) => p.fs === 'ESP' && p.mount === '/boot/efi' && p.size >= 0.1)) {
    return 'Crie uma partição de sistema EFI (ESP) de pelo menos 100 MB em /boot/efi.';
  }
  const mounts = parts.map((p) => p.mount).filter((m) => m !== 'swap');
  if (new Set(mounts).size !== mounts.length) {
    return 'Cada ponto de montagem deve ser usado por apenas uma partição.';
  }
  if (
    parts.some(
      (p) =>
        !Number.isFinite(p.size) ||
        p.size <= 0 ||
        !p.mount ||
        /\s/.test(p.mount) ||
        p.mount.includes('..') ||
        (p.fs === 'swap' && p.mount !== 'swap') ||
        (p.fs !== 'swap' && !p.mount.startsWith('/')),
    )
  ) {
    return 'Informe um tamanho positivo e um ponto de montagem válido para cada partição.';
  }
  if (parts.reduce((sum, p) => sum + p.size, 0) > capacity(values) + 0.001) {
    return 'As partições ultrapassam o espaço disponível no disco selecionado.';
  }
  return '';
}
export const formatSize = (size: number) =>
  `${size.toLocaleString('pt-BR', { minimumFractionDigits: 1, maximumFractionDigits: 1 })} GB`;
