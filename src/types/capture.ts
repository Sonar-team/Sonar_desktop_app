// Types des données de capture échangées avec le backend : graphe (nœuds,
// arêtes, updates), stats et contrat des événements du Channel Tauri
// (miroir de `CaptureEvent` côté Rust).
import { Channel } from "@tauri-apps/api/core";

export type GraphData = {
  nodes: Record<NodeId, NodeData>;
  edges: Record<EdgeId, EdgeData>;
};

export type Stats = {
  received: number;
  dropped: number;
  ifDropped: number;
  appDropped?: number;
};

export type PacketMinimal = {
  ts_sec: number;
  ts_usec: number;
  caplen: number;
  len: number;
  flow: PacketFlow | null;
};

export type PacketFlow = {
  source_mac?: string;
  destination_mac?: string;
  ethertype?: string;
  vlan?: { id?: number | null } | null;
  source_ip?: string | null;
  ip_source_type?: string | null;
  destination_ip?: string | null;
  ip_destination_type?: string | null;
  protocol_internet?: string | null;
  source_port?: number | null;
  destination_port?: number | null;
  protocol_transport?: string | null;
  application_protocol?: string | null;
  protocol?: string | null;
};

export type DataLinkLayer = {
  protocol?: string;
  source?: string;
  destination?: string;
};

export type InternetLayer = {
  protocol?: string;
  source?: string;
  destination?: string;
};

export type TransportLayer = {
  protocol?: string;
  source?: number;
  destination?: number;
};

export type ApplicationLayer = {
  protocol?: string;
};

export type Node = {
  id: string;
  name: string;
  color: string;
  mac: string;
  ip: string;
  label?: string;
};

export type Edge = {
  id: string;
  source: string;
  target: string;
  label: string;
  source_port: number | null;
  destination_port: number | null;
};

export type NodeId = string;
export type EdgeId = string;

export interface NodeData {
  id: string;
  name: string;
  /** Première MAC observée (clé de labellisation). */
  mac?: string;
  /** Toutes les MAC unicast observées ; plusieurs entrées = anomalie. */
  macs?: string[];
  ip?: string;
  color: string;
  label?: string;
  _hover?: string;
  _stroke?: string;
}

export interface EdgeData {
  source: NodeId;
  target: NodeId;
  label: string;
  source_port?: string | number | null;
  destination_port?: string | number | null;
  bidir?: boolean;
  count?: number;
  total_bytes?: number;
  /** Tunnels (encap_id hex) auxquels ce flux participe, cf. TUNNELS.md */
  encap_ids?: string[];
  _color?: string;
}

// enum GraphUpdate : soit edge soit node
export type GraphUpdate =
  | { type: "NodeAdded"; payload: Node }
  | { type: "NodeUpdated"; payload: Node }
  | { type: "EdgeAdded"; payload: Edge }
  | { type: "EdgeUpdated"; payload: Edge };

export type CaptureEvent =
  | {
    event: "started";
    data: {
      device: string;
      bufferSize: number;
      timeout: number;
    };
  }
  | {
    event: "stats";
    data: {
      stats: Stats;
      processed: number;
    };
  }
  | {
    event: "channelCapacityPayload";
    data: {
      channelSize: number;
      currentSize: number;
      backpressure: boolean;
    };
  }
  | {
    event: "error";
    data: {
      message: string;
    };
  }
  | {
    event: "stopped";
    data: {
      reason: string;
    };
  }
  | {
    event: "packet";
    data: {
      packet: PacketMinimal;
    };
  }
  | {
    event: "packetBatch";
    data: {
      packets: PacketMinimal[];
    };
  }
  | {
    event: "flowMatrixLen";
    data: {
      flowMatrixLen: number;
    };
  }
  | {
    event: "graph";
    data: {
      update: GraphUpdate;
    };
  }
  | {
    event: "graphBatch";
    data: {
      updates: GraphUpdate[];
    };
  }
  | {
    event: "stopped";
    data: {
      reason: string;
    };
  }
  | {
    event: "finished";
    data: {
      fileName: string;
      packetTotalCount: number;
      matrixTotalCount: number;
    };
  }
  | {
    event: "graphSnapshot";
    data: {
      graphData: GraphData;
    };
  };

export interface CaptureChannel extends Channel<CaptureEvent> {
  onmessage: (event: { event: string; data: any }) => void;
}
