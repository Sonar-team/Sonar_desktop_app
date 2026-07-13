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

export type CapturedPacket = {
  ts_sec: number;
  ts_usec: number;
  caplen: number;
  len: number;
  flow: PacketFlow | null;
};

// Couche liaison packet_parser 7 : typée par LINKTYPE. Les MAC/EtherType ne
// sont présents que pour les liens qui en ont réellement (Ethernet, 802.11) ;
// SLL expose une adresse source unique, RAW IP aucune adresse.
export type DataLink = {
  link_type?: number;
  network_protocol?: { kind?: string; value?: number } | null;
  link_kind?: "ethernet" | "raw_ip" | "linux_sll" | "linux_sll2" | "ieee80211" | string;
  link_details?: {
    source_mac?: string;
    destination_mac?: string;
    source_address?: number[] | null;
    ethertype?: string;
    snap_protocol?: string;
    vlan?: { id?: number | null } | null;
  } | null;
};

export type PacketFlow = {
  data_link?: DataLink | null;
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
  /** Ports « service » observés sur l'arête (triés, plafonnés backend). */
  ports?: number[];
  /** Trafic sur ports exclusivement dynamiques/éphémères (rendu « … »). */
  has_dynamic_ports?: boolean;
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
  /** Ports « service » observés (triés, plafonnés backend). */
  ports?: number[];
  /** Trafic sur ports exclusivement dynamiques/éphémères (rendu « … »). */
  has_dynamic_ports?: boolean;
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

// `session_id` : session de capture live émettrice (0 = hors session, ex.
// import) ; le store ignore les événements d'une session périmée.
export type CaptureEvent =
  | {
    event: "started";
    data: {
      session_id: number;
      device: string;
      bufferSize: number;
      timeout: number;
      /** Type de liaison (LINKTYPE/DLT), ex. « Ethernet », « Linux cooked v1 ». */
      link_type?: string;
    };
  }
  | {
    event: "stats";
    data: {
      session_id: number;
      stats: Stats;
      processed: number;
    };
  }
  | {
    event: "channelCapacityPayload";
    data: {
      session_id: number;
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
      session_id: number;
      reason: string;
    };
  }
  | {
    event: "packet";
    data: {
      packet: CapturedPacket;
    };
  }
  | {
    event: "packetBatch";
    data: {
      session_id: number;
      packets: CapturedPacket[];
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
      session_id: number;
      updates: GraphUpdate[];
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
