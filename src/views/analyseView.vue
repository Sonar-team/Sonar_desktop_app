<template>
    <div>Annalyse</div>
    <BottomLong :net-Interface=this.netInterface />
    <StopButton />
    
</template>

<script>
import BottomLong from '../components/analyseView/BottomLong.vue';
import StopButton from '../components/analyseView/StopButton.vue';

export default {
  props: {
    netInterface: String
  },
  components: {
    BottomLong,
    StopButton
},
    data() {
      return {
      netInterface: this.$route.query.interface
    };
      
    },
    methods: {

    },
    mounted() {
      console.log("analyse vue Interface sélectionnée :", netInterface);
      // Utilisez netInterface selon vos besoins ici

    }
  };
</script>