<template>
  <div class="filter-overlay" @click="$emit('update:visible', false)">
    <div class="filter-panel" @click.stop>

      <!-- Header -->
      <div class="panel-header">
        <h2>Filtre BPF</h2>
        <button class="close-btn" @click="$emit('update:visible', false)">✕</button>
      </div>

      <!-- Filtre actif -->
      <div class="active-filter-bar" v-if="captureStore.activeFilter">
        <span class="active-filter-label">Filtre actif</span>
        <code class="active-filter-expr">{{ captureStore.activeFilter }}</code>
        <button class="btn ghost active-filter-clear" @click="resetAll">Supprimer</button>
      </div>

      <!-- Presets rapides -->
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

      <!-- Couche / Protocoles -->
      <section class="grid two">
        <div class="card">
          <h3>Couche / Types</h3>
          <label class="row">
            <input type="checkbox" v-model="opt.vlan" />
            <span>Trafic VLAN (802.1Q)</span>
          </label>
          <label class="row">
            <input type="checkbox" v-model="opt.onlyIp4" />
            <span>Uniquement IPv4 (<code>ip</code>)</span>
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
          <div class="proto-grid">
            <label class="row"><input type="checkbox" v-model="proto.tcp" /> TCP</label>
            <label class="row"><input type="checkbox" v-model="proto.udp" /> UDP</label>
            <label class="row"><input type="checkbox" v-model="proto.icmp" /> ICMP</label>
            <label class="row"><input type="checkbox" v-model="proto.icmp6" /> ICMPv6</label>
          </div>
          <p class="hint">Vide = pas de restriction de protocole.</p>
        </div>
      </section>

      <!-- Adresses IP / Ports -->
      <section class="grid two">
        <div class="card">
          <h3>Adresses IP</h3>
          <div class="field">
            <label>Inclure hôte</label>
            <input v-model="ip.includeHost" placeholder="ex: 192.168.1.42" />
          </div>
          <div class="field">
            <label>Exclure hôte</label>
            <input v-model="ip.excludeHost" placeholder="ex: 10.0.0.1" />
          </div>
          <div class="field">
            <label>Inclure réseau</label>
            <input v-model="ip.includeNet" placeholder="ex: 10.0.0.0/8" />
          </div>
          <div class="field">
            <label>Exclure réseau</label>
            <input v-model="ip.excludeNet" placeholder="ex: 192.168.0.0/16" />
          </div>
          <div class="field">
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

        <div class="card">
          <h3>Ports</h3>
          <div class="field">
            <label>Inclure port(s)</label>
            <input v-model="ports.include" placeholder="ex: 80,443,22" />
          </div>
          <div class="field">
            <label>Exclure port(s)</label>
            <input v-model="ports.exclude" placeholder="ex: 25,21" />
          </div>
          <div class="field">
            <label>Plage</label>
            <input v-model="ports.range" placeholder="ex: 10000-20000" />
          </div>
          <div class="field">
            <label>Direction</label>
            <select v-model="ports.direction">
              <option value="any">src ou dst</option>
              <option value="src">src</option>
              <option value="dst">dst</option>
            </select>
          </div>
          <p class="hint">S'applique principalement à TCP/UDP.</p>
          <div class="errors" v-if="portErrors.length">
            <span v-for="e in portErrors" :key="e">• {{ e }}</span>
          </div>
        </div>
      </section>

      <!-- BPF brut -->
      <section class="card">
        <h3>Expression BPF brute</h3>
        <input class="raw-input" v-model="advancedRaw" placeholder="ex: tcp[13] & 0x02 != 0 and tcp[13] & 0x10 = 0" />
        <p class="hint">Concaténé à la fin du filtre généré avec <code>and</code>.</p>
      </section>

      <!-- Aperçu / Actions -->
      <section class="card">
        <div class="preview-header">
          <h3>Aperçu</h3>
          <span class="manual-badge" v-if="isManualPreview">édition manuelle</span>
        </div>
        <textarea
          class="preview-box"
          :value="previewText"
          @input="onPreviewInput"
          placeholder="Le filtre BPF généré apparaît ici…"
          rows="3"
        />
        <div class="errors" v-if="globalErrors.length">
          <span v-for="e in globalErrors" :key="e">• {{ e }}</span>
        </div>
        <div class="actions">
          <button class="btn primary" @click="apply" :disabled="!canApply">Appliquer</button>
          <button class="btn ghost" @click="resetAll">Réinitialiser</button>
          <button class="btn ghost sync" v-if="isManualPreview" @click="syncPreview">↺ Sync auto</button>
        </div>
      </section>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useCaptureStore } from '../../../store/capture';

type Dir = 'any' | 'src' | 'dst';

const emit = defineEmits<{ 'update:visible': [value: boolean] }>();
const captureStore = useCaptureStore();

const opt = ref({ vlan: false, onlyIp4: false, excludeIpv6: false, excludeArp: false });
const proto = ref({ tcp: false, udp: false, icmp: false, icmp6: false });
const ip = ref({ includeHost: '', excludeHost: '', includeNet: '', excludeNet: '', direction: 'any' as Dir });
const ports = ref({ include: '', exclude: '', range: '', direction: 'any' as Dir });
const advancedRaw = ref('');
const previewText = ref('');
const isManualPreview = ref(false);

const ipErrors = computed<string[]>(() => {
  const errs: string[] = [];
  const isIp = (s: string) => /^(\d{1,3}\.){3}\d{1,3}$/.test(s);
  const isCidr = (s: string) => /^(\d{1,3}\.){3}\d{1,3}\/\d{1,2}$/.test(s);
  if (ip.value.includeHost && !isIp(ip.value.includeHost)) errs.push(`IP invalide : ${ip.value.includeHost}`);
  if (ip.value.excludeHost && !isIp(ip.value.excludeHost)) errs.push(`IP invalide : ${ip.value.excludeHost}`);
  if (ip.value.includeNet && !isCidr(ip.value.includeNet)) errs.push(`CIDR invalide : ${ip.value.includeNet}`);
  if (ip.value.excludeNet && !isCidr(ip.value.excludeNet)) errs.push(`CIDR invalide : ${ip.value.excludeNet}`);
  return errs;
});

const portErrors = computed<string[]>(() => {
  const errs: string[] = [];
  const isPortList = (s: string) =>
    s.split(',').every(p => /^\s*\d{1,5}\s*$/.test(p) && Number(p.trim()) <= 65535);
  const isRange = (s: string) =>
    /^\s*\d{1,5}\s*-\s*\d{1,5}\s*$/.test(s) &&
    Number(s.split('-')[0]) <= 65535 &&
    Number(s.split('-')[1]) <= 65535;
  if (ports.value.include && !isPortList(ports.value.include)) errs.push('Ports à inclure invalides');
  if (ports.value.exclude && !isPortList(ports.value.exclude)) errs.push('Ports à exclure invalides');
  if (ports.value.range && !isRange(ports.value.range)) errs.push('Plage de ports invalide (ex: 10000-20000)');
  return errs;
});

const globalErrors = computed(() => [...ipErrors.value, ...portErrors.value]);

const autoPreview = computed(() => {
  const c: string[] = [];
  const groupOr = (clauses: string[]) => clauses.length > 1 ? `(${clauses.join(' or ')})` : clauses[0] ?? '';
  const dirPfx = (d: Dir) => d === 'any' ? '' : `${d} `;

  if (opt.value.vlan) c.push('vlan');
  if (opt.value.onlyIp4) c.push('ip');
  if (opt.value.excludeIpv6) c.push('not ip6');
  if (opt.value.excludeArp) c.push('not arp');

  const protos: string[] = [];
  if (proto.value.tcp) protos.push('tcp');
  if (proto.value.udp) protos.push('udp');
  if (proto.value.icmp) protos.push('icmp');
  if (proto.value.icmp6) protos.push('icmp6');
  if (protos.length) c.push(groupOr(protos));

  if (ip.value.includeHost) c.push(`${dirPfx(ip.value.direction)}host ${ip.value.includeHost}`);
  if (ip.value.excludeHost) c.push(`not ${dirPfx(ip.value.direction)}host ${ip.value.excludeHost}`);
  if (ip.value.includeNet) c.push(`${dirPfx(ip.value.direction)}net ${ip.value.includeNet}`);
  if (ip.value.excludeNet) c.push(`not ${dirPfx(ip.value.direction)}net ${ip.value.excludeNet}`);

  const addPorts = (list: string, negate = false) => {
    if (!list.trim()) return;
    const op = negate ? 'not ' : '';
    const parts = list.split(',').map(s => s.trim()).filter(Boolean);
    const clauses = parts.map(p => `${dirPfx(ports.value.direction)}port ${p}`);
    c.push(op + groupOr(clauses));
  };
  addPorts(ports.value.include);
  addPorts(ports.value.exclude, true);
  if (ports.value.range.trim()) c.push(`${dirPfx(ports.value.direction)}portrange ${ports.value.range.trim()}`);

  if (advancedRaw.value.trim()) c.push(advancedRaw.value.trim());

  return c.join(' and ').trim();
});

const canApply = computed(() => previewText.value.trim().length > 0 && globalErrors.value.length === 0);

watch(autoPreview, val => {
  if (!isManualPreview.value) previewText.value = val;
}, { immediate: true });

function onPreviewInput(event: Event) {
  previewText.value = (event.target as HTMLTextAreaElement).value;
  isManualPreview.value = true;
}

function syncPreview() {
  isManualPreview.value = false;
  previewText.value = autoPreview.value;
}

async function apply() {
  if (!canApply.value) return;
  const filter = previewText.value.trim();
  try {
    await invoke('set_filter', { filter });
    captureStore.setActiveFilter(filter);
    emit('update:visible', false);
  } catch (e) {
    console.error('set_filter failed:', e);
  }
}

async function resetAll() {
  opt.value = { vlan: false, onlyIp4: false, excludeIpv6: false, excludeArp: false };
  proto.value = { tcp: false, udp: false, icmp: false, icmp6: false };
  ip.value = { includeHost: '', excludeHost: '', includeNet: '', excludeNet: '', direction: 'any' };
  ports.value = { include: '', exclude: '', range: '', direction: 'any' };
  advancedRaw.value = '';
  isManualPreview.value = false;
  previewText.value = '';
  try {
    await invoke('set_filter', { filter: '' });
    captureStore.setActiveFilter('');
  } catch (e) {
    console.error('clear filter failed:', e);
  }
}

function preset(name: string) {
  resetAll();
  switch (name) {
    case 'ipv4':
      opt.value.onlyIp4 = true;
      break;
    case 'web':
      opt.value.onlyIp4 = true;
      proto.value.tcp = true;
      ports.value.include = '80,443';
      break;
    case 'dns':
      opt.value.onlyIp4 = true;
      proto.value.udp = true;
      proto.value.tcp = true;
      ports.value.include = '53';
      break;
    case 'ntp':
      opt.value.onlyIp4 = true;
      proto.value.udp = true;
      ports.value.include = '123';
      break;
    case 'syn':
      opt.value.onlyIp4 = true;
      proto.value.tcp = true;
      advancedRaw.value = 'tcp[13] & 0x02 != 0 and tcp[13] & 0x10 = 0';
      break;
    case 'no-arp-ipv6':
      opt.value.excludeArp = true;
      opt.value.excludeIpv6 = true;
      break;
  }
}
</script>

<style scoped>
.filter-overlay {
  position: fixed;
  inset: 40px 0 0 0;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: 20px 16px;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.5);
  cursor: default;
}

.filter-panel {
  width: min(1100px, 96vw);
  max-height: calc(100vh - 80px);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: #1e1e2e;
  border: 1px solid #2d2d50;
  border-radius: 14px;
  padding: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  cursor: auto;
}

/* Active filter bar */
.active-filter-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(74, 124, 255, 0.08);
  border: 1px solid rgba(74, 124, 255, 0.3);
  border-radius: 8px;
}
.active-filter-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #4a7cff;
  flex-shrink: 0;
}
.active-filter-expr {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
  color: #8ab4ff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.active-filter-clear {
  font-size: 12px;
  flex-shrink: 0;
  color: #6060a0;
  padding: 3px 10px;
}
.active-filter-clear:hover {
  color: #ff6060;
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.panel-header h2 {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: #c0c0e0;
}
.close-btn {
  background: transparent;
  border: none;
  color: #6060a0;
  font-size: 16px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  line-height: 1;
  transition: background 0.15s, color 0.15s;
}
.close-btn:hover {
  background: #2d2d50;
  color: #e0e0f0;
}

/* Grid */
.grid.two {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
@media (max-width: 720px) {
  .grid.two { grid-template-columns: 1fr; }
}

/* Card */
.card {
  background: #252535;
  border: 1px solid #2d2d50;
  border-radius: 10px;
  padding: 14px;
}
.card h3 {
  margin: 0 0 12px 0;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: #7070a0;
  font-weight: 600;
}

/* Fields */
.field {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.field label {
  min-width: 110px;
  font-size: 13px;
  color: #8888b0;
}
.field input,
.field select {
  flex: 1;
  background: #12121e;
  color: #d0d0f0;
  border: 1px solid #2d2d50;
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 13px;
  outline: none;
  transition: border-color 0.15s;
}
.field input:focus,
.field select:focus {
  border-color: #4a7cff;
}
.field input::placeholder {
  color: #454570;
}
.field select option {
  background: #1e1e2e;
}

/* Checkboxes */
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 13px;
  color: #b0b0d8;
  cursor: pointer;
}
.row input[type="checkbox"] {
  accent-color: #4a7cff;
  width: 14px;
  height: 14px;
  cursor: pointer;
}
.row code {
  font-size: 11px;
  color: #6080c0;
  background: #1a1a2e;
  padding: 1px 4px;
  border-radius: 3px;
}

.proto-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2px;
}

/* Hint */
.hint {
  font-size: 12px;
  color: #50507a;
  margin: 6px 0 0 0;
}
.hint code {
  color: #5070b0;
}

/* Raw BPF input */
.raw-input {
  width: 100%;
  background: #12121e;
  color: #d0d0f0;
  border: 1px solid #2d2d50;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 13px;
  font-family: monospace;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.15s;
}
.raw-input:focus {
  border-color: #4a7cff;
}
.raw-input::placeholder {
  color: #454570;
}

/* Preview */
.preview-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.preview-header h3 {
  margin: 0;
}
.manual-badge {
  font-size: 11px;
  color: #f0a030;
  background: rgba(240, 160, 48, 0.1);
  border: 1px solid rgba(240, 160, 48, 0.25);
  border-radius: 4px;
  padding: 2px 7px;
}
.preview-box {
  width: 100%;
  background: #12121e;
  color: #d0d0f0;
  border: 1px solid #2d2d50;
  border-radius: 8px;
  padding: 10px;
  font-family: monospace;
  font-size: 13px;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.15s;
}
.preview-box:focus {
  border-color: #4a7cff;
}
.preview-box::placeholder {
  color: #454570;
}

/* Actions */
.actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  flex-wrap: wrap;
}
.btn {
  background: #2a2a45;
  color: #a0a0cc;
  border: 1px solid #2d2d50;
  border-radius: 8px;
  padding: 7px 14px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}
.btn:hover {
  background: #32325a;
  border-color: #404070;
  color: #c0c0e8;
}
.btn.primary {
  background: #3a6eee;
  border-color: #3a6eee;
  color: #fff;
  font-weight: 600;
}
.btn.primary:hover {
  background: #4a7eff;
  border-color: #4a7eff;
}
.btn.primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.btn.ghost {
  background: transparent;
  color: #60609a;
  border-color: transparent;
}
.btn.ghost:hover {
  background: #1e1e38;
  color: #9090c0;
  border-color: #2d2d50;
}
.btn.sync {
  font-family: inherit;
}

/* Chips */
.chips {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.chip {
  background: #1e1e30;
  color: #8080b0;
  border: 1px solid #2d2d50;
  border-radius: 999px;
  padding: 5px 13px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}
.chip:hover {
  background: #2a2a50;
  border-color: #4a7cff;
  color: #c0c0f0;
}

/* Errors */
.errors {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.errors span {
  color: #ff6060;
  font-size: 12px;
}
</style>
