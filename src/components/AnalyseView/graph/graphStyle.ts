// Helpers purs de style et de mise en forme du graphe réseau : couleurs,
// tailles proportionnelles au trafic, labels et courbure des arêtes.
import { DEFAULT_EDGE_CURVATURE } from "@sigma/edge-curve"
import { EdgeData, EdgeId } from "../../../types/capture"
import { colorForProtocol } from "../../../utils/protocolColors"

// --- Couleurs ---------------------------------------------------------------
function clamp01(x: number) { return x < 0 ? 0 : x > 1 ? 1 : x }

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
  return rgbToHex((r * (1 - factor)) | 0, (g * (1 - factor)) | 0, (b * (1 - factor)) | 0)
}

export function brighten(hex: string, factor = 0.15) {
  const { r, g, b } = hexToRgb(hex)
  return rgbToHex(
    (clamp01(r / 255 + factor) * 255) | 0,
    (clamp01(g / 255 + factor) * 255) | 0,
    (clamp01(b / 255 + factor) * 255) | 0
  )
}

// Couleurs d'estompage pendant la surbrillance d'un tunnel (fond noir)
export const DIM_EDGE_COLOR = "#2a2a2a"
export const DIM_NODE_COLOR = "#3a3a3a"

// Bordure d'alerte : IP observée avec plusieurs MAC (anomalie à investiguer)
export const MAC_CONFLICT_BORDER_COLOR = "#FF5252"

// --- Clés d'arêtes ----------------------------------------------------------
const EDGE_SEP = "__"
export function edgeKey(e: EdgeData): EdgeId {
  return `${e.source}${EDGE_SEP}${e.target}${EDGE_SEP}${e.label}`
}

// --- Positionnement ---------------------------------------------------------
// Jitter autour d'un point d'ancrage pour les nouveaux nœuds (évite l'empilement)
export function jitterAround(x: number, y: number, radius = 120) {
  const angle = Math.random() * 2 * Math.PI
  const r = radius * (0.4 + 0.6 * Math.random())
  return { x: x + Math.cos(angle) * r, y: y + Math.sin(angle) * r }
}

// --- Seuils de zoom pour l'affichage des labels d'arêtes (zoom = 1 / camera.ratio)
export const EDGE_LABEL_ZOOM = 1.2
export const PORT_LABEL_ZOOM = 1.8

// --- Tailles ----------------------------------------------------------------
// Tailles proportionnelles au trafic, en échelle log : les gros parleurs
// ressortent sans écraser les hôtes discrets.
export const NODE_SIZE_MIN = 7
const NODE_SIZE_MAX = 18
export function nodeSizeFor(bytes: number) {
  return Math.min(NODE_SIZE_MAX, NODE_SIZE_MIN + 1.1 * Math.log2(1 + bytes / 2000))
}

const EDGE_SIZE_MIN = 1.2
const EDGE_SIZE_MAX = 7
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
export function drawNodeLabel(context: CanvasRenderingContext2D, data: any, settings: any) {
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
export function nodeAttributes(node: any) {
  const color = node.color || "#2196F3"
  const rawLabel = node.label || ""
  const macs: string[] = Array.isArray(node.macs) ? node.macs : []
  // Plusieurs MAC unicast pour une même IP : anomalie (IP partagée, VRRP,
  // usurpation ARP…) signalée par une bordure d'alerte.
  const macConflict = macs.length > 1
  return {
    name: node.name || node.id,
    mac: node.mac || "",
    macs,
    macConflict,
    ip: node.ip || "",
    rawLabel,
    label: rawLabel || node.name || node.id,
    color,
    borderColor: macConflict ? MAC_CONFLICT_BORDER_COLOR : darken(color, 0.25),
    hoverColor: brighten(color, 0.18),
  }
}

export function edgeAttributes(e: any) {
  const totalBytes = Number(e.total_bytes) || 0
  return {
    protocol: e.label || "",
    source_port: e.source_port ?? null,
    destination_port: e.destination_port ?? null,
    // Tous les ports « service » observés sur l'arête (triés, plafonnés
    // backend) : plusieurs services entre deux équipements ne sont plus
    // masqués par le premier couple de ports.
    ports: Array.isArray(e.ports) ? e.ports : [],
    bidir: !!e.bidir,
    count: Number(e.count) || 0,
    total_bytes: totalBytes,
    // Tunnels (encap_id hex) auxquels ce flux participe : le backend envoie
    // la liste cumulative, on remplace donc simplement à chaque update.
    encapIds: Array.isArray(e.encap_ids) ? e.encap_ids : [],
    color: e._color || colorForProtocol(e.label || ""),
    size: edgeSizeFor(totalBytes),
  }
}
