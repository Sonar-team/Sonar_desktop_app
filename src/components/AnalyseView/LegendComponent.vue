<template>
  <div class="legend-container">
    <h3>Légende</h3>
    
    <div class="legend-section">
      <h4>Couleurs des nœuds</h4>
      <div class="legend-item">
        <div class="color-box" style="background-color: #4a6aa0;"></div>
        <span>@IP Publique</span>
      </div>
      <div class="legend-item">
        <div class="color-box" style="background-color: #487048;"></div>
        <span>@IP Privée</span>
      </div>
    </div>

    <div class="legend-section">
      <h4>Couleurs des arêtes (protocoles)</h4>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #a89040;"></div>
        <span>ARP</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #a07030;"></div>
        <span>IPv4 / NTP</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #8a5f8a;"></div>
        <span>IPv6</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #3f7040;"></div>
        <span>Profinet RT</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #3a5c9a;"></div>
        <span>TLS</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #8a3c3c;"></div>
        <span>DNS</span>
      </div>
      <div class="legend-item">
        <div class="color-box edge-color" style="background-color: #808095;"></div>
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
  background-color: #343448;
  border: 1px solid #484858;
  border-radius: 8px;
  padding: 14px;
  color: #9090a0;
  font-family: Arial, sans-serif;
  max-width: 220px;
  z-index: 100;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}

.legend-container h3 {
  margin: 0 0 12px 0;
  color: #888894;
  font-size: 11px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  border-bottom: 1px solid #484858;
  padding-bottom: 8px;
}

.legend-section {
  margin-bottom: 12px;
}

.legend-section h4 {
  margin: 0 0 6px 0;
  color: #656578;
  font-size: 11px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
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
  background-color: #808095;
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
  border-left: 6px solid #808095;
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
  border-right: 6px solid #808095;
  border-top: 4px solid transparent;
  border-bottom: 4px solid transparent;
}
</style>