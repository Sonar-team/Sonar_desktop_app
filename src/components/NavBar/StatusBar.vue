<template>
  <div class="status-bar">
    <div class="status-content">
      <p>Début: {{ heureDepart }}</p>
      <p>Fin: {{ heureFin }}</p>
      <p>Temps restant: {{ tempsReleve }}</p>
      <p>Temps écoulé: {{ tempsEcoule }}</p>
      <p>Trames reçues: {{ tramesRecues }} </p>
      <p>Matrice de flux: {{ tramesEnregistrees }}</p>
    </div>
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
  
  .status-content {
  display: flex;
  justify-content: space-around; /* Répartit les éléments équitablement */
  align-items: center; /* Aligne les éléments verticalement */
  flex-wrap: wrap; /* Permet le passage à la ligne si la largeur est insuffisante */
}
</style>