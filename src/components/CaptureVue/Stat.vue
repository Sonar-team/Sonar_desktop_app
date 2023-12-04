<template>
    <div class="chart-container">
      <BarChart />
      <PieChart />
    </div>
  </template>
  
  <script>
  import BarChart from './Stats/Bar.vue';
  import PieChart from './Stats/Pie.vue';
  
  export default {
    name: 'App',
    components: {
      BarChart,
      PieChart
    }
  }
  </script>
  
  <style>
table_mat {
  width: 100%;
  border-collapse: collapse;
  table-layout: auto; /* Pour des largeurs de colonnes flexibles */
  margin: 20px 0; /* Ajouter de l'espace autour du tableau */
  background-color: rgba(255, 255, 255, 0.9); /* Fond semi-transparent pour la lisibilité */
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* Ombre portée pour une meilleure distinction */
}

th_mat, td_mat {
  padding: 15px; /* Augmentation du padding */
  text-align: left; /* Alignement du texte à gauche */
  border-bottom: 1px solid #ddd; /* Bordure inférieure plus claire */
  color: #042330; /* Couleur de texte foncée pour contraste */
  font-size: 15px; /* Taille de police légèrement augmentée */
}

th_mat {
  background-color: #3a6978; /* Couleur d'arrière-plan foncée pour les en-têtes */
  color: #ffffff; /* Texte blanc pour contraste */
  font-size: 16px; /* Taille de police plus grande pour les en-têtes */
}

tbody tr:nth-child(even) {
  background-color: rgba(255, 255, 255, 0.3); /* Lignes alternées plus claires */
}

tbody tr:hover {
  background-color: rgba(255, 255, 255, 0.5); /* Effet de survol */
}

  
  </style>
  