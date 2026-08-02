<!--
  Panneau de gestion des labels (#157) : table éditable (recherche, ajout,
  édition inline, suppression) sur le store backend — source de vérité
  unique —, statut observé/dormant par ligne, arbitrage des conflits et
  import/export rattachés. Les mutations retournent les updates graphe,
  appliquées via le store (pas de dépendance au channel de capture).
-->
<template>
  <div
    class="container"
    role="dialog"
    aria-modal="true"
    aria-label="Gestion des labels"
    @keydown.esc="close"
  >
    <div class="center-container" ref="panel" tabindex="-1">
      <button type="button" class="btn image-btn cross" @click.prevent="close" title="Fermer (Échap)">❌</button>

      <h1 class="dialog-title">Gestion des labels</h1>
      <p class="text subtitle">
        {{ rows.length }} label(s) — <span class="observed-dot">●</span> observé dans la capture courante,
        <span class="dormant-dot">●</span> dormant. La table est l'état des labels : supprimer une ligne
        désétiquette l'équipement.
      </p>

      <div class="toolbar">
        <input
          ref="search"
          v-model="search"
          class="input search"
          type="search"
          placeholder="Rechercher (MAC, IP, label)…"
          aria-label="Rechercher un label"
        />
        <button type="button"
          v-if="conflicts.length > 0"
          class="btn btn-conflicts"
          @click="showArbitration = true"
        >
          ⚖️ Arbitrer {{ conflicts.length }} conflit(s)
        </button>
        <button type="button" class="btn" @click="importFile" :disabled="busy">📥 Importer</button>
        <button type="button" class="btn" @click="exportFile" :disabled="busy || rows.length === 0">💾 Exporter</button>
      </div>

      <!-- Ajout d'une ligne -->
      <form class="add-row" @submit.prevent="addRow">
        <input v-model="draft.mac" class="input mono" placeholder="MAC (aa:bb:cc:dd:ee:ff)" aria-label="MAC" />
        <input v-model="draft.ip" class="input mono" placeholder="IP (v4 ou v6)" aria-label="IP" />
        <input v-model="draft.label" class="input" placeholder="Label" aria-label="Label" required />
        <button class="btn btn-open" type="submit" :disabled="busy">Ajouter</button>
      </form>

      <p v-if="filteredRows.length === 0" class="text empty">
        {{ rows.length === 0 ? "Aucun label. Ajoute une ligne ou importe un fichier." : "Aucun label ne correspond à la recherche." }}
      </p>

      <div v-else class="data-table" role="table" aria-label="Labels">
        <div class="table-head" role="row">
          <span role="columnheader" class="col-status" title="Observé / dormant">•</span>
          <span role="columnheader">MAC</span>
          <span role="columnheader">IP</span>
          <span role="columnheader">Label</span>
          <span role="columnheader" class="col-actions"></span>
        </div>
        <div class="table-body">
          <div
            v-for="row in filteredRows"
            :key="rowKey(row)"
            class="table-row"
            role="row"
          >
            <template v-if="editingKey === rowKey(row)">
              <span class="col-status" :class="row.observed ? 'observed-dot' : 'dormant-dot'">●</span>
              <input v-model="edit.mac" class="input mono" aria-label="MAC" @keydown.enter.prevent="saveEdit(row)" />
              <input v-model="edit.ip" class="input mono" aria-label="IP" @keydown.enter.prevent="saveEdit(row)" />
              <input v-model="edit.label" class="input" aria-label="Label" @keydown.enter.prevent="saveEdit(row)" />
              <span class="col-actions">
                <button type="button" class="btn image-btn" @click="saveEdit(row)" :disabled="busy" title="Enregistrer (Entrée)">✅</button>
                <button type="button" class="btn image-btn" @click="editingKey = null" title="Annuler">↩️</button>
              </span>
            </template>
            <template v-else>
              <span
                class="col-status"
                :class="row.observed ? 'observed-dot' : 'dormant-dot'"
                :title="row.observed ? 'Observé dans la capture courante' : 'Dormant (aucun endpoint correspondant)'"
              >●</span>
              <span class="text mono">{{ row.mac || "-" }}</span>
              <span class="text mono">{{ row.ip || "-" }}</span>
              <span class="text">{{ row.label || "-" }}</span>
              <span class="col-actions">
                <button type="button" class="btn image-btn" @click="startEdit(row)" title="Éditer">✏️</button>
                <button type="button" class="btn image-btn" @click="deleteRow(row)" :disabled="busy" title="Supprimer">🗑️</button>
              </span>
            </template>
          </div>
        </div>
      </div>
    </div>

    <ArbitrationDialog
      v-if="showArbitration"
      :conflicts="conflicts"
      @close="showArbitration = false"
      @resolved="onConflictsResolved"
    />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { Channel, invoke } from '@tauri-apps/api/core';
import { info, warn, error } from '@tauri-apps/plugin-log';
import { ask, open, save } from '@tauri-apps/plugin-dialog';

import ArbitrationDialog from './ArbitrationDialog.vue';
import { displayCaptureError } from '../../../errors/capture';
import { getCurrentDate } from '../../../utils/time';
import { EXPECTED_PROTOCOL_VERSION, useCaptureStore } from '../../../store/capture';
import type { CaptureEvent, GraphUpdate } from '../../../types/capture';
import type { LabelConflictReport } from '../../../types/labels';

/** Ligne du store enrichie du statut observé/dormant (backend). */
type LabelRowStatus = { mac: string; ip: string; label: string; observed: boolean };

export default defineComponent({
  name: 'LabelsPanel',
  components: { ArbitrationDialog },
  emits: ['update:visible'],
  data() {
    return {
      rows: [] as LabelRowStatus[],
      conflicts: [] as LabelConflictReport[],
      search: '',
      draft: { mac: '', ip: '', label: '' },
      edit: { mac: '', ip: '', label: '' },
      editingKey: null as string | null,
      showArbitration: false,
      busy: false,
    };
  },
  computed: {
    filteredRows(): LabelRowStatus[] {
      const needle = this.search.trim().toLowerCase();
      if (!needle) return this.rows;
      return this.rows.filter((r) =>
        [r.mac, r.ip, r.label].some((v) => v.toLowerCase().includes(needle)),
      );
    },
  },
  async mounted() {
    await this.load();
    (this.$refs.search as HTMLInputElement | undefined)?.focus();
  },
  methods: {
    rowKey(row: LabelRowStatus): string {
      return `${row.mac}|${row.ip}`;
    },
    close() {
      this.$emit('update:visible', false);
    },
    async load() {
      try {
        this.rows = await invoke<LabelRowStatus[]>('get_labels_with_status');
        this.conflicts = await invoke<LabelConflictReport[]>('get_label_conflicts');
      } catch (err) {
        error(`Chargement des labels impossible: ${err}`);
        displayCaptureError(err);
      }
    },
    /** Applique les updates graphe retournées par une mutation puis recharge. */
    async afterMutation(updates: GraphUpdate[]) {
      useCaptureStore().applyGraphUpdates(updates);
      await this.load();
    },
    async addRow() {
      if (this.busy) return;
      this.busy = true;
      try {
        const updates = await invoke<GraphUpdate[]>('add_label', {
          mac: this.draft.mac,
          ip: this.draft.ip,
          label: this.draft.label,
        });
        this.draft = { mac: '', ip: '', label: '' };
        await this.afterMutation(updates);
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.busy = false;
      }
    },
    startEdit(row: LabelRowStatus) {
      this.editingKey = this.rowKey(row);
      this.edit = { mac: row.mac, ip: row.ip, label: row.label };
    },
    async saveEdit(row: LabelRowStatus) {
      if (this.busy) return;
      this.busy = true;
      try {
        const updates = await invoke<GraphUpdate[]>('update_label', {
          old_mac: row.mac,
          old_ip: row.ip,
          mac: this.edit.mac,
          ip: this.edit.ip,
          label: this.edit.label,
        });
        this.editingKey = null;
        await this.afterMutation(updates);
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.busy = false;
      }
    },
    async deleteRow(row: LabelRowStatus) {
      const confirmed = await ask(
        `Supprimer le label « ${row.label} » (${row.mac || '-'} / ${row.ip || '-'}) ?\nL'équipement sera désétiqueté.`,
        { title: 'Supprimer le label', kind: 'warning' },
      );
      if (!confirmed || this.busy) return;
      this.busy = true;
      try {
        const updates = await invoke<GraphUpdate[]>('delete_label', {
          mac: row.mac,
          ip: row.ip,
        });
        await this.afterMutation(updates);
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.busy = false;
      }
    },
    async onConflictsResolved() {
      this.showArbitration = false;
      await this.load();
    },
    async importFile() {
      if (this.busy) return;
      const path = await open({
        multiple: false,
        filters: [{ name: 'Labels CSV', extensions: ['csv'] }],
        title: 'Importer un fichier de labels',
      });
      if (typeof path !== 'string') return;
      this.busy = true;
      try {
        // Channel local : les updates graphe de l'import sont routées vers
        // les abonnés du store sans toucher au channel de capture principal.
        const onEvent = new Channel<CaptureEvent>();
        onEvent.onmessage = (msg) => {
          if (msg.event === 'started' && msg.data.protocolVersion !== EXPECTED_PROTOCOL_VERSION) {
            warn(
              `[LabelsPanel] version du contrat IPC inattendue : reçu ${msg.data.protocolVersion}, attendu ${EXPECTED_PROTOCOL_VERSION}`
            );
          }
          if (msg.event === 'graph') {
            useCaptureStore().applyGraphUpdates([msg.data.update]);
          }
        };
        await invoke('import_label_file', { incomingFilePath: path, onEvent });
        info('Labels importés depuis le panneau de gestion');
        await this.load();
        if (this.conflicts.length > 0) this.showArbitration = true;
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.busy = false;
      }
    },
    async exportFile() {
      if (this.busy) return;
      const path = await save({
        filters: [{ name: '.csv', extensions: ['csv'] }],
        title: 'Exporter les labels',
        defaultPath: getCurrentDate() + '_labels.csv',
      });
      if (!path) return;
      this.busy = true;
      try {
        await invoke('export_label_file', { path });
        info('Labels exportés');
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.busy = false;
      }
    },
  },
});
</script>

<style scoped>
.container {
  position: fixed;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.7);
  z-index: 1000;
}
.center-container {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: min(880px, 92vw);
  max-height: 84vh;
  padding: 20px 24px;
  border-radius: 10px;
  background-color: #1e1e2e;
  border: 1px solid #444;
  outline: none;
}
.cross {
  position: absolute;
  top: 10px;
  right: 10px;
}
.dialog-title {
  margin: 0;
  color: #eee;
  font-size: 1.2rem;
}
.text {
  color: #ccc;
}
.subtitle {
  margin: 0;
  font-size: 0.85rem;
}
.mono {
  font-family: monospace;
}
.observed-dot {
  color: #4caf50;
}
.dormant-dot {
  color: #777;
}
.toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
}
.input {
  background: #2a2a3a;
  border: 1px solid #555;
  border-radius: 6px;
  color: #eee;
  padding: 6px 8px;
  font-size: 0.85rem;
  min-width: 0;
}
.search {
  flex: 1;
}
.btn {
  background: #2a2a3a;
  border: 1px solid #555;
  border-radius: 6px;
  color: #eee;
  padding: 6px 10px;
  cursor: pointer;
  font-size: 0.85rem;
}
.btn:hover:not(:disabled) {
  background: #3a3a4e;
}
.btn:disabled {
  opacity: 0.5;
  cursor: default;
}
.btn-open {
  border-color: #4caf50;
}
.btn-conflicts {
  border-color: #e6a23c;
  color: #e6a23c;
}
.image-btn {
  padding: 2px 6px;
  border: none;
  background: transparent;
}
.add-row {
  display: grid;
  grid-template-columns: 1.2fr 1.2fr 1.5fr auto;
  gap: 8px;
}
.empty {
  margin: 12px 0;
  font-style: italic;
}
.data-table {
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid #444;
  border-radius: 6px;
  overflow: hidden;
}
.table-head,
.table-row {
  display: grid;
  grid-template-columns: 24px 1.2fr 1.2fr 1.5fr 76px;
  gap: 8px;
  align-items: center;
  padding: 6px 10px;
}
.table-head {
  background: #2a2a3a;
  color: #aaa;
  font-size: 0.8rem;
  position: sticky;
  top: 0;
}
.table-body {
  overflow-y: auto;
}
.table-row {
  border-top: 1px solid #333;
}
.table-row:hover {
  background: #26263a;
}
.col-status {
  text-align: center;
}
.col-actions {
  display: flex;
  gap: 4px;
  justify-content: flex-end;
}
</style>
