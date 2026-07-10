// Configuration de capture (miroir de `CaptureConfig` côté Rust) et valeurs
// par défaut proposées dans le panneau de configuration.

export type CaptureConfig = {
  device_name: string;
  buffer_size: number;
  chan_capacity: number;
  timeout: number;
  snaplen: number;
};

export const DEFAULT_CAPTURE_CONFIG = {
  buffer_size: 18_000_000,
  chan_capacity: 10_000,
  timeout: 25,
  snaplen: 65_536,
} satisfies Omit<CaptureConfig, "device_name">;
