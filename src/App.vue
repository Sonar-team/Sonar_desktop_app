<template>
  <div class="bg"></div>
  <router-view></router-view>
</template>

<script>
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { confirm } from '@tauri-apps/plugin-dialog';
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
      const confirmed = await confirm('Etes vous sûr ?');
      if (!confirmed) {
        // User did not confirm closing the window; let's prevent it
        event.preventDefault();
      }
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


