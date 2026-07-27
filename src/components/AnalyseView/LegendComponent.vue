<!--
  Légende du graphe réseau : couleurs par protocole (voir
  utils/protocolColors.ts).
-->
<template>
  <div class="legend-container">
    <h3>Légende</h3>
    
    <div class="legend-section">
      <h4>Couleurs des nœuds</h4>
      <div class="legend-item">
        <div class="color-box public-ip-color"></div>
        <span>@IP Publique</span>
      </div>
      <div class="legend-item">
        <div class="color-box private-ip-color"></div>
        <span>@IP Privée</span>
      </div>
    </div>

    <div class="legend-section">
      <h4>Couleurs des arêtes (protocoles)</h4>
      <div
        v-for="item in PROTOCOL_COLOR_LEGEND"
        :key="item.label"
        class="legend-item"
      >
        <svg
          class="protocol-color-swatch"
          viewBox="0 0 20 3"
          aria-hidden="true"
          focusable="false"
        >
          <rect width="20" height="3" rx="1" :fill="item.color" />
        </svg>
        <span>{{ item.label }}</span>
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
import { PROTOCOL_COLOR_LEGEND } from "../../utils/protocolColors"
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

.public-ip-color {
  background-color: #2196f3;
}

.private-ip-color {
  background-color: #4caf50;
}

.protocol-color-swatch {
  width: 20px;
  height: 3px;
  margin-right: 8px;
  flex-shrink: 0;
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
