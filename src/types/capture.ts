// Types des données de capture échangées avec le backend : graphe (nœuds,
// arêtes, updates), stats et contrat des événements du Channel Tauri.
//
// Contrat GÉNÉRÉ depuis les types Rust (#142) : `cargo test
// export_ipc_bindings` dans `src-tauri` écrit `./generated/*.ts` depuis
// `crate::events::contract` (miroir vérifié par un test de fidélité JSON,
// variante par variante) et `crate::errors`. Ne pas éditer `./generated/*`
// à la main — modifier les types Rust puis relancer le test, la CI échoue
// si le contrat commité a dérivé.
import type { CaptureEvent } from "./generated/CaptureEvent";
export type { CaptureEvent };
export type ImportProgress = Extract<CaptureEvent, { event: "importProgress" }>["data"];

export type { Stats } from "./generated/Stats";
export type { Node } from "./generated/Node";
export type { Edge } from "./generated/Edge";
export type { GraphUpdate } from "./generated/GraphUpdate";
export type { GraphData } from "./generated/GraphData";
export type { DataLink } from "./generated/DataLink";
export type { NetworkProtocol } from "./generated/NetworkProtocol";
// Nom conservé côté frontend (le type Rust/généré s'appelle
// `PacketBatchPacket`) : forme d'un paquet, qu'il soit lu depuis
// `packetBatch.packets` ou redistribué un par un par `onPacket` côté store
// — aucun événement `packet` unitaire n'existe sur le fil (#142).
export type { PacketBatchPacket as CapturedPacket } from "./generated/PacketBatchPacket";

export type NodeId = string;
export type EdgeId = string;

// Forme enrichie d'un `Node`/`Edge` généré pour les besoins d'affichage du
// graphe (graphology/sigma.js) : tous les champs additionnels sont dérivés
// côté frontend (survol, contour, couleur), jamais reçus du backend.
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
