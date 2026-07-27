<!--
  Dialogue des erreurs bloquantes d'import de labels : formats MAC/IP
  invalides, lignes malformées ou conflits entre lignes, avec le détail
  ligne par ligne remonté par le backend.
-->
<template>
  <div class="container">
    <div class="center-container">

      <h1 v-show="(same_ip_diff_mac.length > 0 || same_ip_diff_label.length > 0) && !resolved" class="dialog-title">Conflits détectés</h1>
      <h1 v-show="(same_ip_diff_mac.length > 0 || same_ip_diff_label.length > 0) && resolved" class="dialog-title">Conflits de labels résolus</h1>
      <h1 v-show="invalid_mac.length > 0 || invalid_ip.length > 0" class="dialog-title">MAC/IP invalide(s) détectée(s)</h1>
      <h1 v-show="invalid_lines.length > 0" class="dialog-title">Format de fichier invalide</h1>
      <p v-show="totalErrors > 0 && resolved" class="text error-count resolved-note">
        {{ totalErrors }} doublon{{ totalErrors > 1 ? "s" : "" }} de label résolu{{ totalErrors > 1 ? "s" : "" }} automatiquement (premier gardé). À arbitrer si besoin.
      </p>
      <p v-show="totalErrors > 0 && !resolved" class="text error-count">
        Total : {{ totalErrors }} erreur{{ totalErrors > 1 ? "s" : "" }} détectée{{ totalErrors > 1 ? "s" : "" }} dans le CSV
      </p>

      <div class="panels">
        <div class="left-panel">
          <img class="image" src="../../../assets/images/warning-sign.png"/>
        </div>
        <div class="right-panel">

          <div v-show="same_ip_diff_mac.length > 0 || same_ip_diff_label.length > 0" class="file-list">
              <ul v-show="same_ip_diff_mac.length > 0">
                <h3 class="text">Conflits IP -> MAC</h3>
                <li v-for="([lineA, lineB, ip, ref_mac, mac, rowA, rowB], index) in same_ip_diff_mac" :key="index">
                  <label>
                    <span class="text">'{{ ip }}'(IP) — lignes {{ lineA }} / {{ lineB }}:</span><br>
                    <span class="text indented">ligne {{ lineA }} MAC: '{{ ref_mac || "-" }}'</span>
                    <span class="text indented line-context">ligne {{ lineA }} complète : '{{ rowA }}'</span>
                    <span class="text indented">ligne {{ lineB }} MAC: '{{ mac || "-" }}'</span>
                    <span class="text indented line-context">ligne {{ lineB }} complète : '{{ rowB }}'</span>
                  </label>
                </li>
              </ul>
              <ul v-show="same_ip_diff_label.length > 0">
                <h3 class="text">Conflits IP -> Label</h3>
                <li v-for="([lineA, lineB, ip, ref_label, label, rowA, rowB], index) in same_ip_diff_label" :key="index">
                  <label>
                    <span class="text">'{{ ip }}'(IP) — lignes {{ lineA }} / {{ lineB }}:</span><br>
                    <span class="text indented">ligne {{ lineA }} Label: '{{ ref_label || "-" }}'</span>
                    <span class="text indented line-context">ligne {{ lineA }} complète : '{{ rowA }}'</span>
                    <span class="text indented">ligne {{ lineB }} Label: '{{ label || "-" }}'</span>
                    <span class="text indented line-context">ligne {{ lineB }} complète : '{{ rowB }}'</span>
                  </label>
                </li>
              </ul>
          </div>

          <div v-show="invalid_mac.length > 0 || invalid_ip.length > 0" class="file-list">
              <ul v-show="invalid_mac.length > 0">
                <h3 class="text">MAC invalides</h3>
                <li v-for="([line, mac, row], index) in shownInvalidMac" :key="index">
                  <label>
                    <span class="text indented">ligne {{ line }} — MAC: '{{ mac }}'</span>
                    <span class="text indented line-context">ligne complète : '{{ row }}'</span>
                  </label>
                </li>
                <li v-if="invalid_mac.length > maxShown" class="text indented truncated">
                  … et {{ invalid_mac.length - maxShown }} autre{{ invalid_mac.length - maxShown > 1 ? 's' : '' }} (corrigez les premières : souvent la même cause)
                </li>
              </ul>
              <ul v-show="invalid_ip.length > 0">
                <h3 class="text">IP invalides</h3>
                <li v-for="([line, ip, row], index) in shownInvalidIp" :key="index">
                  <label>
                    <span class="text indented">ligne {{ line }} — IP: '{{ ip }}'</span>
                    <span class="text indented line-context">ligne complète : '{{ row }}'</span>
                  </label>
                </li>
                <li v-if="invalid_ip.length > maxShown" class="text indented truncated">
                  … et {{ invalid_ip.length - maxShown }} autre{{ invalid_ip.length - maxShown > 1 ? 's' : '' }} (corrigez les premières : souvent la même cause)
                </li>
              </ul>
          </div>
          <div v-show="invalid_lines.length > 0" class="file-list">
            <ul>
              <h3 class="text">Lignes concernées :</h3>
              <li v-for="([line, value], index) in shownInvalidLines" :key="index">
                <label>
                  <span class="text indented">ligne {{ line }} : '{{ value }}'</span>
                </label>
              </li>
              <li v-if="invalid_lines.length > maxShown" class="text indented truncated">
                … et {{ invalid_lines.length - maxShown }} autre{{ invalid_lines.length - maxShown > 1 ? 's' : '' }}
              </li>
            </ul>
          </div>
          <p v-show="invalid_lines.length > 0 || invalid_mac.length > 0 || invalid_ip.length > 0" class="text">Format attendu : <code>mac, ip, label</code> — les colonnes suivantes sont ajoutées au label</p>
        </div>
      </div>

      <div>
          <button class="btn image-btn" @click.prevent="windowClosed">❌</button>
      </div>
     
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';

type ConflictRow = [number, number, string, string, string, string, string]
type FieldLineValue = [number, string, string]
type LineValue = [number, string]

export default defineComponent({
  name: 'ConflictDialog',
  emits: ['showConflictDialog'],

  props: {
    same_ip_diff_mac: {
      type: Array as PropType<ConflictRow[]>,
      required: true
    },
    same_ip_diff_label: {
      type: Array as PropType<ConflictRow[]>,
      required: true
    },
    invalid_mac: {
      type: Array as PropType<FieldLineValue[]>,
      required: true
    },
    invalid_ip: {
      type: Array as PropType<FieldLineValue[]>,
      required: true
    },
    invalid_lines: {
      type: Array as PropType<LineValue[]>,
      required: true
    },
    // true : les "conflits" ont été résolus automatiquement (import non
    // bloquant, premier gardé) et sont affichés à titre informatif.
    resolved: {
      type: Boolean,
      default: false
    }
  },

  data() {
    return {
      // Plafond d'erreurs détaillées par catégorie : au-delà (mauvais
      // fichier, encodage…), la liste complète noie le diagnostic — le
      // compteur « … et N autres » suffit.
      maxShown: 10,
    };
  },

  computed: {
    totalErrors(): number {
      return this.same_ip_diff_mac.length
        + this.same_ip_diff_label.length
        + this.invalid_mac.length
        + this.invalid_ip.length
        + this.invalid_lines.length;
    },
    shownInvalidMac(): FieldLineValue[] {
      return this.invalid_mac.slice(0, this.maxShown);
    },
    shownInvalidIp(): FieldLineValue[] {
      return this.invalid_ip.slice(0, this.maxShown);
    },
    shownInvalidLines(): LineValue[] {
      return this.invalid_lines.slice(0, this.maxShown);
    },
  },

  methods: {
    windowClosed() {
      this.$emit('showConflictDialog', false);
    },
  }
})
</script>

<style scoped>
.container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.7);
  z-index: 1000;
}

.center-container {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: stretch;
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 2rem;
  width: 100%;
  max-width: 1200px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.panels {
  display: flex;
  flex-direction: row;
  width: 100%;
  align-items: stretch;
}

.dialog-title {
  color: whitesmoke;
  font-size: 2rem;
  margin: 0 0 0.5rem 0;
  text-align: center;
}

.error-count {
  align-self: center;
  background-color: #3a2630;
  border: 1px solid #d8392b;
  border-radius: 4px;
  font-weight: 700;
  margin: 0 0 1rem 0;
  padding: 0.45rem 0.75rem;
}

/* Conflits résolus (non bloquants) : ton informatif plutôt qu'alerte. */
.error-count.resolved-note {
  background-color: #26333a;
  border-color: #2596be;
}

.left-panel {
  justify-content: center;
  width: 20%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.right-panel {
  width: 75%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}


.csv-group {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.file-group {
  display: flex;
  flex-direction:row;
  gap: 1rem;
  margin-bottom: 1.5rem;
  justify-content: center;
}

.btn {
  border-radius: 8px;
  border: 1px solid;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: whitesmoke;
  background-color: #181829;
  transition: border-color 0.25s, background-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  cursor: pointer;
}

.btn-clear {
  background-color: #181829;
  border-color: #d8392b;
  color: white;
}

.btn-clear:hover{
  background-color:#313152 ;
}

.btn-clear:active {
  background-color: #d8392b;
}

.btn-add {
  border-color: whitesmoke;
}

.btn-add:hover {
  border-color: #2596be;
  background-color:#313152 ;
}
.btn-add:active {
  border-color: #2596be;
  background-color: #2596be;
}

.image-btn {
  background: none;
  border:none;
  padding: 0;
  cursor: pointer;
  margin-left: auto;
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
}

.image-btn:hover {
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.image-btn:active {
  transform: translateY(1px) scale(0.99) translateZ(0);
  transition: transform 0.1s ease, background-color 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}


.image {
  width: 100%;
  max-width: 180px;
  height: auto;
  object-fit: contain;
  margin-right: auto;
}

.file-list label {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.text {
  color: whitesmoke
}

.indented {
  display: block;
  padding-left: 2rem;
}

.line-context {
  color: #d5d7e2;
  overflow-wrap: anywhere;
  white-space: normal;
  word-break: break-word;
}

.file-list {
  width: 100%;
  max-height: 250px;
  overflow-y: auto;
  background-color: #2d3748;
  border-radius: 4px;
  padding: 0.5rem;
  margin-bottom: 1.5rem;
}

.file-list li {
  list-style: none;
  padding: 0.5rem;
  margin: 0.25rem 0;
  border-radius: 4px;
  word-break: break-all;
  font-family: monospace;
  font-size: 0.9rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
