<!-- YourVueComponent.vue -->
<script  >
import { NetworkGraphComponent } from "../components/GraphVue/NetworkGraphComponent.vue"
export default {
  data() {
    return {

    };
  },
  components: {
    NetworkGraphComponent,
  },
}
</script>

<template>
    <div class="button-container">
        <button >Vue analyse</button>
    </div>
    <NetworkGraphComponent />

</template>

<style>

/* You can add more custom styles targeting specific classes within the v-network-graph library */
</style>

