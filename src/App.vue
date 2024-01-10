<template>
<div class="bg"></div>
    <router-view></router-view> 

</template>

<script>
  import { appWindow } from "@tauri-apps/api/window";
  import { confirm } from '@tauri-apps/api/dialog';

export default {

  async mounted() {
      console.log("mounted");
      const unlisten = await appWindow.onCloseRequested(async (event) => {
      const confirmed = await confirm('Etes vous sûr ?');
      if (!confirmed) {
        // user did not confirm closing the window; let's prevent it
        event.preventDefault();
      }
    });

    this.$once('hook:beforeDestroy', () => {
      unlisten();
    });
    }
};
</script>

<style scoped>
/* Global styles for your app */

</style>
