<template>
  <div class="legend-container">
    <h3>Légende</h3>
    
    <div class="legend-section">
      <h4>Couleurs des nœuds</h4>
      <div class="legend-item">
        <div class="color-box" style="background-color: #2196F3;"></div>
        <span>Nœud par défaut</span>
      </div>
      <div class="legend-item">
        <div class="color-box" style="background-color: #4CAF50;"></div>
        <span>Nœud actif/sélectionné</span>
      </div>
    </div>

    <div class="legend-section">
      <h4>Couleurs des arêtes (protocoles)</h4>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #FFFF00;"></div>
        <span>ARP</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #FFA500;"></div>
        <span>IPv4 / NTP</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #EE82EE;"></div>
        <span>IPv6</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #008000;"></div>
        <span>Profinet RT</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #0000FF;"></div>
        <span>TLS</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #FF0000;"></div>
        <span>DNS</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #ffffff;"></div>
        <span>Autre</span>
      </div>
    </div>

    <div class="legend-section">
      <h4>Style des arêtes</h4>
      <div class="legend-item">
        <div class="edge-line"></div>
        <span>Connexion simple</span>
      </div>
      <div class="legend-item">
        <div class="edge-line bidirectional"></div>
        <span>Connexion bidirectionnelle</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// Composant de légende pour le graphe réseau
</script>

<style scoped>
.legend-container {
  position: absolute;
  top: 10px;
  right: 10px;
  background-color: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  padding: 16px;
  color: #eaeaea;
  font-family: Arial, sans-serif;
  max-width: 250px;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.legend-container h3 {
  margin: 0 0 16px 0;
  color: #fff;
  font-size: 16px;
  font-weight: bold;
  border-bottom: 1px solid #3a3a3a;
  padding-bottom: 8px;
}

.legend-section {
  margin-bottom: 16px;
}

.legend-section h4 {
  margin: 0 0 8px 0;
  color: #bbb;
  font-size: 14px;
  font-weight: 600;
}

.legend-item {
  display: flex;
  align-items: center;
  margin-bottom: 6px;
  font-size: 12px;
}

.color-box {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  margin-right: 8px;
  border: 1px solid #555;
  flex-shrink: 0;
}

.edge-color {
  border-radius: 2px;
  width: 20px;
  height: 3px;
}

.edge-line {
  width: 20px;
  height: 2px;
  background-color: #ffffff;
  margin-right: 8px;
  position: relative;
  flex-shrink: 0;
}

.edge-line::after {
  content: '';
  position: absolute;
  right: -4px;
  top: -3px;
  width: 0;
  height: 0;
  border-left: 6px solid #ffffff;
  border-top: 4px solid transparent;
  border-bottom: 4px solid transparent;
}

.bidirectional::before {
  content: '';
  position: absolute;
  left: -4px;
  top: -3px;
  width: 0;
  height: 0;
  border-right: 6px solid #ffffff;
  border-top: 4px solid transparent;
  border-bottom: 4px solid transparent;
}
</style>