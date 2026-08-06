<!--
  Qualification du relevé (#154, tranche 2) : où et par quoi il a été capté.

  Ces informations n'existent pas dans les paquets — l'opérateur les déclare.
  L'arbitrage du 04/08/2026 place la saisie au moment de l'arrêt de la capture
  ou de l'enregistrement, à la manière de Wireshark, et non en configuration
  préalable : à l'arrêt, l'opérateur sait ce qu'il vient de relever.

  Les trois champs sont facultatifs. Valider sans rien saisir est une réponse
  légitime — un relevé non qualifié reste un relevé valide, et rien n'est
  inventé à sa place.
-->
<template>
  <div
    class="container"
    role="dialog"
    aria-modal="true"
    aria-labelledby="survey-context-title"
    @keydown.esc="cancel"
  >
    <div class="center-container" ref="panel" tabindex="-1">
      <h1 id="survey-context-title" class="dialog-title">Qualifier le relevé</h1>
      <p class="text intro">
        Où et par quoi ce relevé a-t-il été capté ? Ces informations ne sont
        pas dans les paquets : elles accompagnent la matrice et le projet pour
        que deux relevés de sites différents restent distinguables.
      </p>

      <form class="fields" @submit.prevent="confirm">
        <label class="field">
          <span class="text field-label">Site</span>
          <input
            ref="firstField"
            v-model="site"
            type="text"
            class="field-input"
            placeholder="Usine Nord — atelier 3"
            autocomplete="off"
          />
        </label>

        <label class="field">
          <span class="text field-label">Capteur</span>
          <input
            v-model="sensor"
            type="text"
            class="field-input"
            placeholder="Nom de la sonde ou de la machine"
            autocomplete="off"
          />
        </label>

        <label class="field">
          <span class="text field-label">Interface</span>
          <input
            v-model="interfaceName"
            type="text"
            class="field-input"
            placeholder="eth0"
            autocomplete="off"
          />
          <span class="text field-hint">
            Préremplie depuis l'interface d'écoute quand elle est connue.
          </span>
        </label>
      </form>

      <div class="actions">
        <button type="button" class="btn btn-clear" @click.prevent="cancel">
          Annuler
        </button>
        <button type="button" class="btn btn-add" @click.prevent="confirm">
          {{ confirmLabel }}
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import type { SurveyContext } from '../../../types/generated/SurveyContext';

export default defineComponent({
  name: 'SurveyContextDialog',
  emits: ['confirm', 'cancel'],

  props: {
    /** Contexte déjà connu, servant de valeurs par défaut. */
    context: {
      type: Object as PropType<SurveyContext>,
      default: () => ({}),
    },
    /** Libellé du bouton de validation, selon ce qui suit la saisie. */
    confirmLabel: {
      type: String,
      default: 'Valider',
    },
  },

  data() {
    return {
      site: this.context.site ?? '',
      sensor: this.context.sensor ?? '',
      // `interface` est un mot réservé en TypeScript : le champ local porte
      // un autre nom, la clé envoyée au backend reste `interface`.
      interfaceName: this.context.interface ?? '',
    };
  },

  mounted() {
    (this.$refs.panel as HTMLElement | undefined)?.focus();
    (this.$refs.firstField as HTMLInputElement | undefined)?.focus();
  },

  methods: {
    confirm() {
      // Les saisies vides ou blanches sont normalisées côté Rust par la
      // fonction du cœur : on n'en réimplémente pas une seconde version ici,
      // qui pourrait diverger du format publié.
      this.$emit('confirm', {
        site: this.site,
        sensor: this.sensor,
        interface: this.interfaceName,
      } satisfies SurveyContext);
    },
    cancel() {
      this.$emit('cancel');
    },
  },
});
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
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 2rem;
  width: 100%;
  max-width: 560px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.dialog-title {
  color: whitesmoke;
  font-size: 1.75rem;
  margin: 0 0 0.75rem 0;
  text-align: center;
}

.intro {
  margin: 0 0 1.5rem 0;
  line-height: 1.45;
}

.fields {
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.field-label {
  font-weight: 600;
}

.field-input {
  border-radius: 6px;
  border: 1px solid #3a3a52;
  background-color: #181829;
  color: whitesmoke;
  padding: 0.55rem 0.7rem;
  font-size: 1rem;
  font-family: inherit;
}

.field-input:focus {
  outline: 2px solid #2596be;
  outline-offset: 1px;
  border-color: #2596be;
}

.field-hint {
  color: #d5d7e2;
  font-size: 0.85rem;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  margin-top: 1.75rem;
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
  border-color: #d8392b;
}

.btn-clear:hover {
  background-color: #313152;
}

.btn-clear:active {
  background-color: #d8392b;
}

.btn-add {
  border-color: whitesmoke;
}

.btn-add:hover {
  border-color: #2596be;
  background-color: #313152;
}

.btn-add:active {
  border-color: #2596be;
  background-color: #2596be;
}

.text {
  color: whitesmoke;
}
</style>
