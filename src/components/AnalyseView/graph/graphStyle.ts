// Helpers purs de style et de mise en forme du graphe réseau : couleurs,
// tailles proportionnelles au trafic, labels et courbure des arêtes.
import { DEFAULT_EDGE_CURVATURE } from "@sigma/edge-curve"
import type { NodeLabelDrawingFunction } from "sigma/rendering"
import { Edge, EdgeData, EdgeId, Node } from "../../../types/capture"
import { colorForProtocol } from "../../../utils/protocolColors"

// --- Couleurs ---------------------------------------------------------------
function clamp01(x: number) { return Math.min(1, Math.max(0, x)) }

function hexToRgb(hex: string) {
  const h = hex.startsWith("#") ? hex.slice(1) : hex
  const v = parseInt(h.length === 3 ? h.replace(/(.)/g, "$1$1") : h, 16)
  return { r: (v >> 16) & 255, g: (v >> 8) & 255, b: v & 255 }
}

function rgbToHex(r: number, g: number, b: number) {
  return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)
}

export function darken(hex: string, factor = 0.2) {
  const { r, g, b } = hexToRgb(hex)
  return rgbToHex(Math.trunc(r * (1 - factor)), Math.trunc(g * (1 - factor)), Math.trunc(b * (1 - factor)))
}

export function brighten(hex: string, factor = 0.15) {
  const { r, g, b } = hexToRgb(hex)
  return rgbToHex(
    Math.trunc(clamp01(r / 255 + factor) * 255),
    Math.trunc(clamp01(g / 255 + factor) * 255),
    Math.trunc(clamp01(b / 255 + factor) * 255)
  )
}

// Couleurs d'estompage pendant la surbrillance d'un tunnel (fond noir)
export const DIM_EDGE_COLOR = "#2a2a2a"
export const DIM_NODE_COLOR = "#3a3a3a"

// Bordure d'alerte : IP observée avec plusieurs MAC (anomalie à investiguer)
export const MAC_CONFLICT_BORDER_COLOR = "#FF5252"

// Bordure d'alerte : même IP portée par plusieurs nœuds (VLAN différents,
// #154). Moins sévère que le conflit de MAC (qui garde priorité) : ambre.
export const DUPLICATE_IP_BORDER_COLOR = "#FFB300"

// --- Clés d'arêtes ----------------------------------------------------------
const EDGE_SEP = "__"
export function edgeKey(e: EdgeData): EdgeId {
  return `${e.source}${EDGE_SEP}${e.target}${EDGE_SEP}${e.label}`
}

// --- Positionnement ---------------------------------------------------------
// CSPRNG plutôt que Math.random() : purement cosmétique ici (jitter de
// positionnement des nœuds), mais évite le signalement générique de
// SonarQube sur Math.random() (S2245, "PRNG non sécurisé").
export function randomFloat(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0] / 2 ** 32
}

// Jitter autour d'un point d'ancrage pour les nouveaux nœuds (évite l'empilement)
export function jitterAround(x: number, y: number, radius = 120) {
  const angle = randomFloat() * 2 * Math.PI
  const r = radius * (0.4 + 0.6 * randomFloat())
  return { x: x + Math.cos(angle) * r, y: y + Math.sin(angle) * r }
}

// --- Seuils de zoom pour l'affichage des labels d'arêtes (zoom = 1 / camera.ratio)
export const EDGE_LABEL_ZOOM = 1.2
export const PORT_LABEL_ZOOM = 1.8

// --- Tailles ----------------------------------------------------------------
// Tailles proportionnelles au trafic, en échelle log : les gros parleurs
// ressortent sans écraser les hôtes discrets.
export const NODE_SIZE_MIN = 7
export const NODE_SIZE_MAX = 80
export function nodeSizeFor(bytes: number) {
  return Math.min(NODE_SIZE_MAX, NODE_SIZE_MIN + 2 * Math.log2(1 + bytes / 2000))
}

export const EDGE_SIZE_MIN = 1.2
export const EDGE_SIZE_MAX = 7
export function edgeSizeFor(bytes: number) {
  return Math.min(EDGE_SIZE_MAX, EDGE_SIZE_MIN + 0.55 * Math.log2(1 + bytes / 1500))
}

export function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} o`
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} ko`
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} Mo`
  return `${(bytes / 1024 ** 3).toFixed(2)} Go`
}

// --- Courbure des arêtes parallèles (repris de l'exemple officiel @sigma/edge-curve)
export function getCurvature(index: number, maxIndex: number): number {
  if (maxIndex <= 0) return DEFAULT_EDGE_CURVATURE
  const amplitude = 3.5
  const maxCurvature = amplitude * (1 - Math.exp(-maxIndex / amplitude)) * DEFAULT_EDGE_CURVATURE
  return (maxCurvature * index) / maxIndex
}

// --- Rendu des labels de nœuds ----------------------------------------------
// Label de nœud : texte blanc sur fond noir, au-dessus du nœud.
// Sert aussi au rendu du hover (fond blanc par défaut, illisible sur fond noir).
export const drawNodeLabel: NodeLabelDrawingFunction = (context, data, settings) => {
  if (!data.label) return
  const size = settings.labelSize
  context.font = `${settings.labelWeight} ${size}px ${settings.labelFont}`
  const width = context.measureText(data.label).width + 10
  const x = data.x - width / 2
  const y = data.y - data.size - size - 8
  context.fillStyle = "#000000CC"
  context.fillRect(x, y, width, size + 6)
  context.fillStyle = "#ffffff"
  context.fillText(data.label, x + 5, y + size)
}

// --- Attributs graphology ---------------------------------------------------
export function nodeAttributes(node: Node) {
  const color = node.color || "#2196F3"
  const rawLabel = node.label || ""
  const macs: string[] = Array.isArray(node.macs) ? node.macs : []
  // Plusieurs MAC unicast pour une même IP : anomalie (IP partagée, VRRP,
  // usurpation ARP…) signalée par une bordure d'alerte.
  const macConflict = macs.length > 1
  // Identité contextualisée (#154) : le VLAN fait partie de la clé du nœud,
  // il est affiché dans le libellé ; une IP présente sur plusieurs VLAN est
  // signalée (bordure ambre), le conflit de MAC (rouge) garde priorité.
  const vlanId: number | null = typeof node.vlan_id === "number" ? node.vlan_id : null
  const duplicateIp = node.duplicate_ip === true
  const vlanSuffix = vlanId !== null ? ` (VLAN ${vlanId})` : ""
  const borderColor = macConflict
    ? MAC_CONFLICT_BORDER_COLOR
    : duplicateIp
    ? DUPLICATE_IP_BORDER_COLOR
    : darken(color, 0.25)
  return {
    name: node.name || node.id,
    mac: node.mac || "",
    macs,
    macConflict,
    ip: node.ip || "",
    vlanId,
    duplicateIp,
    rawLabel,
    label: (rawLabel || node.name || node.id) + vlanSuffix,
    color,
    borderColor,
    hoverColor: brighten(color, 0.18),
  }
}

export function edgeAttributes(e: Edge) {
  const totalBytes = Number(e.totalBytes) || 0
  return {
    protocol: e.label || "",
    source_port: e.sourcePort ?? null,
    destination_port: e.destinationPort ?? null,
    // Tous les ports « service » observés sur l'arête (triés, plafonnés
    // backend) : plusieurs services entre deux équipements ne sont plus
    // masqués par le premier couple de ports. Les ports éphémères ne sont
    // pas listés : has_dynamic_ports les signale (rendu « … »).
    ports: Array.isArray(e.ports) ? e.ports : [],
    has_dynamic_ports: e.hasDynamicPorts === true,
    bidir: !!e.bidir,
    count: Number(e.count) || 0,
    total_bytes: totalBytes,
    // Tunnels (encap_id hex) auxquels ce flux participe : le backend envoie
    // la liste cumulative, on remplace donc simplement à chaque update.
    encapIds: Array.isArray(e.encapIds) ? e.encapIds : [],
    color: colorForProtocol(e.label || ""),
    size: edgeSizeFor(totalBytes),
  }
}

// --- Libellé d'arête rendu (reducer Sigma) -----------------------------------
interface EdgeLabelSource {
  protocol?: string
  ports?: number[]
  has_dynamic_ports?: boolean
  source_port?: number | string | null
  destination_port?: number | string | null
}

/** Libellé d'arête affiché par le edgeReducer de NetworkGraphComponent.vue :
 *  protocole, plus le détail des ports si affiché à ce niveau de zoom.
 *  Extrait du reducer (complexité cognitive 18, hors du seuil Sonar de 15)
 *  pour rester une fonction pure testable indépendamment. */
export function edgeLabelFor(data: EdgeLabelSource, portLabelsShown: boolean): string {
  const protocol = data.protocol ?? ""
  if (!portLabelsShown) return protocol

  const ports: number[] = Array.isArray(data.ports) ? data.ports : []
  const hasDynamic = data.has_dynamic_ports === true
  if (ports.length > 0) {
    // Ports « service » de l'arête (les éphémères ne sont pas listés,
    // le backend les résume par has_dynamic_ports → « … »).
    return `${protocol} :${ports.join(",")}${hasDynamic ? ",…" : ""}`
  }
  if (hasDynamic) {
    // Uniquement du trafic sur ports dynamiques : signalé sans liste.
    return `${protocol} :…`
  }
  if (data.source_port != null || data.destination_port != null) {
    return `${protocol} ${data.source_port ?? ""}→${data.destination_port ?? ""}`
  }
  return protocol
}
