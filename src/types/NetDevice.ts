// types.d.ts ou en haut du composant
export type Address = {
  addr: string; // IpAddr est sérialisé en string
  netmask?: string | null;
  broadcast_addr?: string | null;
  dst_addr?: string | null;
};

export type IfFlags = { 
    bits: number 
};

export type ConnectionStatus =
  | "Unknown"
  | "Connected"
  | "Disconnected"
  | "NotApplicable";

export type DeviceFlags = {
  if_flags: IfFlags;
  connection_status: ConnectionStatus;
};

export type NetDevice = {
  name: string;
  desc?: string | null;
  addresses: Address[];
  flags: DeviceFlags;
};
