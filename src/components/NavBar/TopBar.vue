<!--
  Barre d'actions principale : démarrer/arrêter la capture, reset, exports
  (matrice, labels, logs), ouverture des panneaux (import, filtre, config)
  et raccourcis clavier associés.
-->
<template>
  <div class="top-bar">
    <button type="button" class="image-btn" @click="start" title="Démarrer (ctrl+p)" aria-label="Démarrer la capture" :disabled="isRunning || activePanel !== null">
      <img src="../../../src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
    </button>

    <button type="button" class="image-btn" @click="stop" title="Arrêter (ctrl+shift+p)" aria-label="Arrêter la capture" :disabled="!isRunning">🛑</button>
    <button type="button" class="image-btn" @click="reset" :disabled="activePanel !== null" title="Réinitialiser (ctrl+shift+r)" aria-label="Réinitialiser">🔄</button>
    <button type="button" class="image-btn" title="Config (ctrl+,)" aria-label="Configuration" :disabled="isRunning" @click="handleConfigClick">
      <img src="../../assets/mdi--gear.svg" alt="Configuration" class="icon-img" />
    </button>

    <button type="button" class="image-btn" @click="triggerSave" title="Sauvegarder (ctrl+s)" aria-label="Sauvegarder la matrice de flux">💾</button>
    <button type="button" class="image-btn" @click="saveProject" title="Enregistrer le projet (.sonar)" aria-label="Enregistrer le projet">🗃️</button>
    <button type="button" class="image-btn" @click="openProject" :disabled="isRunning || captureStore.hasData" title="Ouvrir un projet (.sonar)" aria-label="Ouvrir un projet">📂</button>
    <button type="button" class="image-btn" @click="SaveLabels" title="Exporter les labels" aria-label="Exporter les labels">🏷️</button>
    <button type="button" class="image-btn" @click="handleLabelsClick" :disabled="isRunning" title="Gérer les labels" aria-label="Gérer les labels">🗂️</button>

    <button type="button" class="image-btn" @click="displayPcapOpener" :disabled="isRunning || captureStore.hasData" title="Ouvrir un fichier Pcap ou une matrice de flux (ctrl+o)" aria-label="Ouvrir un fichier Pcap ou une matrice de flux">📄</button>
    <button type="button" class="image-btn" @click="displayCsvOpener" :disabled="isRunning" title="Importer un fichier de label"><img src="../../assets/images/import_csv.png" alt="Importer un fichier de label" /></button>

    <button type="button" class="image-btn" @click="quit" title="Quitter (ctrl+q)" aria-label="Quitter">❎</button>
    <button type="button" class="image-btn" @click="export_logs" title="Logs (ctrl+l)" aria-label="Exporter les logs">📒</button>
    <button type="button" class="image-btn" @click="handleFilterClick" :disabled="isRunning" title="Filtrer (ctrl+f)" aria-label="Filtrer">🔍</button>
  </div>
</template>

<script lang="ts">
import { Channel, invoke } from '@tauri-apps/api/core';
import { info, error } from '@tauri-apps/plugin-log';
import { ask, open, save } from '@tauri-apps/plugin-dialog';

import { displayCaptureError } from '../../errors/capture'; // Gestion des erreurs propre
import { getCurrentDate } from '../../utils/time';
import { useCaptureStore } from '../../store/capture';
import { CaptureEvent } from '../../types/capture';
import { requestAppExit } from '../../utils/appExit';
import { getLastProjectDir, recordRecentProject } from '../../utils/recentProjects';

type Panel = 'config' | 'pcap' | 'csv' | 'filter' | 'labels';

const BUSY_MESSAGE = "Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.";

export default {
  name: "TopBar",
  emits: ['toggle-config', 'toggle-pcap','toggle-csv', 'toggle-filter', 'toggle-graph', 'toggle-labels'],

  props: {
    configOpen: Boolean,
    filterOpen: Boolean,
    csvOpen: Boolean,
    pcapOpen: Boolean,
    labelsOpen: Boolean,
  },

  watch: {
  configOpen(val) { if (!val && this.activePanel === 'config') this.activePanel = null; },
  filterOpen(val) { if (!val && this.activePanel === 'filter') this.activePanel = null; },
  csvOpen(val)    { if (!val && this.activePanel === 'csv') this.activePanel = null; },
  pcapOpen(val)   { if (!val && this.activePanel === 'pcap') this.activePanel = null; },
  labelsOpen(val) { if (!val && this.activePanel === 'labels') this.activePanel = null; },
},

  computed: {
    captureStore() {
      return useCaptureStore();
    },

    isRunning(): boolean {
      return this.captureStore.isRunning;
    },
  },
  data() {
    return {
      localHandler: null as ((e: KeyboardEvent) => void) | null,
      activePanel: null as Panel | null,
    };
  },
  async mounted() {
    // Raccourcis locaux à la fenêtre : des raccourcis globaux voleraient
    // Ctrl+S, Ctrl+F, etc. à toutes les applications du système.
    this.localHandler = (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;
      if (!ctrl) return;

      const key = e.key.toLowerCase();
      if (key === 's' && !e.shiftKey) { e.preventDefault(); this.SaveAsCsv(); }
      else if (key === 'r' && e.shiftKey) { e.preventDefault(); this.reset(); }
      else if (key === 'p' && !e.shiftKey) { e.preventDefault(); this.start(); }
      else if (key === 'p' && e.shiftKey) { e.preventDefault(); this.stop(); }
      else if (key === 'o') { e.preventDefault(); this.displayPcapOpener(); }
      else if (key === ',') { e.preventDefault(); this.handleConfigClick(); }
      else if (key === 'f') { e.preventDefault(); this.handleFilterClick(); }
      else if (key === 'l') { e.preventDefault(); this.export_logs(); }
      else if (key === 'q') { e.preventDefault(); this.quit(); }
    };
    window.addEventListener('keydown', this.localHandler);

    useCaptureStore().refreshHasData(); // Vérifie s'il y a déjà des données au montage pour ajuster l'état de hasData
  },

  async beforeUnmount() {
    // recommandé en dev/hot reload
    if (this.localHandler) {
      window.removeEventListener('keydown', this.localHandler);
      this.localHandler = null;
    }
  },
  methods: {
    /** Pose le verrou global d'import/export, ferme le panneau actif,
     * exécute `action`, puis libère le verrou (même en cas d'erreur). Si une
     * opération est déjà en cours, `action` n'est pas exécutée. */
    async withImportLock(action: () => Promise<void>) {
      if (useCaptureStore().isImporting) {
        info(BUSY_MESSAGE);
        return;
      }
      if (this.activePanel !== null) {
        this.$emit(`toggle-${this.activePanel}`, false);
      }
      useCaptureStore().isImporting = true;
      try {
        await action();
      } finally {
        useCaptureStore().isImporting = false;
      }
    },

    async export_logs() {
      info("export logs")
      await this.withImportLock(async () => {
        try {
          const response = await save({
            filters: [{ name: '.zip', extensions: ['zip'] }],
            title: 'Sauvegarder les logs',
            defaultPath: 'sonar-logs.zip'
          });

          if (!response) {
            info("Aucun chemin de fichier sélectionné");
            return;
          }
          // Attendez que l'invocation d'API pour sauvegarder soit terminée
          const saveResponse = await invoke('export_logs', { destination: response });
          info(`Sauvegarde terminée: ${JSON.stringify(saveResponse)}`);
        } catch (err) {
          error(`Erreur export logs: ${err}`);
          await displayCaptureError(err);
        }
      });
    },

    async SaveAsCsv() {
      info("Save as csv");
      await this.withImportLock(async () => {
        try {
          const response = await save({
            filters: [{ name: '.csv', extensions: ['csv'] }],
            title: 'Sauvegarder la matrice de flux',
            defaultPath: getCurrentDate() + '_DR_Matrice.csv'
          });

          if (response) {
            const saveResponse = await invoke('export_csv', { path: response });
            info(`response: ${JSON.stringify(saveResponse)}`);
          } else {
            info("Aucun chemin sélectionné");
          }
        } catch (err) {
          error(`Erreur sauvegarde csv: ${JSON.stringify(err)}`);
        }
      });
    },
    async SaveLabels() {
      info("Export des labels");
      await this.withImportLock(async () => {
        try {
          const response = await save({
            filters: [{ name: '.csv', extensions: ['csv'] }],
            title: 'Exporter les labels',
            defaultPath: getCurrentDate() + '_labels.csv'
          });

          if (response) {
            await invoke('export_label_file', { path: response });
            info("Labels exportés");
          } else {
            info("Aucun chemin sélectionné");
          }
        } catch (err) {
          error(`Erreur export labels: ${err}`);
          displayCaptureError(err);
        }
      });
    },
    async triggerSave() {
      info("trigger save")
      this.SaveAsCsv();

    },
    async saveProject() {
      info("Enregistrement du projet");
      await this.withImportLock(async () => {
        try {
          const lastDir = await getLastProjectDir();
          const fileName = getCurrentDate() + '_projet.sonar';
          const response = await save({
            filters: [{ name: 'Projet SONAR', extensions: ['sonar'] }],
            title: 'Enregistrer le projet',
            defaultPath: lastDir ? `${lastDir}/${fileName}` : fileName
          });

          if (!response) {
            info("Aucun chemin sélectionné");
            return;
          }
          await invoke('save_project', { path: response });
          await recordRecentProject(response);
          info("Projet enregistré");
        } catch (err) {
          error(`Erreur enregistrement projet: ${err}`);
          await displayCaptureError(err);
        }
      });
    },
    async openProject() {
      info("Ouverture d'un projet");
      await this.withImportLock(async () => {
        try {
          const response = await open({
            multiple: false,
            filters: [{ name: 'Projet SONAR', extensions: ['sonar'] }],
            title: 'Ouvrir un projet',
            defaultPath: await getLastProjectDir()
          });

          if (!response) {
            info("Aucun projet sélectionné");
            return;
          }
          // Même discipline que l'import de matrice : un Channel Tauri est à
          // usage unique, on en crée un neuf par invoke ; pendant une capture
          // live le backend passe par le channel de la capture.
          const store = useCaptureStore();
          const onEvent = new Channel<CaptureEvent>();
          if (!store.isRunning) {
            store.setChannel(onEvent);
          }
          store.clearImportProgress();
          await invoke('open_project', { path: response, onEvent });
          await recordRecentProject(response);
          await store.refreshHasData();
          info("Projet ouvert");
        } catch (err) {
          error(`Erreur ouverture projet: ${err}`);
          await displayCaptureError(err);
        }
      });
    },
    async reset() {
      info("reset")

      if (useCaptureStore().isImporting) {
        info(BUSY_MESSAGE);
        return;
      }

      try {
        // Confirmation seulement si du travail non enregistré serait perdu
        // (#159). En cas de doute (IPC en échec), on demande.
        const dirty = await invoke<boolean>('is_session_dirty').catch(() => true);
        if (dirty) {
          const confirmed = await ask(
            'La réinitialisation effacera le relevé non enregistré.\nContinuer ?',
            { title: 'SONAR', kind: 'warning' },
          );
          if (!confirmed) {
            info('reset annulé par l\'utilisateur');
            return;
          }
        }
        await invoke('reset_capture');
        await useCaptureStore().refreshHasData();
        this.$bus.emit('reset');
      } catch (err) {
        // Le contrôle frontend peut perdre une course face à un import qui
        // démarre juste après. Le refus typé du backend reste alors visible.
        await displayCaptureError(err);
      }
    },


    /** Ouvre `panel` (sauf capture en cours ou `blocked`), en fermant
     * d'abord le panneau actif s'il y en a un autre. */
    openPanel(panel: Panel, blocked = false) {
      if (this.captureStore.isRunning || blocked) return;
      if (useCaptureStore().isImporting) {
        info(BUSY_MESSAGE);
        return;
      }
      if (this.activePanel !== null && this.activePanel !== panel) {
        this.$emit(`toggle-${this.activePanel}`, false);
      }
      this.activePanel = panel;
      this.$emit(`toggle-${panel}`);
    },
    handleConfigClick() {
      info("[TopBar] Bouton config cliqué");
      this.openPanel('config');
    },
    displayPcapOpener() {
      info("[TopBar] Bouton open cliqué");
      // Empêche d'ouvrir le panneau pcap si la matrice de flux contient déjà des données.
      this.openPanel('pcap', this.captureStore.hasData);
    },
    displayCsvOpener() {
      info("[TopBar] Bouton open cliqué");
      this.openPanel('csv');
    },
    handleLabelsClick() {
      this.openPanel('labels');
    },
    handleFilterClick() {
      info("[TopBar] Bouton filter cliqué");
      this.openPanel('filter');
    },
    async start() {
      if (this.activePanel !== null || useCaptureStore().isImporting) return;

      // isStarting posé de façon synchrone : un double-clic ne peut pas
      // déclencher deux invocations (et donc deux channels) concurrentes.
      if (this.captureStore.isRunning || this.captureStore.isStarting) {
        return;
      }
      this.captureStore.isStarting = true;

      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent); // 🟢 rendre le Channel accessible

      try {
        const status = await invoke('start_capture', { onEvent });
        const typedStatus = status as { is_running: boolean; session_id: number };
        this.captureStore.updateStatus(typedStatus);
        info('Capture démarrée : ' + this.captureStore.isRunning);
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.captureStore.isStarting = false;
      }
    },

    async stop() {
      if (!this.captureStore.isRunning || useCaptureStore().isImporting) {
        return;
      }
      const onEvent = this.captureStore.getChannel();
      await invoke('stop_capture',{ onEvent })
        .then((status) => {
          const typedStatus = status as { is_running: boolean; session_id: number };
          this.captureStore.updateStatus(typedStatus);
          info('Capture arrêtée : ' + this.captureStore.isRunning);
        })
        .catch((err) => {
          // Même en erreur (ex. échec d'envoi de l'événement Stopped), le
          // backend a joint les threads et normalisé son statut : le store
          // doit refléter l'arrêt, sinon l'UI reste bloquée « en cours ».
          this.captureStore.updateStatus({ is_running: false });
          displayCaptureError(err);
        })
        .finally(() =>useCaptureStore().refreshHasData());
    },
    async quit() {
      info('Fermeture demandée');
      await requestAppExit();
    },
  }
}
</script>

<style scoped>
.top-bar {
  position: fixed;
  top: 0;
  left: 0;
  height: 40px;
  width: 100%;
  background-color: #070416;
  display: flex;
  align-items: center;
  padding: 0 10px;
  gap: 8px;
  border-bottom: 1px solid #252526;
  z-index: 9999;
}

.image-btn {
  background: transparent;
  border: none;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 18px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-font-smoothing: subpixel-antialiased;
}

.image-btn:hover {
  background-color: #3f4758;
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.image-btn:active {
  transform: translateY(1px) scale(0.99) translateZ(0);
  transition: transform 0.1s ease, background-color 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
.image-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  background-color: transparent;
  transform: none !important;
  box-shadow: none !important;
}
.icon-img {
  height: 30;
  width: 30px;
  vertical-align: middle;
}
</style>
