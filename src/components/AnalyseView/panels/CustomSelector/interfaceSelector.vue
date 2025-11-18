<template>
  <div class="interface-selector">
    <label class="selector-label">Interface réseau</label>
    <div class="selector-container">
      <select 
        v-model="selectedInterface" 
        @change="onInterfaceChange"
        class="interface-select"
        :disabled="netInterfaces.length === 0"
      >
        <option value="" disabled>Choisir une interface...</option>
        <option 
          v-for="netInterface in netInterfaces" 
          :key="netInterface.name" 
          :value="netInterface"
        >
          {{ formatInterfaceDisplay(netInterface) }}
        </option>
      </select>
      
      <div v-if="selectedInterface" class="interface-details">
        <div class="detail-item" v-if="selectedInterface.desc">
          <span class="detail-label">Description:</span>
          <span class="detail-value">{{ selectedInterface.desc }}</span>
        </div>
        
        <div class="detail-item" v-if="selectedInterface.addresses.length > 0">
          <span class="detail-label">Adresses:</span>
          <div class="addresses-list">
            <span 
              v-for="addr in selectedInterface.addresses" 
              :key="addr.addr" 
              class="address-item"
            >
              {{ addr.addr }}
            </span>
          </div>
        </div>
        
        <div class="detail-item">
          <span class="detail-label">Statut:</span>
          <span class="detail-value status" :class="getStatusClass(selectedInterface.flags.connection_status)">
            {{ getStatusText(selectedInterface.flags.connection_status) }}
          </span>
        </div>
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

const formatInterfaceDisplay = (netInterface: NetDevice): string => {
  if (netInterface.desc) {
    return `${netInterface.name} - ${netInterface.desc}`
  }
  return netInterface.name
}

const getStatusText = (status: string): string => {
  switch (status) {
    case 'Connected':
      return 'Connecté'
    case 'Disconnected':
      return 'Déconnecté'
    case 'Unknown':
      return 'Inconnu'
    case 'NotApplicable':
      return 'Non applicable'
    default:
      return status
  }
}

const getStatusClass = (status: string): string => {
  switch (status) {
    case 'Connected':
      return 'status-connected'
    case 'Disconnected':
      return 'status-disconnected'
    case 'Unknown':
      return 'status-unknown'
    case 'NotApplicable':
      return 'status-not-applicable'
    default:
      return ''
  }
}

const onInterfaceChange = () => {
  emit('update:modelValue', selectedInterface.value)
  if (selectedInterface.value) {
    emit('interface-selected', selectedInterface.value)
  }
}

// Synchroniser avec le modelValue externe
watch(() => props.modelValue, (newValue) => {
  selectedInterface.value = newValue
})

// Sélectionner automatiquement la première interface disponible si aucune n'est sélectionnée
watch(() => props.netInterfaces, (newInterfaces) => {
  if (newInterfaces.length > 0 && !selectedInterface.value) {
    // Chercher une interface connectée en priorité
    const connectedInterface = newInterfaces.find(
      iface => iface.flags.connection_status === 'Connected'
    )
    
    if (connectedInterface) {
      selectedInterface.value = connectedInterface
    } else {
      // Sinon prendre la première
      selectedInterface.value = newInterfaces[0]
    }
    
    onInterfaceChange()
  }
}, { immediate: true })
</script>

<style scoped>
.interface-selector {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.selector-label {
  font-weight: 600;
  color: #374151;
  font-size: 0.875rem;
}

.selector-container {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.interface-select {
  padding: 0.5rem;
  border: 1px solid #d1d5db;
  border-radius: 0.375rem;
  background-color: white;
  font-size: 0.875rem;
  color: #374151;
  cursor: pointer;
  transition: border-color 0.2s;
}

.interface-select:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.interface-select:disabled {
  background-color: #f9fafb;
  color: #9ca3af;
  cursor: not-allowed;
}

.interface-details {
  background-color: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 0.375rem;
  padding: 0.75rem;
  font-size: 0.875rem;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  margin-bottom: 0.5rem;
}

.detail-item:last-child {
  margin-bottom: 0;
}

.detail-label {
  font-weight: 600;
  color: #6b7280;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

.detail-value {
  color: #374151;
}

.addresses-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.address-item {
  background-color: #e5e7eb;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-family: monospace;
  font-size: 0.75rem;
  color: #374151;
}

.status {
  font-weight: 600;
  text-transform: capitalize;
}

.status-connected {
  color: #059669;
}

.status-disconnected {
  color: #dc2626;
}

.status-unknown {
  color: #6b7280;
}

.status-not-applicable {
  color: #6b7280;
}
</style>