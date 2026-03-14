<template>
  <div class="bg"></div>
  <router-view></router-view>
  <QuitDialog
    :visible="showQuitDialog"
    @confirm="onQuitConfirm"
    @cancel="onQuitCancel"
  />
  <ErrorDialog />
</template>

<style>
:root {
  height: 100vh;
  --ease-out: cubic-bezier(0.25, 1, 0.5, 1);
  --ease-out-fast: cubic-bezier(0.22, 1, 0.36, 1);
}

html, body {
  height: 100%;
  margin: 0;
  padding: 0;
}

#app {
  height: 100%;
}

.bg {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: #282838;
  z-index: -1;
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>

<script>
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { exit } from '@tauri-apps/plugin-process';
import { info } from '@tauri-apps/plugin-log';
import QuitDialog from './components/QuitDialog.vue';
import ErrorDialog from './components/ErrorDialog.vue';

const appWindow = getCurrentWebviewWindow()

export default {
  components: { QuitDialog, ErrorDialog },

  data() {
    return {
      unlistenCloseEvent: null,
      showQuitDialog: false,
      pendingCloseEvent: null,
    };
  },

  async mounted() {
    this.unlistenCloseEvent = await appWindow.onCloseRequested(async (event) => {
      info("close requested")
      event.preventDefault();
      this.pendingCloseEvent = event;
      this.showQuitDialog = true;
    });
  },

  beforeUnmount() {
    if (this.unlistenCloseEvent) {
      this.unlistenCloseEvent();
    }
  },

  methods: {
    async onQuitConfirm() {
      info("exit confirmed")
      this.showQuitDialog = false;
      this.pendingCloseEvent = null;
      await exit(0);
    },
    onQuitCancel() {
      info("exit cancelled")
      this.showQuitDialog = false;
      this.pendingCloseEvent = null;
    },
  },
};
</script>
