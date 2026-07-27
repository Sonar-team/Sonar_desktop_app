<!--
  Indicateur d'occupation du canal capture→processing (backpressure),
  alimenté par l'événement ChannelCapacityPayload.
-->
<template>
    <div class="channel-status">
      <p :title="progress + '%'">🚨:</p>
      <p> {{ progress }}%</p>
      <progress
        class="channel-progress"
        :class="{ 'channel-progress--backpressure': backpressure }"
        :title="backpressure ? 'Canal saturé : le pipeline applique la backpressure' : undefined"
        :value="progress"
        max="100"
        aria-label="Occupation du canal de traitement"
      ></progress>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent } from 'vue'
  import { useCaptureStore } from '../../../store/capture'
  import type { CaptureEvent } from '../../../types/capture'

  // Dérivé du contrat généré (#142) plutôt que retapé à la main : une
  // interface locale ne suivrait pas silencieusement un ajout/retrait de
  // champ (`session_id`, `backpressure`) côté Rust, la vérification
  // bivariante de TypeScript sur les méthodes acceptant un callback plus
  // étroit que le vrai payload ne le signalerait pas.
  type ChannelPayload = Extract<CaptureEvent, { event: 'channelCapacityPayload' }>['data']

  export default defineComponent({
    data() {
      return {
        progress: 0,
        backpressure: false,
        unlisten: undefined as undefined | (() => void),
      }
    },
    mounted() {
      const captureStore = useCaptureStore()
      this.unlisten = captureStore.onChannelCapacityPayload((payload: ChannelPayload) => {
        const { channelSize, currentSize, backpressure } = payload

        const computed = channelSize > 0
          ? Math.min(100, (currentSize * 100) / channelSize)
          : 0
        this.progress = Math.round(computed)
        this.backpressure = backpressure
      })
    },
    beforeUnmount() {
      this.unlisten?.()
    }
  })
  </script>
  
  
  <style scoped>
  .channel-status {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .channel-progress {
    appearance: none;
    -webkit-appearance: none;
    width: 100px; /* Taille fixe de la barre */
    height: 3px;
    border: 0;
    background-color: #f7f7f7;
  }

  .channel-progress::-webkit-progress-bar {
    background-color: #f7f7f7;
  }

  .channel-progress::-webkit-progress-value {
    background-color: #e79a6e;
  }

  .channel-progress::-moz-progress-bar {
    background-color: #e79a6e;
  }

  .channel-progress--backpressure::-webkit-progress-value {
    background-color: #ff5252;
  }

  .channel-progress--backpressure::-moz-progress-bar {
    background-color: #ff5252;
  }
  </style>
  
