<!-- Légende du graphe, alimentée par les mêmes sources que le rendu Sigma. -->
<template>
  <aside
    class="legend-container"
    :class="{ 'is-collapsed': !expanded }"
    aria-label="Légende du graphe réseau"
  >
    <div class="legend-heading">
      <h3>Légende du graphe</h3>
      <button
        class="legend-toggle"
        type="button"
        :aria-expanded="expanded"
        aria-controls="graph-legend-content"
        @click="expanded = !expanded"
      >
        {{ expanded ? 'Réduire' : 'Afficher' }}
      </button>
    </div>

    <div v-show="expanded" id="graph-legend-content">
      <section class="legend-section">
        <h4>Nœuds</h4>
        <div class="legend-grid">
          <div v-for="item in NODE_COLOR_LEGEND" :key="item.kind" class="legend-item">
            <svg class="node-color-swatch" viewBox="0 0 20 20" aria-hidden="true">
              <circle
                cx="10"
                cy="10"
                r="7"
                :fill="item.color"
                :stroke="item.borderColor"
                stroke-width="2"
              />
            </svg>
            <span>{{ item.label }}</span>
          </div>
        </div>

        <div class="legend-item legend-detail">
          <svg class="node-scale-swatch" viewBox="0 0 190 170" aria-hidden="true">
            <circle cx="10" cy="85" :r="rendering.nodeSize.min" />
            <circle cx="105" cy="85" :r="rendering.nodeSize.max" />
          </svg>
          <span>
            Taille : volume cumulé des flux reliés, échelle logarithmique ({{
              formatLegendNumber(rendering.nodeSize.min)
            }}
            à {{ formatLegendNumber(rendering.nodeSize.max) }}).
          </span>
        </div>

        <div class="legend-item legend-detail">
          <svg class="node-color-swatch" viewBox="0 0 20 20" aria-hidden="true">
            <circle
              cx="10"
              cy="10"
              r="7"
              fill="currentColor"
              fill-opacity="0.25"
              :stroke="rendering.macConflictBorderColor"
              stroke-width="3"
            />
          </svg>
          <span>Bordure rouge : plusieurs MAC unicast observées pour la même IP.</span>
        </div>

        <div class="legend-item legend-detail">
          <svg class="node-color-swatch" viewBox="0 0 20 20" aria-hidden="true">
            <circle
              cx="10"
              cy="10"
              r="7"
              fill="currentColor"
              fill-opacity="0.25"
              :stroke="rendering.duplicateIpBorderColor"
              stroke-width="3"
            />
          </svg>
          <span>Bordure ambre : même IP présente sur plusieurs VLAN.</span>
        </div>
      </section>

      <section class="legend-section">
        <h4>Arêtes — couleur par protocole</h4>
        <div class="legend-grid protocol-grid">
          <div v-for="item in PROTOCOL_COLOR_LEGEND" :key="item.label" class="legend-item">
            <svg class="protocol-color-swatch" viewBox="0 0 20 7" aria-hidden="true">
              <rect y="2" width="20" height="3" rx="1" :fill="item.color" />
            </svg>
            <span>{{ item.label }}</span>
          </div>
        </div>

        <div class="legend-item legend-detail">
          <svg class="edge-scale-swatch" viewBox="0 0 40 14" aria-hidden="true">
            <line x1="1" y1="3" x2="39" y2="3" :stroke-width="rendering.edgeSize.min" />
            <line x1="1" y1="10" x2="39" y2="10" :stroke-width="rendering.edgeSize.max" />
          </svg>
          <span>
            Épaisseur : octets du flux, échelle logarithmique ({{
              formatLegendNumber(rendering.edgeSize.min)
            }}
            à {{ formatLegendNumber(rendering.edgeSize.max) }}).
          </span>
        </div>

        <div class="legend-item legend-detail">
          <svg class="edge-arrow-swatch" viewBox="0 0 32 12" aria-hidden="true">
            <line x1="1" y1="6" x2="25" y2="6" />
            <path d="M 24 2 L 31 6 L 24 10 Z" />
          </svg>
          <span>Flèche : sens du premier paquet observé.</span>
        </div>
        <p class="legend-note">
          Les flux retour sont agrégés sur la même arête, sans style distinct.
        </p>
      </section>

      <section class="legend-section">
        <h4>Zoom et ports</h4>
        <div class="legend-item legend-detail">
          <span class="zoom-badge">≥ {{ formatLegendNumber(rendering.edgeLabelZoom) }}×</span>
          <span>Protocole affiché sur l’arête.</span>
        </div>
        <div class="legend-item legend-detail">
          <span class="zoom-badge">≥ {{ formatLegendNumber(rendering.portLabelZoom) }}×</span>
          <span>Ports service ; « … » signale les ports dynamiques.</span>
        </div>
      </section>

      <section class="legend-section">
        <h4>Tunnels</h4>
        <div class="legend-item legend-detail">
          <svg class="tunnel-swatch" viewBox="0 0 34 18" aria-hidden="true">
            <line x1="1" y1="5" x2="33" y2="5" />
            <line x1="1" y1="14" x2="33" y2="14" :stroke="rendering.tunnelDimEdgeColor" />
            <circle cx="4" cy="14" r="3" :fill="rendering.tunnelDimNodeColor" />
          </svg>
          <span>Survol : famille liée conservée, reste estompé. Clic : épingler ou libérer.</span>
        </div>
      </section>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { PROTOCOL_COLOR_LEGEND } from '../../utils/protocolColors';
import { GRAPH_RENDERING_LEGEND, NODE_COLOR_LEGEND } from './graph/graphLegend';

const rendering = GRAPH_RENDERING_LEGEND;
const compactViewport =
  typeof window === 'undefined' ? null : window.matchMedia('(max-width: 520px)');
const expanded = ref(!(compactViewport?.matches ?? false));

function syncLegendToViewport(event: MediaQueryListEvent) {
  expanded.value = !event.matches;
}

onMounted(() => compactViewport?.addEventListener('change', syncLegendToViewport));
onBeforeUnmount(() => compactViewport?.removeEventListener('change', syncLegendToViewport));

function formatLegendNumber(value: number): string {
  return value.toLocaleString('fr-FR', { maximumFractionDigits: 1 });
}
</script>

<style scoped>
.legend-container {
  position: absolute;
  top: 10px;
  right: 10px;
  width: min(330px, calc(100% - 20px));
  max-height: calc(100% - 20px);
  overflow-y: auto;
  box-sizing: border-box;
  background-color: #1a1a1a;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  padding: 14px;
  color: #eaeaea;
  font-family: Arial, sans-serif;
  z-index: 5;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.legend-container.is-collapsed {
  width: auto;
  max-height: none;
}

.legend-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 8px;
  border-bottom: 1px solid #3a3a3a;
}

.legend-heading h3 {
  flex: 1;
  margin: 0;
  color: #fff;
  font-size: 16px;
  font-weight: 700;
}

.legend-toggle {
  padding: 3px 6px;
  border: 1px solid #555;
  border-radius: 4px;
  background: #242424;
  color: #fff;
  font: inherit;
  font-size: 10px;
  cursor: pointer;
}

.legend-toggle:focus-visible {
  outline: 2px solid #90caf9;
  outline-offset: 2px;
}

#graph-legend-content {
  padding-top: 14px;
}

.legend-section {
  margin-bottom: 14px;
}

.legend-section:last-child {
  margin-bottom: 0;
}

.legend-section h4 {
  margin: 0 0 8px;
  color: #bbb;
  font-size: 13px;
  font-weight: 600;
}

.legend-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 5px 10px;
  margin-bottom: 9px;
}

.legend-item {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 7px;
  color: #eaeaea;
  font-size: 11px;
  line-height: 1.3;
}

.legend-detail {
  margin-top: 7px;
}

.legend-note {
  margin: 5px 0 0 31px;
  color: #aaa;
  font-size: 10px;
  line-height: 1.3;
}

.node-color-swatch,
.protocol-color-swatch,
.edge-scale-swatch,
.edge-arrow-swatch,
.tunnel-swatch {
  width: 24px;
  flex: 0 0 24px;
}

.node-color-swatch {
  height: 20px;
}

.node-scale-swatch {
  width: 28px;
  height: 26px;
  flex: 0 0 28px;
  fill: #b0bec5;
  stroke: #78909c;
  stroke-width: 3;
}

.protocol-color-swatch {
  height: 7px;
}

.edge-scale-swatch,
.edge-arrow-swatch,
.tunnel-swatch {
  height: 16px;
  stroke: #eaeaea;
  fill: #eaeaea;
}

.edge-scale-swatch,
.edge-arrow-swatch {
  stroke-linecap: round;
}

.tunnel-swatch line:first-child {
  stroke-width: 3;
}

.tunnel-swatch line:nth-child(2) {
  stroke-width: 2;
}

.zoom-badge {
  min-width: 48px;
  padding: 2px 4px;
  border: 1px solid #555;
  border-radius: 3px;
  color: #fff;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

@media (max-width: 520px) {
  .legend-grid {
    grid-template-columns: 1fr;
  }
}
</style>
