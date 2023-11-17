<template>
    <div>"hello"</div>
    <TramTab />
    <BottomLong />
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';

import TramTab from '../components/CaptureVue/trameTab.vue';
export default {
  components: {
    TramTab,
    BottomLong
},
    data() {
      
    },
    methods: {

    },
    mounted() {
      console.log("analyse mounted");

    }
  };
</script>