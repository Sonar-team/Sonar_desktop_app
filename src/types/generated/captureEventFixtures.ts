// Généré par `cargo test export_ipc_fixtures` (#142) — NE PAS ÉDITER À LA MAIN.
// JSON réellement produit par `CaptureEvent` (src-tauri/src/events/contract.rs,
// test export_ipc_fixtures). Relancer ce test après toute modification du
// contrat ; la CI échoue si le résultat commité a dérivé.
import type { CaptureEvent } from "../capture";

export const startedFixture = ({
  "event": "started",
  "data": {
    "sessionId": 7,
    "device": "eth0",
    "bufferSize": 65536,
    "chanCapacity": 1024,
    "timeout": 100,
    "snaplen": 262144,
    "linkType": "Ethernet",
    "protocolVersion": 1
  }
}) satisfies CaptureEvent;

export const statsFixture = ({
  "event": "stats",
  "data": {
    "sessionId": 1,
    "received": 10,
    "dropped": 0,
    "ifDropped": 0,
    "appDropped": 0,
    "parseErrors": 1,
    "processed": 3
  }
}) satisfies CaptureEvent;

export const channelCapacityPayloadFixture = ({
  "event": "channelCapacityPayload",
  "data": {
    "sessionId": 2,
    "channelSize": 1024,
    "currentSize": 512,
    "backpressure": true
  }
}) satisfies CaptureEvent;

export const stoppedFixture = ({
  "event": "stopped",
  "data": {
    "sessionId": 3,
    "reason": "erreur pcap"
  }
}) satisfies CaptureEvent;

export const finishedFixture = ({
  "event": "finished",
  "data": {
    "fileName": "capture.pcap",
    "packetTotalCount": 100,
    "integratedCount": 95,
    "parseErrorCount": 5,
    "matrixTotalCount": 40
  }
}) satisfies CaptureEvent;

export const importProgressFixture = ({
  "event": "importProgress",
  "data": {
    "fileName": "capture.pcap",
    "fileIndex": 2,
    "filesTotal": 5,
    "current": 1000,
    "total": 48350
  }
}) satisfies CaptureEvent;

export const graphNodeAddedFixture = ({
  "event": "graph",
  "data": {
    "update": {
      "type": "nodeAdded",
      "payload": {
        "id": "node-1",
        "name": "192.168.1.10",
        "color": "#ff0000",
        "mac": "aa:bb:cc:dd:ee:ff",
        "macs": [
          "aa:bb:cc:dd:ee:ff"
        ],
        "ip": "192.168.1.10",
        "label": "Serveur"
      }
    }
  }
}) satisfies CaptureEvent;

export const graphNodeUpdatedFixture = ({
  "event": "graph",
  "data": {
    "update": {
      "type": "nodeUpdated",
      "payload": {
        "id": "node-1",
        "name": "192.168.1.10",
        "color": "#ff0000",
        "mac": "aa:bb:cc:dd:ee:ff",
        "macs": [
          "aa:bb:cc:dd:ee:ff"
        ],
        "ip": "192.168.1.10",
        "label": "Serveur"
      }
    }
  }
}) satisfies CaptureEvent;

export const graphBatchFixture = ({
  "event": "graphBatch",
  "data": {
    "sessionId": 5,
    "updates": [
      {
        "type": "edgeAdded",
        "payload": {
          "id": "edge-1",
          "source": "node-1",
          "target": "node-2",
          "label": "TCP",
          "sourcePort": 443,
          "destinationPort": 51820,
          "ports": [
            80,
            443
          ],
          "hasDynamicPorts": false,
          "bidir": true,
          "count": 12,
          "totalBytes": 4096,
          "encapIds": [
            "deadbeef"
          ]
        }
      },
      {
        "type": "edgeUpdated",
        "payload": {
          "id": "edge-1",
          "source": "node-1",
          "target": "node-2",
          "label": "TCP",
          "sourcePort": 443,
          "destinationPort": 51820,
          "ports": [
            80,
            443
          ],
          "hasDynamicPorts": false,
          "bidir": true,
          "count": 12,
          "totalBytes": 4096,
          "encapIds": [
            "deadbeef"
          ]
        }
      }
    ]
  }
}) satisfies CaptureEvent;

export const graphBatchWithoutLabelOrPortsFixture = ({
  "event": "graphBatch",
  "data": {
    "sessionId": 5,
    "updates": [
      {
        "type": "nodeAdded",
        "payload": {
          "id": "node-2",
          "name": "192.168.1.20",
          "color": "#00ff00",
          "mac": "aa:bb:cc:dd:ee:00",
          "macs": [
            "aa:bb:cc:dd:ee:00"
          ],
          "ip": "192.168.1.20",
          "label": null
        }
      },
      {
        "type": "edgeAdded",
        "payload": {
          "id": "edge-2",
          "source": "node-1",
          "target": "node-2",
          "label": "IPv6",
          "sourcePort": null,
          "destinationPort": null,
          "ports": [],
          "hasDynamicPorts": false,
          "bidir": false,
          "count": 3,
          "totalBytes": 512,
          "encapIds": []
        }
      }
    ]
  }
}) satisfies CaptureEvent;

export const graphSnapshotFixture = ({
  "event": "graphSnapshot",
  "data": {
    "graphData": {
      "nodes": {
        "node-1": {
          "id": "node-1",
          "name": "192.168.1.10",
          "color": "#ff0000",
          "mac": "aa:bb:cc:dd:ee:ff",
          "macs": [
            "aa:bb:cc:dd:ee:ff"
          ],
          "ip": "192.168.1.10",
          "label": "Serveur"
        }
      },
      "edges": {
        "edge-1": {
          "id": "edge-1",
          "source": "node-1",
          "target": "node-2",
          "label": "TCP",
          "sourcePort": 443,
          "destinationPort": 51820,
          "ports": [
            80,
            443
          ],
          "hasDynamicPorts": false,
          "bidir": true,
          "count": 12,
          "totalBytes": 4096,
          "encapIds": [
            "deadbeef"
          ]
        }
      }
    }
  }
}) satisfies CaptureEvent;

export const packetBatchEthernetWithVlanFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4",
              "vlan": {
                "id": 42,
                "pcp": 0,
                "dei": false
              }
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP",
          "source_port": 51820,
          "destination_port": 443,
          "protocol_transport": "TCP",
          "application_protocol": "HTTPS"
        },
        "encapId": "00000000deadbeef"
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchMinimalFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          }
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchTunneledFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP",
          "inner": {
            "data_link": {
              "link_type": 1,
              "network_protocol": {
                "kind": "ipv4"
              },
              "link_kind": "ethernet",
              "link_details": {
                "destination_mac": "aa:bb:cc:dd:ee:ff",
                "source_mac": "11:22:33:44:55:66",
                "ethertype": "IPv4"
              }
            },
            "source_ip": "10.0.0.1",
            "ip_source_type": "Private",
            "destination_ip": "10.0.0.2",
            "ip_destination_type": "Public",
            "protocol_internet": "TCP"
          }
        },
        "encapId": "1122334455667788"
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchCorruptedFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "corrupted": {
            "layer": "Internet",
            "error": "en-tête IPv4 tronqué"
          }
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "corrupted": {
            "layer": "Transport",
            "error": "en-tête TCP tronqué"
          }
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchSllFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 113,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "linux_sll",
            "link_details": {
              "packet_type": 4,
              "hardware_type": 1,
              "address_length": 6,
              "source_address": [
                224,
                213,
                94,
                40,
                155,
                212
              ],
              "protocol": 2048
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP",
          "source_port": 51820,
          "destination_port": 443,
          "protocol_transport": "TCP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchInternetAndTransportPresentWithoutAddressesFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": null,
          "ip_source_type": null,
          "destination_ip": null,
          "ip_destination_type": null,
          "protocol_internet": "ARP",
          "source_port": null,
          "destination_port": null,
          "protocol_transport": "ICMP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchSllWithoutAddressFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 113,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "linux_sll",
            "link_details": {
              "packet_type": 4,
              "hardware_type": 1,
              "address_length": 0,
              "source_address": null,
              "protocol": 2048
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP",
          "source_port": 51820,
          "destination_port": 443,
          "protocol_transport": "TCP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchSll2Fixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 276,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "linux_sll2",
            "link_details": {
              "protocol": 2048,
              "reserved_mbz": 0,
              "interface_index": 3,
              "hardware_type": 1,
              "packet_type": 0,
              "address_length": 6,
              "source_address": [
                224,
                213,
                94,
                40,
                155,
                212
              ]
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchSll2WithoutAddressFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 276,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "linux_sll2",
            "link_details": {
              "protocol": 2048,
              "reserved_mbz": 0,
              "interface_index": 3,
              "hardware_type": 1,
              "packet_type": 0,
              "address_length": 0,
              "source_address": null
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP",
          "source_port": 51820,
          "destination_port": 443,
          "protocol_transport": "TCP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchIeee80211Fixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 105,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ieee80211",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "snap_protocol": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchRawIpFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 101,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "raw_ip",
            "link_details": {
              "ip_version": 4
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP",
          "source_port": 51820,
          "destination_port": 443,
          "protocol_transport": "TCP",
          "application_protocol": "HTTPS"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchAllIpTypesFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Private",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Private",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Multicast",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Multicast",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Loopback",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Loopback",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Apipa",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Apipa",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "LinkLocal",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "LinkLocal",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Ula",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Ula",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Public",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Public",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Documentation",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Documentation",
          "protocol_internet": "TCP"
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv4"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv4"
            }
          },
          "source_ip": "192.168.1.10",
          "ip_source_type": "Unknown",
          "destination_ip": "8.8.8.8",
          "ip_destination_type": "Unknown",
          "protocol_internet": "TCP"
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;

export const packetBatchAllNetworkProtocolsFixture = ({
  "event": "packetBatch",
  "data": {
    "sessionId": 1,
    "packets": [
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "ipv6"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "IPv6"
            }
          }
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "arp"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "ARP"
            }
          }
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "profinet"
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "Profinet"
            }
          }
        },
        "encapId": null
      },
      {
        "tsSec": 0,
        "tsUsec": 0,
        "caplen": 60,
        "len": 60,
        "flow": {
          "data_link": {
            "link_type": 1,
            "network_protocol": {
              "kind": "other",
              "value": 34997
            },
            "link_kind": "ethernet",
            "link_details": {
              "destination_mac": "aa:bb:cc:dd:ee:ff",
              "source_mac": "11:22:33:44:55:66",
              "ethertype": "Unknown (0x88B5)"
            }
          }
        },
        "encapId": null
      }
    ]
  }
}) satisfies CaptureEvent;
