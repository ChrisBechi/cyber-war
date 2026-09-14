type Transport = (command: string, args: Record<string, unknown>) => Promise<unknown>;
let transport: Transport | undefined;
/** Used only by development scenario playback; production always uses Rust IPC. */
export function installDevelopmentTransport(next: Transport): () => void {
  if (!import.meta.env.DEV) {
    throw new Error('Scenario transport is development-only');
  }
  transport = next;
  return () => {
    if (transport === next) {
      transport = undefined;
    }
  };
}
export const developmentTransport = (): Transport | undefined =>
  import.meta.env.DEV ? transport : undefined;
