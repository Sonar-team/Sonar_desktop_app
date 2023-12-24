<template>
    <div class="container">
      <div class="sidebar">

        
        <!-- Navigation Links -->
        <router-link to="/" class="nav-link">Home</router-link>
        <router-link :to="{ name: 'Analyse' }" class="nav-link">Analyse</router-link>
        <router-link to="/graph" class="nav-link">Graph View</router-link>
  
      </div>
  
      <!-- Contenu principal de la page -->
      <div class="content">
        <router-view></router-view>
      </div>
    </div>
  </template>
  
<script>
  export default {
  data() {
    return {
      niveauConfidentialite: '',
      installationName:''
    };
  },
  components: {
    NetworkGraphComponent
  },
  
  mounted() {
    console.log("analyse mounted");
    this.updateTempsReleve();


  }

}
</script>

  
  <style scoped>

  </style>
  