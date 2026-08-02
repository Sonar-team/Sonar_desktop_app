<!--
  Panneau d'import (modal) : labels CSV en mode « csv », captures PCAP et
  matrices CSV en mode « pcap », avec drag & drop, table des labels
  importés et accès à l'arbitrage des conflits.
  Props : mode ('csv' | 'pcap'). Événement principal : update:visible.
-->
<template>
  <div
    class="container"
    role="dialog"
    aria-modal="true"
    :aria-label="mode === 'csv' ? 'Import de labels' : 'Import de captures ou de matrices'"
    @keydown.esc="closeOnEscape"
  >
    <div class="center-container" :class="{ 'drag-over': isDragOver }" ref="panel" tabindex="-1">
      <!-- Retour visuel de drag & drop -->
      <div class="drop-hint" v-if="isDragOver">
        <div class="drop-hint-inner">
          <span class="drop-icon">📥</span>
          <span class="text">{{ dropHint }}</span>
        </div>
      </div>

      <ConflictDialog v-if="showConflictDialog"
      :same_ip_diff_mac="sameIpDiffMac"
      :same_ip_diff_label="sameIpDiffLabel"
      :invalid_mac="invalidMac"
      :invalid_ip="invalidIp"
      :invalid_lines="invalidLines"
      :resolved="conflictsResolved"
      @showConflictDialog="showConflictDialog = false"/>

      <ArbitrationDialog v-if="showArbitration"
      :conflicts="arbitrationConflicts"
      @close="showArbitration = false"
      @resolved="onArbitrationResolved"/>

        <!-- Overlay de chargement -->
      <div class="overlay" v-if="isConverting">
        <div class="spinner"></div>
        <p class="overlay-text">Conversion en cours…</p>
        <div v-if="importProgress" class="progress-group">
          <progress
            class="progress-track"
            :value="progressPercent"
            max="100"
            aria-label="Progression de l'import"
          ></progress>
          <p class="progress-label">
            Fichier {{ importProgress.fileIndex }}/{{ importProgress.filesTotal }} ·
            {{ progressFileName }} ·
            {{ importProgress.current.toLocaleString() }}/{{ importProgress.total.toLocaleString() }}
          </p>
        </div>
        <button type="button"
          v-if="cancellableImport"
          class="btn btn-cancel-import"
          @click="cancelImport"
          :disabled="cancelRequested"
        >
          {{ cancelRequested ? 'Annulation…' : "Annuler l'import" }}
        </button>
      </div>
      <button type="button" class="btn image-btn cross" @click.prevent="windowClosed" :disabled="isConverting">❌</button>
      
      <div v-if="mode === 'csv'" class="csv-group">
          <button type="button" class="btn btn-add text" @click="addCsvFiles" :disabled="isConverting">
            Ajouter un fichier
          </button>
          <button type="button" v-if="pendingConflicts > 0" class="btn btn-arbitrate" @click="openArbitration" :disabled="isConverting">
            ⚖️ Arbitrer les conflits ({{ pendingConflicts }})
          </button>
          <p v-show="labelRows.length == 0" class="text">Aucun label importé pour le moment</p>
          <div v-show="labelRows.length > 0" class="table-header-row">
            <h2 class="text table-title">Contenu importé</h2>
            <div class="search-group">
              <input class="input-search" v-model="searchInput" @input="listFilter" aria-label="Rechercher dans les labels importés"/>
              <button type="button" class="btn image-btn icon-lg" @click.prevent="clearLabelStore()" title="Réinitialiser le contenu">🔄</button>
            </div>
          </div>
          <div v-show="labelRows.length > 0" class="data-table">
            <div class="data-table-header">
              <div class="col text">Adresse MAC</div>
              <div class="col text">Adresse IP</div>
              <div class="col text">Label</div>
            </div>
            <div class="separator"></div>
            <div class="data-table-body">
              <div v-for="([mac, ip, label], index) in filteredlabelRows" :key="index" class="data-table-row">
                <div class="col text">{{ mac || "-" }}</div>
                <div class="col text">{{ ip || "-" }}</div>
                <div class="col text">{{ label || "-" }}</div>
              </div>
            </div>
          </div>
          <p class="text hint">Format : <code>mac, ip, label</code> — les colonnes suivantes sont ajoutées au label</p>
      </div>
      
      <div v-else-if="mode === 'pcap'">
        <div class="file-group">
          <button type="button" class="btn btn-add text" @click="addPcapFiles" :disabled="isConverting">
            Ajouter des fichiers .pcap
          </button>
          <button type="button" v-if="packetFiles.length > 0" class="btn btn-clear" @click="clearFiles" :disabled="isConverting">
            Effacer
          </button>
        </div>
        <ul class="file-list" v-if="packetFiles.length > 0">
          <li v-for="(file, index) in packetFiles" :key="index">
            {{ file }}
          </li>
        </ul>
        <button type="button" v-if="packetFiles.length > 0" @click="convertPcap" class="btn btn-open" :disabled="isConverting">
          Ouvrir
        </button>
        <div class="separator matrix-separator"></div>
        <div class="file-group">
          <button type="button" class="btn btn-add text" @click="addMatrixFiles" :disabled="isConverting">
            Ajouter une ou plusieurs matrices (CSV ou XLSX)
          </button>
          <button type="button" v-if="matrixFiles.length > 0" class="btn btn-clear" @click="clearMatrixFiles" :disabled="isConverting">
            Effacer
          </button>
        </div>
        <ul class="file-list" v-if="matrixFiles.length > 0">
          <li v-for="(file, index) in matrixFiles" :key="index">
            {{ file }}
          </li>
        </ul>
        <button type="button" v-if="matrixFiles.length > 0" @click="importMatrixFiles" class="btn btn-open" :disabled="isConverting">
          Ouvrir
        </button>
      </div>
    </div> 
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { message, open } from '@tauri-apps/plugin-dialog';
import { invoke, Channel } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { info } from '@tauri-apps/plugin-log';
import { useCaptureStore } from '../../../store/capture';
import type { CaptureEvent, ImportProgress } from '../../../types/capture';
import { displayCaptureError, isImportCancellation } from '../../../errors/capture';
import { classifyLabelImportError } from '../../../utils/labelImport';
import { LabelConflictReport, LabelImportReport } from '../../../types/labels';
import ConflictDialog from './ConflictDialog.vue'
import ArbitrationDialog from './ArbitrationDialog.vue'
import { appendUniquePaths, MATRIX_EXTENSIONS, PCAP_EXTENSIONS, routeDroppedPaths } from './import/fileTypes';
import { progressFileNameOf, progressPercentOf } from './import/importProgress';
import { filterLabelRows } from './import/labelSearch';
import { finalizeImportState } from './import/importLifecycle';
import { invokePcapImport } from '../../../utils/pcapImportContract';


export default defineComponent({
  name: 'ImportPanel',
  emits: ['update:visible','toggle-pcap', 'toggle-warning', 'showConflictDialog'],
  components: {
    ConflictDialog,
    ArbitrationDialog
  },
  props: {
    mode: {
      type: String,
      default: 'pcap'
    }
  },
  data() {
    return {
      packetFiles: [] as string[],
      matrixFiles: [] as string[],
      labelRows: [] as [string, string, string][],
      filteredlabelRows: [] as [string, string, string][],
      searchInput: "",
      isConverting: false,
      // Annulation : seul l'import PCAP est annulable (les autres imports
      // sont courts et ne consultent pas le jeton backend).
      cancellableImport: false,
      cancelRequested: false,
      showConflictDialog: false,
      conflictsResolved: false,
      showArbitration: false,
      arbitrationConflicts: [] as LabelConflictReport[],
      pendingConflicts: 0,
      isDragOver: false,
      unlistenDrop: null as (() => void) | null,
      sameIpDiffMac: [] as [number, number, string, string, string, string, string][],
      sameIpDiffLabel: [] as [number, number, string, string, string, string, string][],
      invalidMac: [] as [number, string, string][],
      invalidIp: [] as [number, string, string][],
      invalidLines: [] as [number, string][]
    };
  },

  computed: {
    captureStore() {
      return useCaptureStore();
    },
    isRunning(): boolean {
      return this.captureStore.isRunning;
    },
    importProgress(): ImportProgress | null {
      return this.captureStore.importProgress;
    },
    progressPercent(): number {
      return progressPercentOf(this.importProgress);
    },
    progressFileName(): string {
      return progressFileNameOf(this.importProgress);
    },
    dropHint(): string {
      return this.mode === 'csv'
        ? 'Déposez un fichier de labels (.csv)'
        : 'Déposez des captures (.pcap, .pcapng, .cap) ou des matrices (.csv, .xlsx)';
    },
  },

  methods: {
    windowClosed() {
      this.$emit('update:visible', false);
    },

    // Échap ne ferme pas pendant une conversion en cours (même garde que le
    // bouton ❌, désactivé via :disabled="isConverting").
    closeOnEscape() {
      if (this.isConverting) return;
      this.windowClosed();
    },

    // Route les fichiers déposés (drag & drop) selon le mode du panneau et
    // l'extension : labels en mode csv ; captures/matrices en mode pcap.
    // Classification déléguée à ./import/fileTypes.ts.
    async handleDroppedPaths(paths: string[]) {
      if (this.isConverting) return;
      const routing = routeDroppedPaths(paths, this.mode);

      if (routing.kind === 'ignored') {
        info(this.mode === 'csv'
          ? 'drop ignoré : aucun fichier .csv'
          : 'drop ignoré : ni capture réseau ni matrice CSV/XLSX');
        return;
      }

      if (routing.kind === 'csv') {
        useCaptureStore().isImporting = true;
        try {
          await this.importLabelFile(routing.path);
        } finally {
          useCaptureStore().isImporting = false;
        }
        return;
      }

      if (routing.pcaps.length > 0) {
        this.packetFiles = appendUniquePaths(this.packetFiles, routing.pcaps);
      }
      if (routing.matrices.length > 0) {
        this.matrixFiles = appendUniquePaths(this.matrixFiles, routing.matrices);
      }
    },

    addPcapFiles() {
      return this.addFiles('pcap', PCAP_EXTENSIONS);
    },

    addCsvFiles() {
      return this.addFiles('csv', ['csv']);
    },

    addMatrixFiles() {
      return this.addFiles('matrix', MATRIX_EXTENSIONS);
    },

    async addFiles(type: 'pcap' | 'csv' | 'matrix', extensions: string[]) {
        const labels: Record<'pcap' | 'csv' | 'matrix', string> = {
          csv: 'Label File',
          matrix: 'Matrice CSV ou XLSX',
          pcap: 'Capture File',
        };
        const label = labels[type];
        useCaptureStore().isImporting = true;

      try {
        const files = await open({
          // Seul l'import de labels remplace le store : un fichier à la fois.
          multiple: type !== 'csv',
          filters: [{ name: label, extensions: extensions }],
        });

        if (!files) return;


        if (type === 'csv') {
          await this.importLabelFile(files);
        } else {
          // Même déduplication que le drag & drop : resélectionner un fichier
          // déjà listé ne doit pas l'importer deux fois (#161).
          const list = Array.isArray(files) ? files : [files];
          if (type === 'matrix') {
            this.matrixFiles = appendUniquePaths(this.matrixFiles, list);
          } else {
            this.packetFiles = appendUniquePaths(this.packetFiles, list);
          }
        }
      } finally {
        useCaptureStore().isImporting = false;
      }
    },

    clearFiles() {
      this.packetFiles = [];
    },

    clearMatrixFiles() {
      this.matrixFiles = [];
    },

    async convertPcap() {
      if (this.packetFiles.length === 0) return;

      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent);

      info('convert_from_pcap_list : ' + this.packetFiles);

      this.isConverting = true;
      this.cancellableImport = true;
      this.cancelRequested = false;
      this.captureStore.clearImportProgress();
      // Verrou global : bloque les entrées de l'app pendant la conversion.
      useCaptureStore().isImporting = true;

      try {
        await invokePcapImport(invoke, this.packetFiles, onEvent);
        info('réponse invoke');
        this.$emit('update:visible', false);
      } catch (err) {
        // L'annulation demandée par l'opérateur est une issue normale : elle
        // est notifiée comme telle, pas affichée en dialogue d'erreur. Le
        // backend garantit que le relevé courant est intact (import
        // transactionnel).
        if (isImportCancellation(err)) {
          info('import PCAP annulé par l\'opérateur ; relevé courant inchangé');
          await message('Import annulé. Le relevé courant est inchangé.', {
            title: 'Import annulé',
            kind: 'info',
          });
        } else {
          await displayCaptureError(err);
        }
      } finally {
        await finalizeImportState(
          () => {
            this.captureStore.isImporting = false;
            this.isConverting = false;
            this.cancellableImport = false;
            this.cancelRequested = false;
            this.captureStore.clearImportProgress();
          },
          () => this.captureStore.refreshHasData(),
          displayCaptureError,
        );
      }

      this.packetFiles = [];
    },

    /** Demande l'annulation coopérative de l'import PCAP en cours. Le
     *  backend draine le fichier courant sans l'analyser et n'ouvre pas les
     *  suivants ; `convert_from_pcap_list` échoue alors avec l'erreur typée
     *  `import/cancelled`, traitée ci-dessus comme une issue normale. */
    async cancelImport() {
      this.cancelRequested = true;
      try {
        const active = await invoke<boolean>('cancel_import');
        if (!active) {
          // Course avec la fin d'import : rien à annuler, l'issue normale
          // (succès ou erreur) de l'invoke en cours reprend la main.
          info('cancel_import : aucun import actif, demande ignorée');
          this.cancelRequested = false;
        }
      } catch (err) {
        this.cancelRequested = false;
        await displayCaptureError(err);
      }
    },

    async importMatrixFiles() {
      if (this.matrixFiles.length === 0) return;

      info('import_matrix_files: ' + this.matrixFiles.join(', '));

      // Un Channel Tauri est à usage unique (index d'ordre par commande +
      // cleanup à la fin) : on en crée un neuf à chaque invoke. Pendant une
      // capture live on n'écrase pas le channel attaché — le backend passe
      // alors par celui de la capture.
      const onEvent = new Channel<CaptureEvent>();
      if (!this.isRunning) {
        this.captureStore.setChannel(onEvent);
      }

      this.isConverting = true;
      this.captureStore.clearImportProgress();
      // Verrou global : bloque les entrées de l'app pendant l'import.
      useCaptureStore().isImporting = true;
      try {
        await invoke('import_matrix_files', { incomingFilePaths: this.matrixFiles, onEvent });
        info('réponse invoke');
        this.matrixFiles = [];
        this.$emit('update:visible', false);
      } catch (err) {
        await displayCaptureError(err);
      } finally {
        await finalizeImportState(
          () => {
            this.captureStore.isImporting = false;
            this.isConverting = false;
            this.captureStore.clearImportProgress();
          },
          () => this.captureStore.refreshHasData(),
          displayCaptureError,
        );
      }
    },

    listFilter() {
      this.filteredlabelRows = filterLabelRows(this.labelRows, this.searchInput);
    },

    async importLabelFile(path: string) {
      if (path.length === 0) return;

      info('import_label_file: ' + path);

      this.isConverting = true;
      this.invalidIp = [];
      this.invalidMac = [];
      this.sameIpDiffLabel = [];
      this.sameIpDiffMac = [];
      this.conflictsResolved = false;

      // Un Channel Tauri est à usage unique (index d'ordre par commande +
      // cleanup à la fin) : on en crée un neuf à chaque invoke. Pendant une
      // capture live on n'écrase pas le channel attaché — le backend passe
      // alors par celui de la capture.
      const onEvent = new Channel<CaptureEvent>();
      if (!this.isRunning) {
        this.captureStore.setChannel(onEvent);
      }

      try {
        // Les conflits ne bloquent plus : le backend garde le premier label par
        // clé (mac, ip) et renvoie les doublons écartés pour arbitrage.
        const report = await invoke<LabelImportReport>('import_label_file', { incomingFilePath: path, onEvent });
        info(`réponse invoke: ${report.applied} label(s), ${report.conflicts.length} conflit(s) résolu(s)`);

        this.pendingConflicts = report.conflicts.length;
        if (report.conflicts.length > 0) {
          // Les doublons sont appliqués « premier gardé » ; on ouvre le module
          // d'arbitrage pour que l'utilisateur choisisse le bon label.
          this.arbitrationConflicts = report.conflicts;
          this.showArbitration = true;
        }
      } catch (err) {
        this.showLabelFileIssues(err);
      } finally {
        // L'UI est libérée avant le refresh : un échec de get_label_rows ne
        // peut ni bloquer le panneau ni masquer l'erreur d'import (#161).
        await finalizeImportState(
          () => {
            this.isConverting = false;
          },
          async () => {
            this.labelRows = await invoke('get_label_rows');
            this.filteredlabelRows = filterLabelRows(this.labelRows, this.searchInput);
          },
          displayCaptureError,
        );
      }
    },
  

    /** Erreur bloquante d'import de labels : ouvre le dialogue de conflits
     *  avec le détail typé, ou affiche l'erreur générique sinon. */
    showLabelFileIssues(err: unknown) {
      const issues = classifyLabelImportError(err);
      if (!issues) {
        displayCaptureError(err);
        return;
      }
      if (issues.kind === 'invalidMacIpFormat') {
        this.invalidMac = issues.invalidMac;
        this.invalidIp = issues.invalidIp;
      } else if (issues.kind === 'labelLinesConflicts') {
        this.sameIpDiffMac = issues.sameIpDiffMac;
        this.sameIpDiffLabel = issues.sameIpDiffLabel;
      } else {
        this.invalidLines = issues.invalidLines;
      }
      this.showConflictDialog = true;
    },

    async clearLabelStore() {
        try {
          await invoke('clear_label_store');
          info('réponse invoke');
          this.labelRows = await invoke('get_label_rows');
          this.filteredlabelRows = filterLabelRows(this.labelRows, this.searchInput);
        } catch (err) {
          displayCaptureError(err);
        }
      },

    async onArbitrationResolved() {
      this.showArbitration = false;
      this.arbitrationConflicts = [];
      this.pendingConflicts = 0;
      try {
        // Rafraîchit la table de labels après arbitrage.
        this.labelRows = await invoke('get_label_rows');
        this.filteredlabelRows = filterLabelRows(this.labelRows, this.searchInput);
      } catch (err) {
        await displayCaptureError(err);
      }
    },

    async openArbitration() {
      try {
        this.arbitrationConflicts = await invoke<LabelConflictReport[]>('get_label_conflicts');
        if (this.arbitrationConflicts.length > 0) {
          this.showArbitration = true;
        }
      } catch (err) {
        await displayCaptureError(err);
      }
    },

  },

  mounted() {
    (this.$refs.panel as HTMLElement | undefined)?.focus();

    // NB : les imports ne touchent plus à isRunning (réservé à la capture
    // live) — leur cycle de vie passe par isImporting. L'ancien couplage aux
    // événements Started/Finished laissait isRunning bloqué à vrai si la
    // conversion échouait, et le premier Finished d'un import multi-fichiers
    // arrêtait visuellement l'import trop tôt.

    // Conflits de labels en attente d'arbitrage (import précédent).
    invoke<LabelConflictReport[]>('get_label_conflicts')
      .then((c) => { this.pendingConflicts = c.length; })
      .catch(() => { /* ignoré */ });

    // Drag & drop de fichiers sur le panneau (labels, matrices, pcap).
    getCurrentWebviewWindow()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === 'enter' || payload.type === 'over') {
          this.isDragOver = true;
        } else if (payload.type === 'leave') {
          this.isDragOver = false;
        } else if (payload.type === 'drop') {
          this.isDragOver = false;
          this.handleDroppedPaths(payload.paths);
        }
      })
      .then((unlisten) => { this.unlistenDrop = unlisten; })
      .catch((e) => { info(`onDragDropEvent indisponible: ${e}`); });
  },

  beforeUnmount() {
    if (this.unlistenDrop) {
      this.unlistenDrop();
      this.unlistenDrop = null;
    }
  },

})
</script>

<style scoped>

.container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
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
  justify-content: center;
  align-items: stretch;
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 2rem;
  width: 90%;
  max-width: 600px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  border: 2px solid transparent;
  transition: border-color 0.15s;
}

.center-container.drag-over {
  border-color: #2596be;
}

.drop-hint {
  position: absolute;
  inset: 0;
  z-index: 2500;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(30, 30, 46, 0.92);
  border-radius: 8px;
  pointer-events: none;
}

.drop-hint-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.8rem;
  padding: 1.5rem 2rem;
  border: 2px dashed #2596be;
  border-radius: 10px;
  text-align: center;
}

.drop-icon { font-size: 2.5rem; }

.csv-group {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.file-group {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
  justify-content: center;
}

.btn {
  border-radius: 8px;
  border: 1px solid;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: whitesmoke;
  background-color: #181829;
  transition: border-color 0.25s, background-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-clear {
  background-color: #181829;
  border-color: #d8392b;
  color: white;
}

.btn-clear:hover{
  background-color:#313152 ;
}

.btn-clear:active {
  background-color: #d8392b;
}

.btn-open {
  background-color: #181829;
  border-color: #48bb78;
  display: block;
  margin: 0 auto;
}

.btn-open:enabled:hover{
    background-color: #313152;
}

.btn-open:active {
  background-color: #48bb78;
}

.btn-arbitrate {
  border-color: #f0a020;
  color: whitesmoke;
  margin-top: 0.6rem;
}
.btn-arbitrate:hover {
  background-color: #313152;
}

.btn-add {
  border-color: whitesmoke;
}

.btn-add:hover {
  border-color: #2596be;
  background-color:#313152 ;
}
.btn-add:active {
  border-color: #2596be;
  background-color: #2596be;
}

.cross {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
}

.image-btn {
  background: none;
  border:none;
  padding: 0;
  cursor: pointer;
  margin-left: auto;
}

.image-btn:hover {
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.image-btn:active {
  transform: translateY(1px) scale(0.99) translateZ(0);
  transition: transform 0.1s ease, background-color 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.table-header-row {
  display: flex;
  align-items: center;
  width: 90%;
  margin-top: 20px;
  margin-bottom: 10px;
}

.table-title {
  margin: 0 auto 0 0;
}

.search-group {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.input-search {
  width: 140px;
  border-radius: 8px;
  border: 1px solid whitesmoke;
  padding: 0.2em 0.8em;
  font-size: 1em;
  font-family: inherit;
  color: whitesmoke;
  background-color: #2d3748;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  transition: border-color 0.25s, background-color 0.25s;
}
.input-search::placeholder {
  color: rgba(245, 245, 245, 0.5);
}
.input-search:hover {
  background-color: #313152;
}
.input-search:focus {
  outline: none;
  border-color: #2596be;
  background-color: #313152;
}

.icon-lg {
  font-size: 1.6rem;
  line-height: 1;
}

.text {
  color: whitesmoke
}

.file-list {
  width: 90%;
  max-height: 250px;
  overflow-y: auto;
  background-color: #2d3748;
  border-radius: 4px;
  padding: 0.5rem;
  margin-bottom: 1.5rem;
}

.file-list li {
  padding: 0.5rem;
  margin: 0.25rem 0;
  border-radius: 4px;
  word-break: break-all;
  font-family: monospace;
  font-size: 0.9rem;
}

.file-list label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.hint {
  font-size: 0.8em;
  color: rgba(245, 245, 245, 0.5);
  margin-top: 0.4rem;
  margin-bottom: 0.8rem;
}

.hint code {
  font-family: monospace;
  color: rgba(245, 245, 245, 0.75);
}

.separator {
  height: 1px;
  width: 95%;
  background-color: whitesmoke;
  margin: 0 auto 16px auto;
}

.matrix-separator {
  margin-top: 16px;
  opacity: 0.3;
}

.data-table {
  width: 90%;
  background-color: #2d3748;
  border-radius: 4px;
  margin-bottom: 1.5rem;
  overflow: hidden; 
  text-align: center;
}

.data-table-header {
  display: flex;
  padding: 0.5rem;
  margin-top: 5px;
}

.data-table-header .col {
  font-weight: 600;
}

.data-table-body {
  max-height: 250px;
  overflow-y: auto;
  padding: 0.5rem;
}

.data-table-row {
  display: flex;
  padding-top: 4px;
  padding-bottom: 4px;
}

.col {
  width: 33.33%;
  padding-left: 8px;
  box-sizing: border-box;
}

/* Overlay + Spinner */

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  border-radius: 8px;
  z-index: 2000;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.1);
  border-left-color: #4299e1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

.overlay-text {
  color: white;
  font-size: 1.1rem;
  font-weight: 500;
}

.btn-cancel-import {
  margin-top: 1.2rem;
  border-color: #d8392b;
}

.btn-cancel-import:enabled:hover {
  background-color: #313152;
}

.btn-cancel-import:active {
  background-color: #d8392b;
}

.progress-group {
  width: min(70%, 420px);
  margin-top: 1rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.progress-track {
  appearance: none;
  -webkit-appearance: none;
  width: 100%;
  height: 8px;
  border: 0;
  background-color: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  overflow: hidden;
}

.progress-track::-webkit-progress-bar {
  background-color: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

.progress-track::-webkit-progress-value {
  background-color: #4299e1;
  border-radius: 4px;
  transition: width 0.2s ease;
}

.progress-track::-moz-progress-bar {
  background-color: #4299e1;
  border-radius: 4px;
  transition: width 0.2s ease;
}

.progress-label {
  width: 100%;
  margin: 0;
  color: rgba(255, 255, 255, 0.85);
  font-size: 0.85rem;
  font-family: monospace;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
