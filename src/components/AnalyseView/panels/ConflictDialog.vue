<template>
  <div class="container" role="dialog" aria-modal="true">
    <div class="center-container">
      <button class="close-button" type="button" @click="windowClosed">Fermer</button>

      <h1 v-if="hasLabelConflicts" class="dialog-title">Conflits detectes</h1>
      <h1 v-else-if="hasInvalidFormats" class="dialog-title">MAC/IP invalides detectees</h1>
      <h1 v-else-if="conflictual_files.length > 0" class="dialog-title">Fichiers en conflit detectes</h1>
      <h1 v-else class="dialog-title">Avertissement</h1>

      <div class="panels">
        <div class="left-panel" aria-hidden="true">
          <div class="warning-mark">!</div>
        </div>

        <div class="right-panel">
          <section v-if="same_ip_diff_mac.length > 0" class="file-list">
            <h3 class="section-title">Conflits IP -> MAC</h3>
            <ul>
              <li v-for="([ip, refMac, firstFile, mac, secondFile], index) in same_ip_diff_mac" :key="`mac-${index}`">
                <span class="text">'{{ ip }}' (IP)</span>
                <span class="text indented">MAC: '{{ refMac }}' &lt;---- {{ shorten(firstFile, 60) }}</span>
                <span class="text indented">MAC: '{{ mac }}' &lt;---- {{ shorten(secondFile, 60) }}</span>
              </li>
            </ul>
          </section>

          <section v-if="same_ip_diff_label.length > 0" class="file-list">
            <h3 class="section-title">Conflits IP -> Label</h3>
            <ul>
              <li v-for="([ip, refLabel, firstFile, label, secondFile], index) in sortedLabelConflicts" :key="`label-${index}`">
                <span class="text">'{{ ip }}' (IP)</span>
                <span class="text indented">Label: '{{ refLabel }}' &lt;---- {{ shorten(firstFile, 60) }}</span>
                <span class="text indented">Label: '{{ label }}' &lt;---- {{ shorten(secondFile, 60) }}</span>
              </li>
            </ul>
          </section>

          <section v-if="invalid_mac.length > 0" class="file-list">
            <h3 class="section-title">MAC invalides</h3>
            <ul>
              <li v-for="([fileName, mac], index) in invalid_mac" :key="`invalid-mac-${index}`">
                <span class="text indented">MAC: '{{ mac }}' &lt;---- {{ shorten(fileName, 60) }}</span>
              </li>
            </ul>
          </section>

          <section v-if="invalid_ip.length > 0" class="file-list">
            <h3 class="section-title">IP invalides</h3>
            <ul>
              <li v-for="([fileName, ip], index) in invalid_ip" :key="`invalid-ip-${index}`">
                <span class="text indented">IP: '{{ ip }}' &lt;---- {{ shorten(fileName, 60) }}</span>
              </li>
            </ul>
          </section>

          <section v-if="conflictual_files.length > 0" class="file-list">
            <h3 class="section-title">Fichiers non importes</h3>
            <ul>
              <li v-for="(fileName, index) in conflictual_files" :key="`file-${index}`">
                <span class="text indented">{{ shorten(fileName, 100) }}</span>
              </li>
            </ul>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';

type ConflictRow = [string, string, string, string, string];
type InvalidFormatRow = [string, string];

export default defineComponent({
  name: 'ConflictDialog',
  emits: ['showConflictDialog'],
  props: {
    same_ip_diff_mac: {
      type: Array as PropType<ConflictRow[]>,
      required: true,
    },
    same_ip_diff_label: {
      type: Array as PropType<ConflictRow[]>,
      required: true,
    },
    invalid_mac: {
      type: Array as PropType<InvalidFormatRow[]>,
      required: true,
    },
    invalid_ip: {
      type: Array as PropType<InvalidFormatRow[]>,
      required: true,
    },
    conflictual_files: {
      type: Array as PropType<string[]>,
      required: true,
    },
  },
  computed: {
    hasLabelConflicts(): boolean {
      return this.same_ip_diff_mac.length > 0 || this.same_ip_diff_label.length > 0;
    },
    hasInvalidFormats(): boolean {
      return this.invalid_mac.length > 0 || this.invalid_ip.length > 0;
    },
    sortedLabelConflicts(): ConflictRow[] {
      return [...this.same_ip_diff_label].sort(([ipA], [ipB]) => ipA.localeCompare(ipB));
    },
  },
  methods: {
    windowClosed() {
      this.$emit('showConflictDialog', false);
    },
    shorten(value: string, maxLength: number): string {
      return value.length > maxLength ? `${value.slice(0, maxLength)}...` : value;
    },
  },
});
</script>

<style scoped>
.container {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  background-color: rgba(0, 0, 0, 0.7);
}

.center-container {
  position: relative;
  display: flex;
  flex-direction: column;
  width: min(100%, 1200px);
  max-height: calc(100vh - 4rem);
  padding: 2rem;
  overflow: hidden;
  background-color: #1e1e2e;
  border-radius: 8px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
}

.dialog-title {
  margin: 0 0 1rem;
  color: whitesmoke;
  font-size: 2rem;
  text-align: center;
}

.panels {
  display: flex;
  gap: 1.5rem;
  min-height: 0;
}

.left-panel {
  display: flex;
  flex: 0 0 140px;
  align-items: center;
  justify-content: center;
}

.warning-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 96px;
  height: 96px;
  color: #181829;
  background-color: #f6c945;
  border-radius: 50%;
  font-size: 4rem;
  font-weight: 800;
}

.right-panel {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 1rem;
  overflow-y: auto;
}

.file-list {
  width: 100%;
  padding: 0.75rem;
  background-color: #2d3748;
  border-radius: 4px;
}

.file-list ul {
  padding: 0;
  margin: 0;
}

.file-list li {
  display: flex;
  flex-direction: column;
  padding: 0.5rem;
  margin: 0.25rem 0;
  list-style: none;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9rem;
  word-break: break-word;
}

.section-title,
.text {
  color: whitesmoke;
}

.section-title {
  margin: 0 0 0.5rem;
}

.indented {
  display: block;
  padding-left: 2rem;
}

.close-button {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  padding: 0.45rem 0.75rem;
  color: whitesmoke;
  cursor: pointer;
  background-color: #181829;
  border: 1px solid #d8392b;
  border-radius: 8px;
}

.close-button:hover {
  background-color: #313152;
}

@media (max-width: 760px) {
  .container {
    padding: 1rem;
  }

  .center-container {
    padding: 1.5rem;
  }

  .panels {
    flex-direction: column;
  }

  .left-panel {
    flex-basis: auto;
  }

  .warning-mark {
    width: 64px;
    height: 64px;
    font-size: 3rem;
  }
}
</style>
