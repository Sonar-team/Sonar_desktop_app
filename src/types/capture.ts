import { Channel } from '@tauri-apps/api/core';

export type Stats = {
  received: number;
  dropped: number;
  ifDropped: number;
}

export type PacketMinimal = {
    ts_sec: number;
    ts_usec: number;
    caplen: number;
    len: number;
    flow: PacketFlow | null;
}

export type PacketFlow = {
    data_link?: DataLinkLayer;
    internet?: InternetLayer;
    transport?: TransportLayer;
    application?: ApplicationLayer;
}

export type DataLinkLayer = {
    protocol?: string;
    source?: string;
    destination?: string;
}

export type InternetLayer = {
    protocol?: string;
    source?: string;
    destination?: string;
}

export type TransportLayer = {
    protocol?: string;
    source?: number;
    destination?: number;
}

export type ApplicationLayer = {
    protocol?: string;
    
}

export type CaptureEvent =
  | {
      event: 'started';
      data: {
        device: string;
        bufferSize: number;
        timeout: number;
      };
    }
  | {
      event: 'stats';
      data: {
        stats: Stats;
        processed: number;
      };
    }
  | {
      event: 'error';
      data: {
        message: string;
      };
    }
  | {
      event: 'stopped';
      data: {
        reason: string;
      };
    }
  | {
      event: 'packet';
      data: {
        packet: PacketMinimal;
      };
    }
  | {
    event: 'GraphUpdate';
    data: {
      graph: GraphUpdate;
    };
  };

export interface CaptureChannel extends Channel<CaptureEvent> {
  onmessage: (event: { event: string; data: any }) => void;
}


export type GraphUpdate = {
  nodes: NodeData[];
  edges: EdgeData[];
}
type NodeId = string
type EdgeId = string

interface NodeData {
  id: string
  name: string
  mac?: string
  color: string // base fill color (hex)
  // precomputed for hover/stroke (avoid conversion on every hover)
  _hover?: string
  _stroke?: string
}

interface EdgeData {
  id: EdgeId
  source: NodeId
  target: NodeId
  label: string
  source_port?: string | number
  destination_port?: string | number
}

// --- Constants --------------------------------------------------------------

export const EDGE_COLORS: Record<string, string> = Object.freeze({
  Arp: "#FFFF00",
  Ipv4: "#FFA500",
  Ipv6: "#EE82EE",
  Profinet_rt: "#008000",
  TLS: "#0000FF",
  DNS: "#FF0000",
  NTP: "#FFA500",
})
