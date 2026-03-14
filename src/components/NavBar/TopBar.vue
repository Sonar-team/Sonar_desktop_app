<template>
  <div class="top-bar">

    <!-- App controls: left side -->
    <div class="bar-left">

    <!-- Start -->
    <button class="image-btn logo-btn" @click="start" title="Démarrer (ctrl+p)" :disabled="isRunning">
      <img src="/src-tauri/icons/StoreLogo.png" alt="Sonar" class="logo-img" />
    </button>

    <!-- Stop -->
    <button class="image-btn" @click="stop" title="Arrêter (ctrl+shift+p)" :disabled="!isRunning">
      <svg viewBox="0 0 16 16" fill="currentColor"><rect x="3.5" y="3.5" width="9" height="9" rx="1.5"/></svg>
    </button>

    <!-- Reset -->
    <button class="image-btn" @click="reset" title="Réinitialiser (ctrl+shift+r)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M13.5 8a5.5 5.5 0 1 1-1.1-3.3"/>
        <path d="M13.5 3v2.5H11"/>
      </svg>
    </button>

    <div class="sep"/>

    <!-- Config -->
    <button class="image-btn" title="Config (ctrl+,)" :disabled="isRunning" @click="handleConfigClick">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
        <circle cx="8" cy="8" r="2"/>
        <path d="M8 1.5v1M8 13.5v1M1.5 8h1M13.5 8h1M3.4 3.4l.7.7M11.9 11.9l.7.7M11.9 4.1l-.7.7M4.8 11.2l-.7.7"/>
      </svg>
    </button>

    <!-- Save -->
    <button class="image-btn" @click="triggerSave" title="Sauvegarder (ctrl+s)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
        <path d="M2.5 13.5h11V5l-3-3H2.5v11.5z"/>
        <rect x="5" y="9" width="6" height="4.5" rx="0.5"/>
        <rect x="5.5" y="2.5" width="4" height="3" rx="0.5"/>
      </svg>
    </button>

    <!-- Open -->
    <button class="image-btn" @click="displayPcapOpener" title="Ouvrir (ctrl+o)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
        <path d="M1.5 4.5h4l1.5 2H14v7H1.5V4.5z"/>
        <path d="M1.5 6.5v-3a1 1 0 0 1 1-1h3"/>
      </svg>
    </button>

    <!-- Logs -->
    <button class="image-btn" @click="export_logs" title="Logs (ctrl+l)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
        <rect x="3" y="1.5" width="10" height="13" rx="1"/>
        <line x1="5.5" y1="5" x2="10.5" y2="5"/>
        <line x1="5.5" y1="8" x2="10.5" y2="8"/>
        <line x1="5.5" y1="11" x2="8.5" y2="11"/>
      </svg>
    </button>

    <!-- Filter -->
    <button class="image-btn" @click="handleFilterClick" title="Filtrer (ctrl+f)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
        <path d="M2 3.5h12M4.5 7.5h7M7 11.5h2"/>
      </svg>
    </button>

    <div class="sep"/>

    <!-- Quit -->
    <button class="image-btn quit-btn" @click="quit" title="Quitter (ctrl+q)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <circle cx="8" cy="8" r="5.5"/>
        <path d="M8 4.5v3.5"/>
        <path d="M8 4.5v0" stroke-width="2"/>
        <path d="M5.2 5.5A4 4 0 1 0 10.8 5.5"/>
      </svg>
    </button>

    </div><!-- end bar-left -->

    <!-- Title centered — drag region -->
    <div class="bar-title" @mousedown="onDragStart">SONAR</div>

    <!-- Window controls: right side -->
    <div class="bar-right">
      <button class="wm-btn" @click="minimizeWindow" title="Réduire">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <line x1="3.5" y1="8" x2="12.5" y2="8"/>
        </svg>
      </button>
      <button class="wm-btn" @click="toggleMaximize" title="Agrandir">
        <svg v-if="!isMaximized" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3.5" y="3.5" width="9" height="9" rx="1"/>
        </svg>
        <svg v-else viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="5" y="5" width="7" height="7" rx="1"/>
          <path d="M4 11V4h7"/>
        </svg>
      </button>
      <button class="wm-btn wm-close" @click="closeWindow" title="Fermer">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <line x1="4" y1="4" x2="12" y2="12"/>
          <line x1="12" y1="4" x2="4" y2="12"/>
        </svg>
      </button>
    </div>

  </div>
</template>

<script lang="ts">
import { Channel, invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { exit } from '@tauri-apps/plugin-process';
import { info, error } from '@tauri-apps/plugin-log';
import { save } from '@tauri-apps/plugin-dialog';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
// when using `"withGlobalTauri": true`, you may use
// const { register } = window.__TAURI__.globalShortcut;



import { displayCaptureError } from '../../errors/capture';
import { getCurrentDate } from '../../utils/time';
import { useCaptureStore } from '../../store/capture';
import { CaptureEvent } from '../../types/capture';

// Stocké hors de Vue pour éviter que le Proxy réactif ne casse les méthodes Tauri
let appWindow: ReturnType<typeof getCurrentWebviewWindow> | null = null;

export default {
  name: "TopBar",
  emits: ['toggle-config','toggle-pcap','toggle-filter'],

  computed: {
    buttonText(): string {
      return this.captureStore.showMatrice ? 'Graphique' : 'Matrice';
    },
    captureStore() {
      return useCaptureStore();
    },

    isRunning(): boolean {
      return this.captureStore.isRunning;
    },
  },
  data() {
    return {
      showMatrice: true,
      shortcuts: [] as string[],
      isMaximized: false,
    };
  },
  async mounted() {
    appWindow = getCurrentWebviewWindow();
    this.isMaximized = await appWindow.isMaximized();
    await appWindow.onResized(async () => {
      this.isMaximized = await appWindow!.isMaximized();
    });

    // Sauvegardes
    this.bindShortcut('CommandOrControl+S', () => this.SaveAsCsv());
    this.bindShortcut('CommandOrControl+Shift+S', () => this.SaveAsXlsx());

    // Reset
    this.bindShortcut('CommandOrControl+Shift+R', () => this.reset());

    // Start / Stop
    // Choix 1 (classique "Play/Stop")
    this.bindShortcut('CommandOrControl+P', () => this.start());
    this.bindShortcut('CommandOrControl+Shift+P', () => this.stop());

    // Ouvrir (pcap opener)
    this.bindShortcut('CommandOrControl+O', () => this.displayPcapOpener());

    // Config
    this.bindShortcut('CommandOrControl+,', () => this.handleConfigClick());

    // Filtre
    this.bindShortcut('CommandOrControl+F', () => this.handleFilterClick());

    // Logs
    this.bindShortcut('CommandOrControl+L', () => this.export_logs());

    // Quit
    this.bindShortcut('CommandOrControl+Q', () => this.quit());
  },

  async beforeUnmount() {
  // recommandé en dev/hot reload
    await this.unbindAllShortcuts();
  },
  methods: {
    bindShortcut(shortcut: string, handler: () => void) {
      this.shortcuts.push(shortcut);
      register(shortcut, (event) => {
        if (event.state === 'Released') handler();
      });
    },

    async unbindAllShortcuts() {
      // unregister accepte string | string[]
      if (this.shortcuts.length > 0) {
        await unregister(this.shortcuts);
      }
      this.shortcuts = [];
    },
    async export_logs() {
      info("export logs")
      const response = await save({
        filters: [{
          name: '.log',
          extensions: ['log']
        }],
        title: 'Sauvegarder les logs',
        defaultPath: 'sonar.log'
      });

      if (response) {
        // Attendez que l'invocation d'API pour sauvegarder soit terminée
        const saveResponse = await invoke('export_logs', { destination: response });
        info("Sauvegarde terminée:", saveResponse);
        return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
      } else {
        info("Aucun chemin de fichier sélectionné");
        throw new Error("Sauvegarde annulée ou chemin non sélectionné");
      }
    },

    async SaveAsCsv() {
      info("Save as csv")
      save({
        filters: [{
          name: '.csv',
          extensions: ['csv']
        }],
        title: 'Sauvegarder la matrice de flux',
        defaultPath: getCurrentDate()+ '_DR_Matrice.csv' // Set the default file name here
      
      }).then((response) => 
        invoke('export_csv', { path: response })
          .then((response: any) => 
            info("response: ", response))
          .catch((error: any) => 
            error("error: ", error))
      )
    },
    async SaveAsXlsx() {
      try {
        info("Début de la sauvegarde en xlsx");
        const response = await save({
          filters: [{
            name: '.xlsx',
            extensions: ['xlsx']
          }],
          title: 'Sauvegarder la matrice de flux',
          defaultPath: getCurrentDate() + '_DR_Matrice' + '.xlsx'
        });

        if (response) {
          // Attendez que l'invocation d'API pour sauvegarder soit terminée
          const saveResponse = await invoke('save_packets_to_excel', { file_path: response });
          info("Sauvegarde terminée:", saveResponse);
          return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
        } else {
          info("Aucun chemin de fichier sélectionné");
          throw new Error("Sauvegarde annulée ou chemin non sélectionné");
        }
      } catch (error) {
        error("Erreur lors de la sauvegarde en xlsx:", error);
        throw error; // Relancer l'erreur pour la gestion dans quit()
      }
    },
    async triggerSave() {
      info("trigger save")
      this.SaveAsCsv();
      
    },
    async reset() {
      info("reset")
      await invoke('reset_capture');
      this.$bus.emit('reset');
    },


    handleConfigClick() {
      info("[TopBar] Bouton config cliqué");
      this.$emit('toggle-config');
    },
    displayPcapOpener() {
      info("[TopBar] Bouton open cliqué");
      this.$emit('toggle-pcap');
    },
    handleFilterClick() {
      info("[TopBar] Bouton filter cliqué");
      this.$emit('toggle-filter');
    },
    async start() {
      if (this.captureStore.isRunning) {
        return;
      }
      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent); // 🟢 rendre le Channel accessible

      await invoke('start_capture', { onEvent })
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          info('Capture démarrée : ' + this.captureStore.isRunning);
        })
        .catch(displayCaptureError);
    },
    async stop() {
      if (!this.captureStore.isRunning) {
        return;
      }
      const onEvent = this.captureStore.getChannel();
      await invoke('stop_capture',{ onEvent })
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          info('Capture arrêtée : ' + this.captureStore.isRunning);
        })
        .catch(displayCaptureError);
    },
    toggleView() {
      info('Vue basculée');
    },
    async quit() {
      info('Fermeture demandée');
      await exit(0);
    },
    toggleConfig() {
      info('Ouverture panneau config');
    },

    async minimizeWindow() {
      await appWindow?.minimize();
    },
    async toggleMaximize() {
      await appWindow?.toggleMaximize();
    },
    async closeWindow() {
      await appWindow?.close();
    },
    async onDragStart(e: MouseEvent) {
      if (e.button === 0) {
        await appWindow?.startDragging();
      }
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
  background-color: #2c2c3a;
  display: flex;
  align-items: center;
  border-bottom: 1px solid #3c3c50;
  z-index: 9999;
  user-select: none;
}

.bar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  flex-shrink: 0;
}

.bar-title {
  flex: 1;
  text-align: center;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.12em;
  color: #585872;
  text-transform: uppercase;
  cursor: default;
}

.bar-right {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  height: 100%;
}

/* Window management buttons */
.wm-btn {
  width: 46px;
  height: 100%;
  background: transparent;
  border: none;
  color: #585872;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease, color 0.15s ease;
}
.wm-btn svg {
  transition: transform 0.1s ease;
}
.wm-btn:active svg {
  transform: scale(0.82);
}

.wm-btn svg {
  width: 14px;
  height: 14px;
}

.wm-btn:hover {
  background-color: #383850;
  color: #9090a0;
}

.wm-close:hover {
  background-color: #5a2828;
  color: #e0a0a0;
}

.image-btn {
  background: transparent;
  border: none;
  padding: 5px;
  border-radius: 4px;
  cursor: pointer;
  color: #6a6a80;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s ease, background-color 0.15s ease, transform 0.1s ease;
}

.image-btn svg {
  width: 16px;
  height: 16px;
  display: block;
  transition: transform 0.1s ease;
}

.image-btn:hover {
  background-color: #383850;
  color: #9090a0;
}

.image-btn:active {
  color: #d4d4d8;
  background-color: #3c3c50;
  transform: scale(0.88);
}

.image-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
  background-color: transparent;
  color: #6a6a80;
}

.quit-btn:hover {
  color: #8a4a4a;
}

.logo-btn {
  padding: 3px;
  margin-right: 4px;
}

.logo-img {
  width: 22px;
  height: 22px;
  display: block;
  opacity: 0.7;
  transition: opacity 0.15s ease;
}

.logo-btn:hover .logo-img {
  opacity: 1;
}

.logo-btn:disabled .logo-img {
  opacity: 0.25;
}

.sep {
  width: 1px;
  height: 18px;
  background: #3c3c50;
  margin: 0 2px;
  flex-shrink: 0;
}
</style>