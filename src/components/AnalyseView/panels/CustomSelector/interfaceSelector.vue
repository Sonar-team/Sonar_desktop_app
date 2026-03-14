<template>
  <div class="interface-selector">
    <label class="field-label">Interface réseau</label>

    <select
      v-model="selectedInterface"
      @change="onInterfaceChange"
      class="field-select"
      :disabled="netInterfaces.length === 0"
    >
      <option value="" disabled>Choisir une interface…</option>
      <option
        v-for="iface in netInterfaces"
        :key="iface.name"
        :value="iface"
      >
        {{ formatInterfaceDisplay(iface) }}
      </option>
    </select>

    <div v-if="selectedInterface" class="iface-details">
      <div class="detail-row" v-if="selectedInterface.desc">
        <span class="detail-label">Desc</span>
        <span class="detail-value">{{ selectedInterface.desc }}</span>
      </div>
      <div class="detail-row" v-if="selectedInterface.addresses?.length">
        <span class="detail-label">IP</span>
        <div class="addr-list">
          <span
            v-for="addr in selectedInterface.addresses"
            :key="addr.addr"
            class="addr-badge"
          >{{ addr.addr }}</span>
        </div>
      </div>
      <div class="detail-row row-status">
        <span class="detail-label">Statut</span>
        <span class="status-dot" :class="getStatusClass(selectedInterface.flags.connection_status)"></span>
        <span class="detail-value">{{ getStatusText(selectedInterface.flags.connection_status) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { NetDevice } from '../../../../types/NetDevice'

interface Props {
  netInterfaces: NetDevice[]
  modelValue?: NetDevice | null
}
interface Emits {
  (e: 'update:modelValue', value: NetDevice | null): void
  (e: 'interface-selected', selectedInterface: NetDevice): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const selectedInterface = ref<NetDevice | null>(props.modelValue || null)

const formatInterfaceDisplay = (iface: NetDevice) =>
  iface.desc ? `${iface.name} — ${iface.desc}` : iface.name

const getStatusText = (status: string) => {
  switch (status) {
    case 'Connected': return 'Connecté'
    case 'Disconnected': return 'Déconnecté'
    case 'NotApplicable': return 'N/A'
    default: return 'Inconnu'
  }
}
const getStatusClass = (status: string) => {
  switch (status) {
    case 'Connected': return 'connected'
    case 'Disconnected': return 'disconnected'
    default: return 'unknown'
  }
}
const onInterfaceChange = () => {
  emit('update:modelValue', selectedInterface.value)
  if (selectedInterface.value) emit('interface-selected', selectedInterface.value)
}

watch(() => props.modelValue, v => { selectedInterface.value = v ?? null })
watch(() => props.netInterfaces, ifaces => {
  if (ifaces.length > 0 && !selectedInterface.value) {
    selectedInterface.value = ifaces.find(i => i.flags.connection_status === 'Connected') ?? ifaces[0]
    onInterfaceChange()
  }
}, { immediate: true })
</script>

<style scoped>
.interface-selector {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  font-weight: 500;
  color: #707088;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.field-select {
  width: 100%;
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 4px;
  color: #b0b0c8;
  font-size: 13px;
  padding: 7px 32px 7px 10px;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%23686884' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round' fill='none'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  transition: border-color 0.15s, color 0.15s;
}
.field-select:focus {
  outline: none;
  border-color: #5a6a80;
  color: #d0d0e0;
}
.field-select:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.field-select option {
  background: #363648;
  color: #b0b0c8;
}

.iface-details {
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 4px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 11px;
  min-width: 0;
}
.detail-row.row-status {
  align-items: center;
}
.detail-label {
  color: #585872;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  width: 36px;
  flex-shrink: 0;
  padding-top: 1px;
}
.detail-value {
  color: #808095;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex: 1;
}

.addr-list { display: flex; flex-direction: column; gap: 3px; min-width: 0; }
.addr-badge {
  background: #303042;
  border: 1px solid #3a3a4e;
  border-radius: 3px;
  padding: 1px 6px;
  font-family: monospace;
  font-size: 11px;
  color: #5a7a8a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.status-dot {
  display: block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
  align-self: center;
}
.status-dot.connected {
  background: #4a8a5a;
  box-shadow: 0 0 0 0 rgba(74, 138, 90, 0.5);
  animation: dot-pulse 2.4s ease-out infinite;
}
.status-dot.disconnected { background: #6a3a3a; }
.status-dot.unknown      { background: #585872; }

@keyframes dot-pulse {
  0%   { box-shadow: 0 0 0 0   rgba(74, 138, 90, 0.5); }
  60%  { box-shadow: 0 0 0 5px rgba(74, 138, 90, 0); }
  100% { box-shadow: 0 0 0 0   rgba(74, 138, 90, 0); }
}
</style>
