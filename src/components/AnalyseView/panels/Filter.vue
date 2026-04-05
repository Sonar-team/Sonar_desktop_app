<template>
  <div class="panel-backdrop" v-if="visible" @click.self="$emit('update:visible', false)">
    <div class="panel">

      <!-- Header -->
      <div class="panel-header">
        <div class="panel-title">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 3.5h12M4.5 7.5h7M7 11.5h2"/>
          </svg>
          Filtre BPF
        </div>
        <button class="close-btn" @click="$emit('update:visible', false)" title="Fermer">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="4" y1="4" x2="12" y2="12"/>
            <line x1="12" y1="4" x2="4" y2="12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="panel-body">

        <!-- Presets -->
        <div class="section">
          <div class="section-label">Presets</div>
          <div class="chips">
            <button class="chip" @click="preset('ipv4')">IPv4</button>
            <button class="chip" @click="preset('web')">Web 80/443</button>
            <button class="chip" @click="preset('dns')">DNS 53</button>
            <button class="chip" @click="preset('ntp')">NTP 123</button>
            <button class="chip" @click="preset('syn')">TCP SYN</button>
            <button class="chip" @click="preset('no-arp-ipv6')">Sans ARP/IPv6</button>
          </div>
        </div>

        <div class="divider"/>

        <!-- Layer / Types -->
        <div class="section">
          <div class="section-label">Couche / Types</div>
          <div class="checks">
            <label class="check-row">
              <input type="checkbox" v-model="opt.vlan" />
              <span>Trafic VLAN (802.1Q)</span>
            </label>
            <label class="check-row">
              <input type="checkbox" v-model="opt.onlyIp4" />
              <span>Uniquement IPv4</span>
            </label>
            <label class="check-row">
              <input type="checkbox" v-model="opt.excludeIpv6" />
              <span>Exclure IPv6</span>
            </label>
            <label class="check-row">
              <input type="checkbox" v-model="opt.excludeArp" />
              <span>Exclure ARP</span>
            </label>
          </div>
        </div>

        <div class="divider"/>

        <!-- Protocols -->
        <div class="section">
          <div class="section-label">Protocoles <span class="hint-inline">laisser vide = pas de restriction</span></div>
          <div class="checks horizontal">
            <label class="check-row"><input type="checkbox" v-model="proto.tcp"   /><span>TCP</span></label>
            <label class="check-row"><input type="checkbox" v-model="proto.udp"   /><span>UDP</span></label>
            <label class="check-row"><input type="checkbox" v-model="proto.icmp"  /><span>ICMP</span></label>
            <label class="check-row"><input type="checkbox" v-model="proto.icmp6" /><span>ICMPv6</span></label>
          </div>
        </div>

        <div class="divider"/>

        <!-- IP -->
        <div class="section">
          <div class="section-label">Adresses IP</div>
          <div class="field-row">
            <label>Inclure hôte</label>
            <input v-model="ip.includeHost" placeholder="192.168.1.42" />
          </div>
          <div class="field-row">
            <label>Exclure hôte</label>
            <input v-model="ip.excludeHost" placeholder="192.168.1.4" />
          </div>
          <div class="field-row">
            <label>Inclure réseau</label>
            <input v-model="ip.includeNet" placeholder="10.0.0.0/8" />
          </div>
          <div class="field-row">
            <label>Exclure réseau</label>
            <input v-model="ip.excludeNet" placeholder="192.168.0.0/16" />
          </div>
          <div class="field-row">
            <label>Direction</label>
            <select v-model="ip.direction">
              <option value="any">src ou dst</option>
              <option value="src">src</option>
              <option value="dst">dst</option>
            </select>
          </div>
          <div class="errors" v-if="ipErrors.length">
            <span v-for="e in ipErrors" :key="e">• {{ e }}</span>
          </div>
        </div>

        <div class="divider"/>

        <!-- Ports -->
        <div class="section">
          <div class="section-label">Ports <span class="hint-inline">TCP/UDP</span></div>
          <div class="field-row">
            <label>Inclure</label>
            <input v-model="ports.include" placeholder="80,443,22" />
          </div>
          <div class="field-row">
            <label>Exclure</label>
            <input v-model="ports.exclude" placeholder="25,21" />
          </div>
          <div class="field-row">
            <label>Plage</label>
            <input v-model="ports.range" placeholder="10000-20000" />
          </div>
          <div class="field-row">
            <label>Direction</label>
            <select v-model="ports.direction">
              <option value="any">src ou dst</option>
              <option value="src">src</option>
              <option value="dst">dst</option>
            </select>
          </div>
          <div class="errors" v-if="portErrors.length">
            <span v-for="e in portErrors" :key="e">• {{ e }}</span>
          </div>
        </div>

        <div class="divider"/>

        <!-- Preview -->
        <div class="section">
          <div class="section-label">Aperçu du filtre</div>
          <textarea
            class="preview-box"
            :value="previewText"
            @input="onPreviewInput"
            placeholder="Filtre BPF généré…"
            rows="3"
          ></textarea>
          <div class="errors" v-if="globalErrors.length">
            <span v-for="e in globalErrors" :key="e">• {{ e }}</span>
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <button class="btn-ghost" @click="resetAll">Réinitialiser</button>
        <button class="btn-primary" @click="apply" :disabled="!canApply">
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 7h10M8 3l4 4-4 4"/>
          </svg>
          Appliquer
        </button>
      </div>

    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useCaptureStore } from "../../../store/capture";
import type { CaptureEvent } from "../../../types/capture";

type Dir = "any" | "src" | "dst";

export default defineComponent({
  name: "BpfFilterBuilder",
  props: {
    visible: { type: Boolean, default: false },
  },
  emits: ["update:visible"],

  data() {
    return {
      opt: { vlan: false, onlyIp4: false, excludeIpv6: false, excludeArp: false },
      proto: { tcp: false, udp: false, icmp: false, icmp6: false },
      ip: { includeHost: "", excludeHost: "", includeNet: "", excludeNet: "", direction: "any" as Dir },
      ports: { include: "", exclude: "", range: "", direction: "any" as Dir },
      size: { less: undefined as number | undefined, greater: undefined as number | undefined },
      advanced: { raw: "" },
      previewText: "",
      isManualPreview: false,
    };
  },

  computed: {
    ipErrors(): string[] {
      const errs: string[] = [];
      const isIp   = (s: string) => /^(\d{1,3}\.){3}\d{1,3}$/.test(s);
      const isCidr = (s: string) => /^(\d{1,3}\.){3}\d{1,3}\/\d{1,2}$/.test(s);
      if (this.ip.includeHost && !isIp(this.ip.includeHost))   errs.push(`IP invalide: ${this.ip.includeHost}`);
      if (this.ip.excludeHost && !isIp(this.ip.excludeHost))   errs.push(`IP invalide: ${this.ip.excludeHost}`);
      if (this.ip.includeNet  && !isCidr(this.ip.includeNet))  errs.push(`CIDR invalide: ${this.ip.includeNet}`);
      if (this.ip.excludeNet  && !isCidr(this.ip.excludeNet))  errs.push(`CIDR invalide: ${this.ip.excludeNet}`);
      return errs;
    },
    portErrors(): string[] {
      const errs: string[] = [];
      const isPortList = (s: string) => s.split(",").every(p => /^\s*\d{1,5}\s*$/.test(p) && Number(p) <= 65535);
      const isRange    = (s: string) => /^\s*\d{1,5}\s*-\s*\d{1,5}\s*$/.test(s);
      if (this.ports.include && !isPortList(this.ports.include)) errs.push("Ports à inclure invalides");
      if (this.ports.exclude && !isPortList(this.ports.exclude)) errs.push("Ports à exclure invalides");
      if (this.ports.range   && !isRange(this.ports.range))      errs.push("Plage de ports invalide");
      return errs;
    },
    globalErrors(): string[] { return [...this.ipErrors, ...this.portErrors]; },

    autoPreview(): string {
      const c: string[] = [];
      const groupOr = (cl: string[]) => cl.length > 1 ? `(${cl.join(" or ")})` : cl[0] ?? "";
      const dirPfx  = (d: Dir) => d === "any" ? "" : `${d} `;

      if (this.opt.vlan) c.push("vlan");
      if (this.opt.onlyIp4) c.push("ip");
      if (this.opt.excludeIpv6) c.push("not ip6");
      if (this.opt.excludeArp) c.push("not arp");

      const protos: string[] = [];
      if (this.proto.tcp)   protos.push("tcp");
      if (this.proto.udp)   protos.push("udp");
      if (this.proto.icmp)  protos.push("icmp");
      if (this.proto.icmp6) protos.push("icmp6");
      if (protos.length) c.push(groupOr(protos));

      if (this.ip.includeHost) c.push(`${dirPfx(this.ip.direction)}host ${this.ip.includeHost}`);
      if (this.ip.excludeHost) c.push(`not ${dirPfx(this.ip.direction)}host ${this.ip.excludeHost}`);
      if (this.ip.includeNet)  c.push(`${dirPfx(this.ip.direction)}net ${this.ip.includeNet}`);
      if (this.ip.excludeNet)  c.push(`not ${dirPfx(this.ip.direction)}net ${this.ip.excludeNet}`);

      const addPorts = (list: string, negate = false) => {
        if (!list.trim()) return;
        const parts = list.split(",").map(s => s.trim()).filter(Boolean);
        const clauses = parts.map(p => `${dirPfx(this.ports.direction)}port ${p}`);
        c.push((negate ? "not " : "") + groupOr(clauses));
      };
      addPorts(this.ports.include, false);
      addPorts(this.ports.exclude, true);
      if (this.ports.range.trim()) c.push(`${dirPfx(this.ports.direction)}portrange ${this.ports.range.trim()}`);
      if (this.size.less    && this.size.less > 0)    c.push(`less ${this.size.less}`);
      if (this.size.greater && this.size.greater > 0) c.push(`greater ${this.size.greater}`);
      if (this.advanced.raw.trim()) c.push(this.advanced.raw.trim());

      return c.join(" and ").trim();
    },

    canApply(): boolean { return this.globalErrors.length === 0; },
    captureStore() { return useCaptureStore(); },
  },

  watch: {
    autoPreview: {
      immediate: true,
      handler(v: string) { if (!this.isManualPreview) this.previewText = v; },
    },
  },

  methods: {
    async apply() {
      if (!this.canApply) return;
      const filter = this.previewText.trim();
      try {
        await invoke("set_filter", { filter });
      } catch (e) {
        console.error("set_filter failed:", e);
        return;
      }

      // Si capture active → stop + restart pour appliquer le filtre immédiatement
      if (this.captureStore.isRunning) {
        const onEvent = this.captureStore.getChannel();
        try {
          await invoke("stop_capture", { onEvent });
        } catch (e) { console.error("stop_capture failed:", e); }

        const newChannel = new Channel<CaptureEvent>();
        this.captureStore.setChannel(newChannel);
        try {
          const status = await invoke("start_capture", { onEvent: newChannel }) as { is_running: boolean };
          this.captureStore.updateStatus(status);
        } catch (e) { console.error("start_capture failed:", e); }
      }

      this.$emit("update:visible", false);
    },
    onPreviewInput(e: Event) {
      this.isManualPreview = true;
      this.previewText = (e.target as HTMLTextAreaElement).value;
    },
    resetAll() {
      this.opt     = { vlan: false, onlyIp4: false, excludeIpv6: false, excludeArp: false };
      this.proto   = { tcp: false, udp: false, icmp: false, icmp6: false };
      this.ip      = { includeHost: "", excludeHost: "", includeNet: "", excludeNet: "", direction: "any" as Dir };
      this.ports   = { include: "", exclude: "", range: "", direction: "any" as Dir };
      this.size    = { less: undefined, greater: undefined };
      this.advanced.raw = "";
      this.isManualPreview = false;
      this.previewText = "";
    },
    preset(name: string) {
      this.resetAll();
      switch (name) {
        case "ipv4":        this.opt.onlyIp4 = true; break;
        case "web":         this.opt.onlyIp4 = true; this.proto.tcp = true; this.ports.include = "80,443"; break;
        case "dns":         this.opt.onlyIp4 = true; this.proto.udp = true; this.proto.tcp = true; this.ports.include = "53"; break;
        case "ntp":         this.opt.onlyIp4 = true; this.proto.udp = true; this.ports.include = "123"; break;
        case "syn":         this.opt.onlyIp4 = true; this.proto.tcp = true; this.advanced.raw = "tcp[13] & 0x02 != 0 and tcp[13] & 0x10 = 0"; break;
        case "no-arp-ipv6": this.opt.excludeArp = true; this.opt.excludeIpv6 = true; break;
      }
    },
  },
});
</script>

<style scoped>
.panel-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}

.panel {
  width: 380px;
  height: 100%;
  background: #363648;
  border-left: 1px solid #4a4a60;
  display: flex;
  flex-direction: column;
  animation: slide-in 0.22s cubic-bezier(0.25, 1, 0.5, 1);
}

@keyframes slide-in {
  from { transform: translateX(18px); opacity: 0; }
  to   { transform: translateX(0);     opacity: 1; }
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 14px 0 16px;
  border-bottom: 1px solid #4a4a60;
  flex-shrink: 0;
}
.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 500;
  color: #a8a8be;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}
.panel-title svg { width: 14px; height: 14px; color: #707090; }

.close-btn {
  width: 26px; height: 26px;
  background: transparent; border: none; border-radius: 4px;
  color: #686884; cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.15s, color 0.15s;
}
.close-btn svg { width: 12px; height: 12px; }
.close-btn:hover { background: #484860; color: #b0b0c8; }

/* Body */
.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.divider { height: 1px; background: #4a4a60; }

/* Sections */
.section { display: flex; flex-direction: column; gap: 8px; }

.section-label {
  font-size: 11px;
  font-weight: 500;
  color: #808096;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  display: flex;
  align-items: center;
  gap: 6px;
}
.hint-inline {
  font-size: 10px;
  color: #686884;
  text-transform: none;
  letter-spacing: 0;
  font-weight: 400;
}

/* Chips */
.chips { display: flex; flex-wrap: wrap; gap: 5px; }
.chip {
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 99px;
  padding: 3px 10px;
  font-size: 11px;
  color: #808096;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s, transform 0.1s ease;
}
.chip:hover  { border-color: #6868a0; color: #c0c0d8; background: #38385a; }
.chip:active { transform: scale(0.93); }

/* Checkboxes */
.checks { display: flex; flex-direction: column; gap: 6px; }
.checks.horizontal { flex-direction: row; flex-wrap: wrap; gap: 12px; }

.check-row {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: #909096;
  cursor: pointer;
  user-select: none;
}
.check-row input[type="checkbox"] {
  width: 13px;
  height: 13px;
  accent-color: #4a6888;
  cursor: pointer;
  flex-shrink: 0;
}

/* Field rows */
.field-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.field-row label {
  font-size: 11px;
  color: #686884;
  width: 90px;
  flex-shrink: 0;
  text-align: right;
}
.field-row input,
.field-row select {
  flex: 1;
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 4px;
  color: #b0b0c8;
  font-size: 12px;
  padding: 6px 10px;
  transition: border-color 0.15s, color 0.15s;
}
.field-row input:focus,
.field-row select:focus {
  outline: none;
  border-color: #5a6a80;
  color: #d0d0e0;
  box-shadow: 0 0 0 2px rgba(90, 106, 128, 0.18);
}
.field-row select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23686884' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round' fill='none'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  padding-right: 24px;
  cursor: pointer;
}
.field-row select option { background: #363648; color: #b0b0c8; }

/* Preview */
.preview-box {
  width: 100%;
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 4px;
  color: #909098;
  font-size: 12px;
  font-family: monospace;
  padding: 8px 10px;
  resize: none;
  box-sizing: border-box;
  transition: border-color 0.15s;
  line-height: 1.5;
}
.preview-box:focus { outline: none; border-color: #5a6a80; }

/* Errors */
.errors {
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: 11px;
  color: #8a5858;
}

/* Footer */
.panel-footer {
  display: flex;
  gap: 8px;
  padding: 14px 16px;
  border-top: 1px solid #4a4a60;
  flex-shrink: 0;
}

.btn-ghost {
  flex: 1;
  padding: 8px 14px;
  background: transparent;
  border: 1px solid #484860;
  border-radius: 5px;
  color: #808096;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-ghost:active { transform: scale(0.97); }
.btn-ghost:hover { border-color: #60607a; color: #a8a8be; }

.btn-primary {
  flex: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 14px;
  background: #304868;
  border: 1px solid #3e5c80;
  border-radius: 5px;
  color: #88b4cc;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-primary:active { transform: scale(0.97); }
.btn-primary svg { width: 13px; height: 13px; }
.btn-primary:hover { background: #3a5878; border-color: #4a6888; color: #aaced0; }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
</style>