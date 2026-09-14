import { z } from 'zod';

export const wirelessSchema = z.object({
  ssid: z.string(),
  bssid: z.string(),
  channel: z.number(),
  signal: z.number(),
  encryption: z.string(),
  clients: z.number(),
  access: z.boolean(),
});
export const signalEvidenceSchema = z.object({
  organization: z.string(),
  capturePath: z.string().nullable(),
  detail: z.string(),
});
export const packetSchema = z.object({
  number: z.number(),
  time: z.string(),
  source: z.string(),
  destination: z.string(),
  protocol: z.string(),
  stream: z.number(),
  text: z.string(),
});
export type WirelessNetwork = z.infer<typeof wirelessSchema>;
export type SignalEvidence = z.infer<typeof signalEvidenceSchema>;
export type Packet = z.infer<typeof packetSchema>;
