<template>
  <div class="bg"></div>
  <router-view></router-view>
</template>

<style>
:root {
  height: 100vh;
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
  background-color: #1a1a1a;
  z-index: -1;
}
</style>

<script>
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { info } from '@tauri-apps/plugin-log';
import { requestAppExit } from './utils/appExit';

const appWindow = getCurrentWebviewWindow()

export default {
  data() {
    return {
      // Add a data property for the unlisten function
      unlistenCloseEvent: null,
    };
  },

  async mounted() {
    console.log("mounted");

    // Set up the close event listener
    this.unlistenCloseEvent = await appWindow.onCloseRequested(async (event) => {
      info("close requested")
      event.preventDefault();
      await requestAppExit();
    });
  },

  beforeUnmount() {
    // Call the unlisten function when the component is unmounted
    if (this.unlistenCloseEvent) {
      this.unlistenCloseEvent();
    }
  }
};
</script>
