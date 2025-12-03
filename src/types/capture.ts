// types/capture.ts
import { Channel } from "@tauri-apps/api/core";
import { EdgeData, EdgeId, NodeData, NodeId } from "../components/AnalyseView/NetworkGraphComponent.vue";

export type GraphData = {
  nodes: Record<NodeId, NodeData>;
  edges: Record<EdgeId, EdgeData>;
};

export type Stats = {
  received: number;
  dropped: number;
  ifDropped: number;
};

export type PacketMinimal = {
  ts_sec: number;
  ts_usec: number;
  caplen: number;
  len: number;
  flow: PacketFlow | null;
};

export type PacketFlow = {
  data_link?: DataLinkLayer;
  internet?: InternetLayer;
  transport?: TransportLayer;
  application?: ApplicationLayer;
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

// enum GraphUpdate : soit edge soit node
export type GraphUpdate =
  | { type: "NewNode"; payload: Node }
  | { type: "NewEdge"; payload: Edge }
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
  }
  ;

export interface CaptureChannel extends Channel<CaptureEvent> {
  onmessage: (event: { event: string; data: any }) => void;
}
