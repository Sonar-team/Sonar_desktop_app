<template>
  <div class="status-bar">
    <p>Status: En cours d’analyse | Trames reçues: {{ tramesRecues }} | Trames enregistrées: {{ tramesEnregistrees }}</p>
  </div>
</template>

<script>
  export default {
    props: {
      tramesRecues: {
        type: Number,
        default: 0
      },
      tramesEnregistrees: {
        type: Number,
        default: 0
      }
    }
  }
</script>
  
<style scoped>
  .status-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    width: 100%;
    background-color: #0b1b25;
    color: white;
    text-align: center;
    padding: 4px;
    font-size: 12px;
  }
</style>