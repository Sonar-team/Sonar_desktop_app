import { Nodes, Edges, Layouts } from "v-network-graph"

const nodes:  Nodes = {
    node1: { name: "Source\nMAC: 00:00:00:00:00:00\nIP: 127.0.0.1\nPort: 17664" },
    node2: { name: "L2 Interface: lo" },
    node3: { name: "Destination\nMAC: 00:00:00:00:00:00\nIP: 127.0.0.1\nPort: 53" },
    node4: { name: "Destination\nMAC: 00:00:00:00:00:00\nIP: 127.0.0.1\nPort: 52" },
    node5: { name: "Additional Node\nMAC: AA:BB:CC:DD:EE:FF\nIP: 127.0.0.2\nPort: 80" },
  }

    const edges: Edges = {
    edge1: { source: "node1", target: "node2" },
    edge2: { source: "node2", target: "node3" },
    edge3: { source: "node2", target: "node4"},
    edge4: { source: "node2", target: "node5" },
    }

    const layouts: Layouts  = {
    nodes: {
        node1: { x: 0, y: 0 },
        node2: { x: 50, y: 50 },
        node3: { x: 100, y: 0 },
        node4: { x: 150, y: 50 },
    },
}

export default {
    nodes,
    edges,
    layouts,
  }