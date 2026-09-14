export const titles = {
  language: 'Selecione um idioma',
  location: 'Selecione sua localização',
  keyboard: 'Configurar o teclado',
  media: 'Detectar e montar a mídia de instalação',
  components: 'Carregar componentes do instalador a partir da mídia de instalação',
  network: 'Configurar a rede',
  wifi: 'Configurar a rede',
  link: 'Configurar a rede',
  hostname: 'Configurar a rede',
  domain: 'Configurar a rede',
  fullname: 'Configurar usuários e senhas',
  username: 'Configurar usuários e senhas',
  password: 'Configurar usuários e senhas',
  timezone: 'Configurar o relógio',
  method: 'Particionar discos',
  disk: 'Particionar discos',
  scheme: 'Particionar discos',
  encryption: 'Particionar discos',
  partitioning: 'Particionar discos',
  overview: 'Particionar discos',
  edit: 'Particionar discos',
  storage: 'Particionar discos',
  confirm: 'Particionar discos',
  format: 'Particionar discos',
  base: 'Instalar o sistema básico',
  software: 'Seleção de software',
  packages: 'Selecionar e instalar software',
  complete: 'Finalizar a instalação',
  finishing: 'Finalizar a instalação',
};
export type Stage = keyof typeof titles;
export const progressStages: Partial<
  Record<Stage, { label: string; details: string[]; next: Stage }>
> = {
  media: {
    label: 'Verificando a mídia de instalação',
    details: [
      'Verificando /cdrom/pool/main/…',
      'Verificando os arquivos da mídia…',
      'Mídia de instalação detectada.',
    ],
    next: 'components',
  },
  components: {
    label: 'Carregando componentes adicionais',
    details: [
      'Obtendo apt-setup-udeb…',
      'Carregando componentes de rede…',
      'Componentes carregados.',
    ],
    next: 'network',
  },
  link: {
    label: 'Detectando a conexão; aguarde…',
    details: [
      'Detectando a conexão de rede…',
      'Configurando a rede com DHCP…',
      'Configuração da rede concluída.',
    ],
    next: 'hostname',
  },
  partitioning: {
    label: 'Particionamento guiado',
    details: [
      'Calculando as novas partições…',
      'Preparando os pontos de montagem…',
      'Cálculo das partições concluído.',
    ],
    next: 'overview',
  },
  format: {
    label: 'Formatando as partições',
    details: [
      'Criando a partição de sistema EFI…',
      'Criando os sistemas de arquivos…',
      'Formatação das partições concluída.',
    ],
    next: 'base',
  },
  base: {
    label: 'Instalando o sistema básico',
    details: [
      'Validando libstdc++6…',
      'Extraindo os pacotes do sistema básico…',
      'Configurando o kernel Linux…',
      'Sistema básico instalado.',
    ],
    next: 'software',
  },
  packages: {
    label: 'Selecionar e instalar software',
    details: [
      'Obtendo os arquivos dos pacotes…',
      'Descompactando os pacotes selecionados…',
      'Configurando os programas…',
      'Instalação do software concluída.',
    ],
    next: 'complete',
  },
  finishing: {
    label: 'Finalizando a instalação',
    details: [
      'Executando netcfg-copy-config…',
      'Configurando o carregador de inicialização…',
      'Desmontando os sistemas de arquivos…',
      'Instalação finalizada.',
    ],
    next: 'complete',
  },
};
export const help: Partial<Record<Stage, [string, string][]>> = {
  method: [
    [
      'Maior espaço livre contínuo',
      'Usa somente a área livre do disco e mantém a partição existente.',
    ],
    ['Disco inteiro', 'Substitui a tabela de partições do disco escolhido após sua confirmação.'],
    ['LVM', 'Agrupa o armazenamento em volumes lógicos para separar os pontos de montagem.'],
    [
      'LVM criptografado',
      'Cria volumes lógicos dentro de um volume marcado como criptografado no sistema do jogo. A frase secreta é confirmada antes de continuar.',
    ],
    [
      'Manual',
      'Permite criar, editar e excluir partições, escolher seus tamanhos, sistemas de arquivos e pontos de montagem.',
    ],
  ],
  scheme: [
    ['Todos os arquivos', 'Coloca o sistema e os dados na partição raiz (/), além de EFI e swap.'],
    [
      '/home separada',
      'Separa os arquivos pessoais do sistema. Formatar /home ainda apaga seus dados.',
    ],
    [
      '/home, /var e /tmp',
      'Separa dados pessoais, arquivos variáveis e temporários para controlar o espaço de cada área.',
    ],
    ['/var e /srv', 'Separa logs e dados de serviços, reservando menos de 1 GB para swap.'],
    ['Disco pequeno', 'Cria uma raiz compacta de 7 GB, EFI e 1 GB de swap. O restante fica livre.'],
  ],
  overview: [
    ['Particionamento guiado', 'Retorna à escolha do método para gerar outro esquema.'],
    [
      'RAID por software',
      'Espelha o armazenamento dos dois discos virtuais (RAID 1). A capacidade útil fica limitada ao menor disco: 64,2 GB.',
    ],
    ['LVM', 'Organiza as partições Linux como volumes lógicos do grupo kali-vg.'],
    [
      'Volumes criptografados',
      'Identifica o contêiner de armazenamento como criptografado e solicita uma frase secreta.',
    ],
    [
      'iSCSI',
      'Usa o disco de laboratório apresentado como um dispositivo iSCSI dentro da rede virtual.',
    ],
    [
      'Partição ou espaço livre',
      'Selecione a linha e continue, ou dê um clique duplo, para editar ou criar uma partição. EFI inicia o sistema; ext4, ext3, ext2, Btrfs e XFS guardam arquivos; swap é a área de troca; FAT32 e FAT16 servem para volumes compatíveis com outros sistemas.',
    ],
    ['Desfazer alterações', 'Descarta a tabela proposta e retorna à escolha do método.'],
    ['Finalizar', 'Valida a tabela e abre a confirmação. Nada é aplicado antes de escolher Sim.'],
  ],
  edit: [
    ['Tamanho', 'Espaço da partição em GB. A soma não pode ultrapassar a capacidade disponível.'],
    ['Usar como', 'Escolhe o sistema de arquivos. ESP usa /boot/efi e swap usa a área de troca.'],
    [
      'Ponto de montagem',
      'Diretório onde a partição aparece. / é obrigatório; /home, /var, /tmp e /srv são opcionais.',
    ],
    ['Excluir', 'Remove a partição da proposta e devolve seu espaço à área livre.'],
  ],
};
