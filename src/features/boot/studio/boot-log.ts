// Local cinematic transcript. Lengths intentionally range from brief status lines
// to detailed kernel messages which wrap at the current terminal width.
const startup = [
  'Linux version 6.12.0-kali-amd64 · root@lifeos',
  'Command line: BOOT_IMAGE=/boot/vmlinuz-6.12.0-kali-amd64 root=UUID=9ad21470-463b-4ae5-8c12-38419fc07a21 ro loglevel=3 systemd.show_status=true console=tty0',
  'BIOS-provided physical RAM map:',
  'e820: [mem 0x0000000000100000-0x00000001ffffffff] usable · reserved regions have been excluded from the kernel allocator and device memory mappings',
  'NX protection: active',
  'DMI: LifeOS Virtual Machine / Standard PC, BIOS 1.16.3',
  'x86/mm: Checked the initial memory map; kernel text, read-only data and the temporary boot page tables are protected against unintended writes',
  'ACPI: Early table checksum verification completed',
  'APIC: Initialized',
  'CPU: 4 logical processors online · detected invariant TSC · scheduler domains and per-CPU interrupt vectors configured',
  'smp: Bringing up secondary CPUs ... done',
  'Memory: 8096256K available',
  'Kernel command line parsed; console output routed to tty0 while the initial RAM filesystem prepares the root device and loads storage drivers',
  'RCU: Hierarchical implementation',
  'clocksource: Switched to tsc',
  'PCI: Probing bus resources and assigning I/O windows; MSI interrupts enabled for storage, Ethernet, display and input devices',
  'iommu: Default domain type: translated',
  'SCSI subsystem initialized',
  'usbcore: Registered new interface driver usbfs',
  'VFS: Mounted the initial RAM filesystem; resolving the root UUID and waiting for required block devices to become available',
  'virtio_blk: [vda] 125829120 512-byte logical blocks',
  ' vda: vda1 vda2',
  'EXT4-fs (vda1): Recovery complete. Mounted filesystem with ordered data mode; journal checksums verified and delayed allocation enabled',
  'VFS: Mounted root (ext4 filesystem)',
  'Freeing unused kernel image memory: 2816K',
  'Run /sbin/init as init process',
  'systemd[1]: Detected architecture x86-64; initializing the system service manager and applying the default dependency order for this machine',
  'systemd[1]: Hostname set to <lifeos>',
  '[  OK  ] Created slice System Slice',
  '[  OK  ] Reached target Local File Systems',
  'systemd-journald: Runtime journal opened; persistent storage available at /var/log/journal, rotating previous boot records before accepting new messages',
  '[  OK  ] Started Journal Service',
  '[  OK  ] Mounted /tmp',
  'systemd-udevd: Loading device rules, applying permissions and creating persistent symbolic links for discovered block, network and input devices',
  '[  OK  ] Started Device Event Manager',
  '[  OK  ] Reached target Basic System',
  'NetworkManager: Starting network discovery; restoring the saved connection profile, requesting an address and checking the local default gateway',
  'eth0: Link is up',
  'eth0: inet 10.20.4.2/24 · gateway 10.20.4.1 · local resolver ready · lease accepted and routing table updated successfully',
  '[  OK  ] Reached target Network',
  'dbus-broker: System message bus is ready',
  '[  OK  ] Started User Login Management',
  'accounts-daemon: Loaded local account records; preparing the root session, language preferences, home directory access and user service environment',
  'systemd-logind: New seat seat0',
  'LightDM: Display :0 initialized',
  'Xorg: Screen 0 configured with the native display mode; keyboard layout, pointer acceleration and monitor geometry have been applied',
  'xfsettingsd: Applying Kali desktop theme',
  'xfwm4: Compositor ready',
  'xfce4-panel: Restoring launchers, workspaces, notification area, clock and session controls from the local desktop configuration',
  '[  OK  ] Desktop services online',
];
const modules = [
  [
    'filesystem',
    'Verified the mounted volumes, user directories and temporary workspaces; file metadata and access permissions are available to the desktop applications',
  ],
  [
    'network',
    'Restored routing and resolver state; the local gateway is reachable and applications may request connections through the configured network profile',
  ],
  ['input', 'Keyboard layout loaded'],
  ['display', 'Native resolution applied'],
  [
    'audio',
    'Output device initialized; separate music, effects and interface channels are registered with the session mixer and ready for playback',
  ],
  ['notifications', 'Message service ready'],
  [
    'terminal',
    'Loaded command history, shell preferences, font metrics and cursor settings; interactive terminal sessions can now be opened',
  ],
  ['desktop', 'Workspaces restored'],
];
export function bootLogRows(count: number, columns: number): string[] {
  const width = Math.max(40, columns);
  const rows: string[] = [];
  let entry = 0;
  while (rows.length < count - 1) {
    const module = modules[(entry - startup.length + modules.length * 100) % modules.length];
    const message = startup[entry] ?? `session[${100 + entry}]: ${module[0]} · ${module[1]}`;
    let text = `[ ${(entry * 0.038741).toFixed(6)} ] ${message}`;
    while (text.length && rows.length < count - 1) {
      let end = Math.min(width, text.length);
      if (end < text.length) {
        const space = text.lastIndexOf(' ', end);
        if (space > width / 2) {
          end = space;
        }
      }
      rows.push(text.slice(0, end));
      text = text.slice(end).trimStart();
    }
    entry++;
  }
  rows.push('[  OK  ] root@lifeos: session prepared · display :0');
  return rows;
}
