<template>
<div class="bg"></div>
    <router-view></router-view> 

</template>

<script>
import { RouterView } from 'vue-router'
export default {
  mounted() {
      console.log("mounted");

    }
  // Any additional script or logic for App.vue
};
</script>

<style>
/* Global styles for your app */

</style>
