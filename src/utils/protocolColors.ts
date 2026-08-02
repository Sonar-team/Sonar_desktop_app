// Couleurs des protocoles dans le graphe réseau : palette fixe pour les
// protocoles connus, couleur dérivée par hachage stable pour les autres.
// Alimente aussi la légende du graphe.
type ProtocolColorLegendItem = {
  label: string
  color: string
}

const UNKNOWN_PROTOCOL_COLOR = "#94A3B8"

const HASH_FALLBACK_COLORS = Object.freeze([
  "#22D3EE",
  "#A78BFA",
  "#34D399",
  "#F472B6",
  "#FBBF24",
  "#60A5FA",
  "#FB7185",
  "#C084FC",
  "#2DD4BF",
  "#F97316",
])

// Labels emitted by packet_parser 1.5.4 are normalized so values such as
// "EtherNet/IP", "OPC UA", "Q-in-Q", and "MPLS Unicast" remain stable keys.
const PROTOCOL_COLORS: Readonly<Record<string, string>> = Object.freeze({
  // Application protocols
  http: "#0EA5E9",
  dhcp: "#FBBF24",
  dhcpv6: "#F59E0B",
  ntp: "#F59E0B",
  ams: "#A855F7",
  bitcoin: "#F7931A",
  opcua: "#7C3AED",
  ethernetip: "#D97706",
  dns: "#EF4444",
  mqtt: "#22C55E",
  postgresql: "#38BDF8",
  snmp: "#10B981",
  tls: "#3B82F6",
  s7comm: "#E11D48",
  cotp: "#F97316",
  copt: "#F97316",
  giop: "#8B5CF6",
  srvloc: "#14B8A6",
  srvlock: "#14B8A6",
  modbustcp: "#DC2626",
  quiq: "#06B6D4",
  quic: "#06B6D4",

  // Internet and data-link protocols / Ethertypes
  arp: "#FACC15",
  rarp: "#FACC15",
  aarp: "#FDBA74",
  ipv4: "#F97316",
  minipv4: "#FB923C",
  ipv6: "#A855F7",
  profinet: "#22C55E",
  profinetrt: "#22C55E",
  vlantaggedframe: "#84CC16",
  qinq: "#A3E635",
  pbridge: "#65A30D",
  lldp: "#38BDF8",
  mrp: "#4ADE80",
  ptp: "#F59E0B",
  trill: "#2DD4BF",
  decnet: "#A78BFA",
  appletalk: "#FB7185",
  ipx: "#C084FC",
  qnx: "#94A3B8",
  cobtranet: "#34D399",
  cfm: "#9CA3AF",
  mplsunicast: "#60A5FA",
  mplsmulticast: "#818CF8",
  pppoediscoverystage: "#F472B6",
  pppoesessionstage: "#EC4899",

  // Common transport / IPv6 next-header protocols
  tcp: "#2563EB",
  udp: "#0EA5E9",
  udplite: "#38BDF8",
  icmp: "#F43F5E",
  icmpv6: "#FB7185",
  igmp: "#A3E635",
  pim: "#84CC16",
  vrrp: "#EAB308",
  egp: "#84CC16",
  igp: "#84CC16",
  ospfigp: "#84CC16",
  eigrp: "#84CC16",
  gre: "#64748B",
  ipip: "#7DD3FC",
  etherip: "#7DD3FC",
  encap: "#7DD3FC",
  l2tp: "#7DD3FC",
  mplsinip: "#60A5FA",
  esp: "#6366F1",
  ah: "#8B5CF6",
  ipcomp: "#A78BFA",
  sctp: "#06B6D4",
  dccp: "#0284C7",
  rsvp: "#F59E0B",
  rsvpe2eignore: "#F59E0B",
})

const IPV6_EXTENSION_KEYS = Object.freeze(new Set([
  "ipv6hopbyhop",
  "ipv6routingheader",
  "ipv6fragmentheader",
  "ipv6destinationoptions",
  "ipv6mobilityheader",
  "nonextheader",
  "hopopt",
]))

const ROUTING_PROTOCOL_KEYS = Object.freeze(new Set([
  "bbnrccmon",
  "cbt",
  "dcnmeas",
  "dsr",
  "idpr",
  "idprcmtp",
  "idrp",
  "isisoveripv4",
  "larp",
  "manet",
  "mtp",
  "narp",
  "pnni",
  "sdrp",
  "sunnd",
  "xnsidp",
]))

const SECURITY_PROTOCOL_KEYS = Object.freeze(new Set([
  "anyprivateencryptionscheme",
  "hip",
  "rohc",
  "securevmtp",
  "shim6",
  "skip",
  "tlsp",
  "wesp",
]))

function protocolColorKey(label: string): string {
  return label.trim().toLowerCase().replace(/[^a-z0-9]+/g, "")
}

function fallbackColorForKey(key: string): string {
  let hash = 0
  for (let i = 0; i < key.length; i++) {
    // `| 0` n'est pas une troncature de flottant ici (hash est déjà entier) :
    // c'est un rebouclage volontaire sur 32 bits signés (hash de type
    // String.hashCode Java), qui borne la valeur pour des clés longues.
    // `Math.trunc` ne reproduit pas ce rebouclage — ne pas « corriger » S7767 ici.
    hash = (hash * 31 + key.charCodeAt(i)) | 0
  }
  return HASH_FALLBACK_COLORS[Math.abs(hash) % HASH_FALLBACK_COLORS.length]
}

export function colorForProtocol(label?: string | null): string {
  const raw = label?.trim() ?? ""
  if (!raw) return UNKNOWN_PROTOCOL_COLOR

  const key = protocolColorKey(raw)
  if (!key || key === "unknown" || key.startsWith("unknown")) return UNKNOWN_PROTOCOL_COLOR

  const exact = PROTOCOL_COLORS[key]
  if (exact) return exact
  if (IPV6_EXTENSION_KEYS.has(key) || key.startsWith("ipv6")) return PROTOCOL_COLORS.ipv6
  if (ROUTING_PROTOCOL_KEYS.has(key)) return "#84CC16"
  if (SECURITY_PROTOCOL_KEYS.has(key)) return "#8B5CF6"
  if (key.includes("mpls")) return "#60A5FA"
  if (key.includes("ip") && (key.includes("inip") || key.includes("over"))) return "#7DD3FC"

  return fallbackColorForKey(key)
}

export const PROTOCOL_COLOR_LEGEND: readonly ProtocolColorLegendItem[] = Object.freeze([
  { label: "TCP", color: colorForProtocol("Tcp") },
  { label: "UDP", color: colorForProtocol("UDP") },
  { label: "DNS", color: colorForProtocol("DNS") },
  { label: "TLS", color: colorForProtocol("TLS") },
  { label: "QUIC", color: colorForProtocol("QUIQ") },
  { label: "ARP", color: colorForProtocol("ARP") },
  { label: "IPv4", color: colorForProtocol("IPv4") },
  { label: "IPv6", color: colorForProtocol("IPv6") },
  { label: "Profinet", color: colorForProtocol("Profinet") },
  { label: "OPC UA", color: colorForProtocol("OPC UA") },
  { label: "S7Comm", color: colorForProtocol("S7Comm") },
  { label: "ModbusTCP", color: colorForProtocol("ModbusTCP") },
  { label: "LLDP", color: colorForProtocol("LLDP") },
  { label: "Tunnel", color: colorForProtocol("GRE") },
  { label: "Inconnu", color: UNKNOWN_PROTOCOL_COLOR },
])
