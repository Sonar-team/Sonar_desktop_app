<!--
  Panneau modal des labels appliqués à la matrice (MAC, IP, label) avec
  recherche plein texte. Chargé depuis le backend à l'ouverture.
  Événement : close.
-->
<script lang="ts">
import { defineComponent } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { error } from "@tauri-apps/plugin-log"

export default defineComponent({
  name: "MatrixLabelsPanel",
  emits: ["close"],

  data() {
    return {
      matrixLabels: [] as [string, string, string][],
      search: "" as string,
    }
  },

  computed: {
    filteredLabels(): [string, string, string][] {
      const q = this.search.trim().toLowerCase()
      if (!q) return this.matrixLabels
      return this.matrixLabels.filter((row) =>
        row.some((field) => field.toLowerCase().includes(q))
      )
    },
  },

  async mounted() {
    try {
      this.matrixLabels = await invoke<[string, string, string][]>("get_matrix_labels")
    } catch (e) {
      error(`Erreur get_matrix_labels: ${e}`)
      this.matrixLabels = []
    }
    (this.$refs.search as HTMLInputElement | undefined)?.focus()
  },
})
</script>

<template>
  <div
    class="labels-overlay"
    role="dialog"
    aria-modal="true"
    aria-label="Labels appliqués à la matrice"
    @click.self="$emit('close')"
    @keydown.esc="$emit('close')"
  >
    <div class="labels-modal">
      <div class="labels-modal-header">
        <h3>Labels appliqués à la matrice ({{ matrixLabels.length }})</h3>
        <button type="button" class="labels-close" @click="$emit('close')" title="Fermer">✕</button>
      </div>

      <input
        ref="search"
        v-model="search"
        class="labels-search"
        type="text"
        placeholder="Rechercher (MAC, IP, label)…"
        aria-label="Rechercher un label"
      />

      <p v-if="matrixLabels.length === 0" class="labels-empty">
        Aucun label appliqué à la matrice pour le moment.
      </p>

      <div v-else class="labels-table">
        <div class="labels-row labels-head">
          <div class="labels-col">Adresse MAC</div>
          <div class="labels-col">Adresse IP</div>
          <div class="labels-col">Label</div>
        </div>
        <div class="labels-body">
          <div
            v-for="([mac, ip, label], index) in filteredLabels"
            :key="index"
            class="labels-row"
          >
            <div class="labels-col">{{ mac || "-" }}</div>
            <div class="labels-col">{{ ip || "-" }}</div>
            <div class="labels-col">{{ label || "-" }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.labels-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 50;
}
.labels-modal {
  display: flex;
  flex-direction: column;
  width: 90%;
  max-width: 640px;
  max-height: 80%;
  background: #1e1e2e;
  border: 1px solid #444;
  border-radius: 8px;
  padding: 1rem 1.25rem;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  color: whitesmoke;
}
.labels-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}
.labels-modal-header h3 { margin: 0; font-size: 1.1rem; }
.labels-close {
  background: transparent;
  color: #bbb;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  line-height: 1;
}
.labels-close:hover { color: #fff; }
.labels-search {
  width: 100%;
  box-sizing: border-box;
  margin-bottom: 0.75rem;
  padding: 0.4em 0.7em;
  border-radius: 6px;
  border: 1px solid #444;
  background: #2d3748;
  color: whitesmoke;
}
.labels-empty { color: rgba(245, 245, 245, 0.6); text-align: center; padding: 1rem 0; }
.labels-table {
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
}
.labels-body { overflow-y: auto; }
.labels-row {
  display: flex;
  padding: 0.35rem 0.5rem;
  border-bottom: 1px solid #2a2a3a;
}
.labels-row:last-child { border-bottom: none; }
.labels-head {
  font-weight: 600;
  background: #262636;
  position: sticky;
  top: 0;
}
.labels-col {
  flex: 1;
  padding: 0 0.4rem;
  word-break: break-all;
  font-family: monospace;
  font-size: 0.85rem;
}
</style>
