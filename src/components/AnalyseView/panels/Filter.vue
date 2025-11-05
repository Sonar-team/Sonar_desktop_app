<template>
  <div class="bpf-wrapper">
    <div class="panel">

      <!-- LAYER / PROTO -->
      <section class="grid two">
        <div class="card">
          <h3>Couche / Types</h3>
          <label class="row">
            <input type="checkbox" v-model="opt.vlan" />
            <span>Trafic VLAN (802.1Q)</span>
          </label>
          <label class="row">
            <input type="checkbox" v-model="opt.onlyIp4" />
            <span>Uniquement IPv4 (équiv. <code>ip</code>)</span>
          </label>
          <label class="row">
            <input type="checkbox" v-model="opt.excludeIpv6" />
            <span>Exclure IPv6 (<code>not ip6</code>)</span>
          </label>
          <label class="row">
            <input type="checkbox" v-model="opt.excludeArp" />
            <span>Exclure ARP (<code>not arp</code>)</span>
          </label>
        </div>

        <div class="card">
          <h3>Protocoles</h3>
          <div class="row">
            <label><input type="checkbox" v-model="proto.tcp" /> TCP</label>
            <label><input type="checkbox" v-model="proto.udp" /> UDP</label>
            <label><input type="checkbox" v-model="proto.icmp" /> ICMP</label>
            <label><input type="checkbox" v-model="proto.icmp6" /> ICMPv6</label>
          </div>
          <div class="hint">Si rien n’est coché ici, on ne restreint pas par protocole.</div>
        </div>
      </section>

      <!-- IP / NET / PORTS -->
      <section class="grid two">
        <div class="card">
          <h3>Adresses IP</h3>

          <div class="row">
            <label>Inclure hôte</label>
            <input v-model="ip.includeHost" placeholder="ex: 192.168.1.42" />
          </div>
          <div class="row">
            <label>Exclure hôte</label>
            <input v-model="ip.excludeHost" placeholder="ex: 192.168.1.4" />
          </div>

          <div class="row">
            <label>Inclure réseau</label>
            <input v-model="ip.includeNet" placeholder="ex: 10.0.0.0/8" />
          </div>
          <div class="row">
            <label>Exclure réseau</label>
            <input v-model="ip.excludeNet" placeholder="ex: 192.168.0.0/16" />
          </div>

          <div class="row">
            <label>Direction</label>
            <select v-model="ip.direction">
              <option value="any">src ou dst</option>
              <option value="src">src</option>
              <option value="dst">dst</option>
            </select>
          </div>

          <div class="errors" v-if="ipErrors.length">
            <div v-for="e in ipErrors" :key="e">• {{ e }}</div>
          </div>
        </div>

        <div class="card">
          <h3>Ports</h3>
          <div class="row">
            <label>Inclure port</label>
            <input v-model="ports.include" placeholder="ex: 80,443,22" />
          </div>
          <div class="row">
            <label>Exclure port</label>
            <input v-model="ports.exclude" placeholder="ex: 25,21" />
          </div>
          <div class="row">
            <label>Plage (portrange)</label>
            <input v-model="ports.range" placeholder="ex: 10000-20000" />
          </div>
          <div class="row">
            <label>Direction</label>
            <select v-model="ports.direction">
              <option value="any">src ou dst</option>
              <option value="src">src</option>
              <option value="dst">dst</option>
            </select>
          </div>

          <div class="hint">Les ports s’appliquent surtout à TCP/UDP.</div>
          <div class="errors" v-if="portErrors.length">
            <div v-for="e in portErrors" :key="e">• {{ e }}</div>
          </div>
        </div>
      </section>

      <!-- PREVIEW / ACTIONS -->
      <section class="card preview">
        <h3>Aperçu du filtre</h3>
        <pre class="preview-box" >Filtre actuel : {{ preview }}</pre>

        <div class="actions">
          <button class="primary" @click="apply" :disabled="!canApply">Appliquer</button>
          <button class="ghost" @click="resetAll">Réinitialiser</button>
        </div>

        <div class="errors" v-if="globalErrors.length">
          <div v-for="e in globalErrors" :key="e">• {{ e }}</div>
        </div>
      </section>

      <!-- PRESETS -->
      <section class="card">
        <h3>Presets rapides</h3>
        <div class="chips">
          <button class="chip" @click="preset('ipv4')">IPv4 only</button>
          <button class="chip" @click="preset('web')">Web (80/443)</button>
          <button class="chip" @click="preset('dns')">DNS (53)</button>
          <button class="chip" @click="preset('ntp')">NTP (123)</button>
          <button class="chip" @click="preset('syn')">TCP SYN only</button>
          <button class="chip" @click="preset('no-arp-ipv6')">Tout sauf ARP/IPv6</button>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

type Dir = "any" | "src" | "dst";

const opt = reactive({
  vlan: false,
  onlyIp4: true,
  excludeIpv6: false,
  excludeArp: false,
});

const proto = reactive({
  tcp: false,
  udp: false,
  icmp: false,
  icmp6: false,
});

const ip = reactive({
  includeHost: "",
  excludeHost: "",
  includeNet: "",
  excludeNet: "",
  direction: "any" as Dir,
});

const ports = reactive({
  include: "",
  exclude: "",
  range: "",
  direction: "any" as Dir,
});

const size = reactive({
  less: undefined as number | undefined,
  greater: undefined as number | undefined,
});

const advanced = reactive({
  raw: "",
});

const ipErrors = computed(() => {
  const errs: string[] = [];
  const isIp = (s: string) => /^(\d{1,3}\.){3}\d{1,3}$/.test(s);
  const isCidr = (s: string) => /^(\d{1,3}\.){3}\d{1,3}\/\d{1,2}$/.test(s);

  if (ip.includeHost && !isIp(ip.includeHost)) errs.push(`IP invalide: ${ip.includeHost}`);
  if (ip.excludeHost && !isIp(ip.excludeHost)) errs.push(`IP invalide: ${ip.excludeHost}`);
  if (ip.includeNet && !isCidr(ip.includeNet)) errs.push(`CIDR invalide: ${ip.includeNet}`);
  if (ip.excludeNet && !isCidr(ip.excludeNet)) errs.push(`CIDR invalide: ${ip.excludeNet}`);
  return errs;
});

const portErrors = computed(() => {
  const errs: string[] = [];
  const isPortList = (s: string) =>
    s.split(",").every(p => /^\s*\d{1,5}\s*$/.test(p) && Number(p) <= 65535);
  const isRange = (s: string) =>
    /^\s*\d{1,5}\s*-\s*\d{1,5}\s*$/.test(s) &&
    Number(s.split("-")[0]) <= 65535 &&
    Number(s.split("-")[1]) <= 65535;

  if (ports.include && !isPortList(ports.include)) errs.push("Ports à inclure invalides");
  if (ports.exclude && !isPortList(ports.exclude)) errs.push("Ports à exclure invalides");
  if (ports.range && !isRange(ports.range)) errs.push("Plage de ports invalide (ex: 10000-20000)");
  return errs;
});

const globalErrors = computed(() => [...ipErrors.value, ...portErrors.value]);

const groupOr = (clauses: string[]) => clauses.length > 1 ? `(${clauses.join(" or ")})` : clauses[0] ?? "";
const dirPrefix = (d: Dir) => d === "any" ? "" : `${d} `;

const preview = computed(() => {
  const c: string[] = [];

  // VLAN
  if (opt.vlan) c.push("vlan");

  // IP / ETH types
  if (opt.onlyIp4) c.push("ip");
  if (opt.excludeIpv6) c.push("not ip6");
  if (opt.excludeArp) c.push("not arp");

  // Protocoles
  const protos: string[] = [];
  if (proto.tcp) protos.push("tcp");
  if (proto.udp) protos.push("udp");
  if (proto.icmp) protos.push("icmp");
  if (proto.icmp6) protos.push("icmp6");
  if (protos.length) c.push(groupOr(protos));

  // IP include/exclude
  if (ip.includeHost) c.push(`${dirPrefix(ip.direction)}host ${ip.includeHost}`);
  if (ip.excludeHost) c.push(`not ${dirPrefix(ip.direction)}host ${ip.excludeHost}`);
  if (ip.includeNet)  c.push(`${dirPrefix(ip.direction)}net ${ip.includeNet}`);
  if (ip.excludeNet)  c.push(`not ${dirPrefix(ip.direction)}net ${ip.excludeNet}`);

  // Ports include/exclude/range
  const addPorts = (list: string, negate = false) => {
    if (!list.trim()) return;
    const op = negate ? "not " : "";
    const parts = list.split(",").map(s => s.trim()).filter(Boolean);
    const clauses = parts.map(p => `${dirPrefix(ports.direction)}port ${p}`);
    c.push(op + groupOr(clauses));
  };
  addPorts(ports.include, false);
  addPorts(ports.exclude, true);

  if (ports.range.trim()) {
    const pr = `${dirPrefix(ports.direction)}portrange ${ports.range.trim()}`;
    c.push(pr);
  }

  // Taille
  if (size.less && size.less > 0) c.push(`less ${size.less}`);
  if (size.greater && size.greater > 0) c.push(`greater ${size.greater}`);

  // Avancé
  if (advanced.raw.trim()) c.push(advanced.raw.trim());

  return c.join(" and ").trim();
});

const canApply = computed(() => preview.value.length > 0 && globalErrors.value.length === 0);

async function apply() {
  if (!canApply.value) return;
  try {
    await invoke("set_filter", { filter: preview.value });
  } catch (e) {
    console.error("set_filter failed:", e);
  }
}

function resetAll() {
  Object.assign(opt, { vlan: false, onlyIp4: true, excludeIpv6: false, excludeArp: false });
  Object.assign(proto, { tcp: false, udp: false, icmp: false, icmp6: false });
  Object.assign(ip, { includeHost: "", excludeHost: "", includeNet: "", excludeNet: "", direction: "any" as Dir });
  Object.assign(ports, { include: "", exclude: "", range: "", direction: "any" as Dir });
  Object.assign(size, { less: undefined, greater: undefined });
  advanced.raw = "";
}

function preset(name: string) {
  resetAll();
  switch (name) {
    case "ipv4":
      opt.onlyIp4 = true;
      break;
    case "web":
      opt.onlyIp4 = true;
      proto.tcp = true;
      ports.include = "80,443";
      break;
    case "dns":
      opt.onlyIp4 = true;
      proto.udp = true; proto.tcp = true;
      ports.include = "53";
      break;
    case "ntp":
      opt.onlyIp4 = true;
      proto.udp = true;
      ports.include = "123";
      break;
    case "syn":
      opt.onlyIp4 = true;
      proto.tcp = true;
      advanced.raw = "tcp[13] & 0x02 != 0 and tcp[13] & 0x10 = 0";
      break;
    case "no-arp-ipv6":
      opt.onlyIp4 = false;
      opt.excludeArp = true;
      opt.excludeIpv6 = true;
      break;
  }
}
</script>

<style scoped>
/* Layout */
.bpf-wrapper { 
  position: fixed;
  top: 15px;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: 30px 16px 16px;
  z-index: 1000;
  pointer-events: none;
}
.panel { 
  width: min(1100px, 96vw); 
  display: flex; 
  flex-direction: column; 
  gap: 16px;
  background: #1e1e2e;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  pointer-events: auto;
  max-height: 90vh;
  overflow-y: auto;
}
.panel-header { display:flex; flex-direction:column; gap:4px; }
.panel-header h2 { margin:0; font-size:20px; }
.grid.two { display:grid; grid-template-columns: 1fr 1fr; gap: 16px; }
@media (max-width: 900px){ .grid.two{ grid-template-columns: 1fr; } }

/* Cards */
.card { background:#f8f8f8; border:1px solid #2c2c2c; border-radius:12px; padding:14px; }
.card h3 { margin:0 0 10px 0; font-size:16px; }

/* Rows / Inputs */
.row { display:flex; align-items:center; gap:10px; margin-bottom:8px; }
.row input[type="text"], .row input[type="number"], .row select, .card > input {
  width:100%; background:#adaaaa; color:#eaeaea; border:1px solid #3a3a3a; border-radius:8px; padding:8px 10px;
}
label.row { gap:8px; }
.hint { font-size:12px; opacity:.7; margin-top:6px; }

/* Preview */
.preview .preview-box {
  background:#aaa7a7; border:1px dashed #3a3a3a; border-radius:8px; padding:10px; white-space:pre-wrap; word-break:break-word;
}

/* Actions */
.actions { display:flex; gap:8px; margin-top:10px; }
button { background:#2b2b2b; color:#cb5151; border:1px solid #3a3a3a; border-radius:10px; padding:8px 14px; cursor:pointer; }
button.primary { background:#3a77ff; border-color:#3a77ff; color:white; }
button.ghost { background:transparent; }
button:disabled { opacity:.5; cursor:not-allowed; }

/* Chips */
.chips { display:flex; gap:8px; flex-wrap:wrap; margin-top:6px; }
.chip { background:#2b2b2b; border:1px solid #3a3a3a; border-radius:999px; padding:6px 10px; cursor:pointer; }
.chip:hover { border-color:#555; }

/* Errors */
.errors { margin-top:8px; color:#ff9a9a; font-size:13px; }
</style>
