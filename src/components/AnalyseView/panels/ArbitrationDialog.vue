<!--
  Dialogue d'arbitrage des conflits de labels : pour chaque clé (mac, ip)
  ayant reçu plusieurs labels à l'import, l'utilisateur choisit le label à
  garder (commande resolve_label_conflicts, transactionnelle).
  Props : conflicts. Événements : close, resolved.
-->
<template>
  <div class="container">
    <div class="center-container">
      <button type="button" class="btn image-btn cross" @click.prevent="$emit('close')" :disabled="resolving">❌</button>

      <h1 class="dialog-title">Arbitrage des conflits de labels</h1>
      <p class="text subtitle">
        {{ conflicts.length }} clé(s) (MAC, IP) reçoivent plusieurs labels. Choisis le label à conserver pour chacune.
      </p>

      <div class="conflict-list">
        <div v-for="(c, i) in conflicts" :key="c.mac + '|' + c.ip" class="conflict-card">
          <div class="conflict-key text">
            <span class="mono">{{ c.mac || '-' }}</span>
            <span class="sep">/</span>
            <span class="mono">{{ c.ip || '-' }}</span>
          </div>
          <label
            v-for="(cand, j) in candidates(c)"
            :key="j"
            class="choice"
            :class="{ selected: selected[i] === cand.label }"
          >
            <input type="radio" :name="'conflict-' + i" :value="cand.label" v-model="selected[i]" />
            <span class="choice-label text">{{ cand.label || '(vide)' }}</span>
            <span class="choice-lines">ligne{{ cand.lines.length > 1 ? 's' : '' }} {{ cand.lines.join(', ') }}</span>
          </label>
        </div>
      </div>

      <div class="actions">
        <button type="button" class="btn btn-clear" @click="$emit('close')" :disabled="resolving">Fermer</button>
        <button type="button" class="btn btn-open" @click="applyAll" :disabled="resolving">
          {{ resolving ? 'Application…' : 'Appliquer' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { info, error } from '@tauri-apps/plugin-log';
import { displayCaptureError } from '../../../errors/capture';
import { LabelConflictReport } from '../../../types/labels';

type Candidate = { label: string; lines: number[] };

export default defineComponent({
  name: 'ArbitrationDialog',
  emits: ['close', 'resolved'],
  props: {
    conflicts: {
      type: Array as PropType<LabelConflictReport[]>,
      required: true
    }
  },
  data() {
    return {
      // Label sélectionné par conflit (défaut : le label gardé = le premier).
      selected: this.conflicts.map((c) => c.kept[1]) as string[],
      resolving: false
    };
  },
  methods: {
    // Labels candidats pour une clé, dédupliqués, avec les lignes d'origine.
    candidates(c: LabelConflictReport): Candidate[] {
      const byLabel = new Map<string, number[]>();
      for (const [line, label] of [c.kept, ...c.dropped]) {
        const lines = byLabel.get(label) ?? [];
        lines.push(line);
        byLabel.set(label, lines);
      }
      return Array.from(byLabel.entries()).map(([label, lines]) => ({ label, lines }));
    },

    async applyAll() {
      this.resolving = true;
      try {
        // Un seul invoke pour tous les arbitrages : le backend les applique
        // dans une même section critique (transactionnel), une erreur ne
        // laisse pas la moitié des choix appliqués.
        const resolutions = this.conflicts.map((c, i) => ({
          mac: c.mac,
          ip: c.ip,
          label: this.selected[i],
        }));
        await invoke('resolve_label_conflicts', { resolutions });
        info(`Arbitrage : ${this.conflicts.length} conflit(s) résolu(s)`);
        this.$emit('resolved');
      } catch (err) {
        error(`Erreur arbitrage: ${err}`);
        await displayCaptureError(err);
      } finally {
        this.resolving = false;
      }
    }
  }
});
</script>

<style scoped>
.container {
  position: fixed;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.7);
  z-index: 1100;
}
.center-container {
  position: relative;
  display: flex;
  flex-direction: column;
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 1.5rem 2rem;
  width: 90%;
  max-width: 620px;
  max-height: 82%;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}
.text { color: whitesmoke; }
.mono { font-family: monospace; }
.dialog-title { color: whitesmoke; font-size: 1.6rem; margin: 0 0 0.4rem 0; text-align: center; }
.subtitle { text-align: center; color: #cfd2e0; margin: 0 0 1rem 0; }

.conflict-list { overflow-y: auto; padding-right: 0.3rem; }
.conflict-card {
  border: 1px solid #333;
  border-radius: 6px;
  padding: 0.6rem 0.8rem;
  margin-bottom: 0.7rem;
  background: #262636;
}
.conflict-key { margin-bottom: 0.45rem; font-weight: 600; }
.conflict-key .sep { margin: 0 0.4rem; color: #888; }

.choice {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.3rem 0.4rem;
  border-radius: 4px;
  cursor: pointer;
}
.choice:hover { background: #313152; }
.choice.selected { background: #2a3a44; }
.choice-label { flex: 1; }
.choice-lines { color: #9aa0b4; font-size: 0.8rem; }

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.8rem;
  margin-top: 1rem;
}
.btn {
  border-radius: 8px;
  border: 1px solid;
  padding: 0.5em 1.1em;
  font-size: 1em;
  font-family: inherit;
  color: whitesmoke;
  background-color: #181829;
  cursor: pointer;
}
.btn:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-open { border-color: #48bb78; }
.btn-open:enabled:hover { background-color: #313152; }
.btn-clear { border-color: #d8392b; }
.btn-clear:enabled:hover { background-color: #313152; }
.image-btn { background: none; border: none; padding: 0; cursor: pointer; }
.cross { position: absolute; top: 0.6rem; right: 0.6rem; font-size: 1rem; }
</style>
