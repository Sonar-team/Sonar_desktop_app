// Construction et validation d'expressions BPF à partir du formulaire du
// panneau de filtre. Logique pure, sans dépendance à Vue ni à Tauri.

export type Dir = 'any' | 'src' | 'dst';

export interface BpfLayerOptions {
  vlan: boolean;
  onlyIp4: boolean;
  excludeIpv6: boolean;
  excludeArp: boolean;
}

export interface BpfProtoOptions {
  tcp: boolean;
  udp: boolean;
  icmp: boolean;
  icmp6: boolean;
}

export interface BpfIpFields {
  includeHost: string;
  excludeHost: string;
  includeNet: string;
  excludeNet: string;
  direction: Dir;
}

export interface BpfPortFields {
  include: string;
  exclude: string;
  range: string;
  direction: Dir;
}

export interface BpfFormState {
  opt: BpfLayerOptions;
  proto: BpfProtoOptions;
  ip: BpfIpFields;
  ports: BpfPortFields;
  advancedRaw: string;
}

export function emptyBpfFormState(): BpfFormState {
  return {
    opt: { vlan: false, onlyIp4: false, excludeIpv6: false, excludeArp: false },
    proto: { tcp: false, udp: false, icmp: false, icmp6: false },
    ip: { includeHost: '', excludeHost: '', includeNet: '', excludeNet: '', direction: 'any' },
    ports: { include: '', exclude: '', range: '', direction: 'any' },
    advancedRaw: '',
  };
}

// --- Validation ---------------------------------------------------------

export function validateIpFields(ip: BpfIpFields): string[] {
  const errs: string[] = [];
  const isIp = (s: string) => /^(\d{1,3}\.){3}\d{1,3}$/.test(s);
  const isCidr = (s: string) => /^(\d{1,3}\.){3}\d{1,3}\/\d{1,2}$/.test(s);
  if (ip.includeHost && !isIp(ip.includeHost)) errs.push(`IP invalide : ${ip.includeHost}`);
  if (ip.excludeHost && !isIp(ip.excludeHost)) errs.push(`IP invalide : ${ip.excludeHost}`);
  if (ip.includeNet && !isCidr(ip.includeNet)) errs.push(`CIDR invalide : ${ip.includeNet}`);
  if (ip.excludeNet && !isCidr(ip.excludeNet)) errs.push(`CIDR invalide : ${ip.excludeNet}`);
  return errs;
}

export function validatePortFields(ports: BpfPortFields): string[] {
  const errs: string[] = [];
  const isPortList = (s: string) =>
    s.split(',').every(p => /^\s*\d{1,5}\s*$/.test(p) && Number(p.trim()) <= 65535);
  const isRange = (s: string) =>
    /^\s*\d{1,5}\s*-\s*\d{1,5}\s*$/.test(s) &&
    Number(s.split('-')[0]) <= 65535 &&
    Number(s.split('-')[1]) <= 65535;
  if (ports.include && !isPortList(ports.include)) errs.push('Ports à inclure invalides');
  if (ports.exclude && !isPortList(ports.exclude)) errs.push('Ports à exclure invalides');
  if (ports.range && !isRange(ports.range)) errs.push('Plage de ports invalide (ex: 10000-20000)');
  return errs;
}

// --- Construction ---------------------------------------------------------

/** Assemble l'expression BPF (clauses jointes par `and`) depuis le formulaire. */
export function buildBpfExpression(state: BpfFormState): string {
  const { opt, proto, ip, ports } = state;
  const c: string[] = [];
  const groupOr = (clauses: string[]) => clauses.length > 1 ? `(${clauses.join(' or ')})` : clauses[0] ?? '';
  const dirPfx = (d: Dir) => d === 'any' ? '' : `${d} `;

  if (opt.vlan) c.push('vlan');
  if (opt.onlyIp4) c.push('ip');
  if (opt.excludeIpv6) c.push('not ip6');
  if (opt.excludeArp) c.push('not arp');

  const protos: string[] = [];
  if (proto.tcp) protos.push('tcp');
  if (proto.udp) protos.push('udp');
  if (proto.icmp) protos.push('icmp');
  if (proto.icmp6) protos.push('icmp6');
  if (protos.length) c.push(groupOr(protos));

  if (ip.includeHost) c.push(`${dirPfx(ip.direction)}host ${ip.includeHost}`);
  if (ip.excludeHost) c.push(`not ${dirPfx(ip.direction)}host ${ip.excludeHost}`);
  if (ip.includeNet) c.push(`${dirPfx(ip.direction)}net ${ip.includeNet}`);
  if (ip.excludeNet) c.push(`not ${dirPfx(ip.direction)}net ${ip.excludeNet}`);

  const addPorts = (list: string, negate = false) => {
    if (!list.trim()) return;
    const op = negate ? 'not ' : '';
    const parts = list.split(',').map(s => s.trim()).filter(Boolean);
    const clauses = parts.map(p => `${dirPfx(ports.direction)}port ${p}`);
    c.push(op + groupOr(clauses));
  };
  addPorts(ports.include);
  addPorts(ports.exclude, true);
  if (ports.range.trim()) c.push(`${dirPfx(ports.direction)}portrange ${ports.range.trim()}`);

  if (state.advancedRaw.trim()) c.push(state.advancedRaw.trim());

  return c.join(' and ').trim();
}

// --- Presets ---------------------------------------------------------------

/** Patch partiel appliqué sur un formulaire vierge. */
export interface BpfPreset {
  opt?: Partial<BpfLayerOptions>;
  proto?: Partial<BpfProtoOptions>;
  ports?: Partial<BpfPortFields>;
  advancedRaw?: string;
}

export const BPF_PRESETS: Record<string, BpfPreset> = {
  ipv4: { opt: { onlyIp4: true } },
  web: { opt: { onlyIp4: true }, proto: { tcp: true }, ports: { include: '80,443' } },
  dns: { opt: { onlyIp4: true }, proto: { udp: true, tcp: true }, ports: { include: '53' } },
  ntp: { opt: { onlyIp4: true }, proto: { udp: true }, ports: { include: '123' } },
  syn: {
    opt: { onlyIp4: true },
    proto: { tcp: true },
    advancedRaw: 'tcp[13] & 0x02 != 0 and tcp[13] & 0x10 = 0',
  },
  'no-arp-ipv6': { opt: { excludeArp: true, excludeIpv6: true } },
};
