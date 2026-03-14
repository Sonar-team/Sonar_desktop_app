<template>
  <Teleport to="body">
    <div class="quit-backdrop" v-if="visible" @keydown.esc="cancel">
      <div class="quit-dialog" role="dialog" aria-modal="true" aria-labelledby="quit-title">

        <div class="quit-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="9"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16" stroke-width="2"/>
          </svg>
        </div>

        <div class="quit-body">
          <p id="quit-title" class="quit-title">Quitter Sonar ?</p>
          <p class="quit-sub">Les données non sauvegardées seront perdues.</p>
        </div>

        <div class="quit-footer">
          <button class="btn-cancel" @click="cancel">Annuler</button>
          <button class="btn-confirm" @click="confirm" ref="confirmBtn">Quitter</button>
        </div>

      </div>
    </div>
  </Teleport>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'QuitDialog',
  props: {
    visible: { type: Boolean, default: false },
  },
  emits: ['confirm', 'cancel'],
  watch: {
    visible(val) {
      if (val) {
        this.$nextTick(() => {
          (this.$refs.confirmBtn as HTMLButtonElement)?.focus()
        })
      }
    },
  },
  methods: {
    confirm() { this.$emit('confirm') },
    cancel()  { this.$emit('cancel') },
  },
})
</script>

<style scoped>
.quit-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.quit-dialog {
  background: #363648;
  border: 1px solid #4a4a60;
  border-radius: 10px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  width: 340px;
  padding: 28px 24px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  animation: pop-in 0.2s cubic-bezier(0.25, 1, 0.5, 1);
}

@keyframes pop-in {
  from { transform: scale(0.92) translateY(6px); opacity: 0; }
  to   { transform: scale(1)    translateY(0);  opacity: 1; }
}

.quit-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: rgba(180, 80, 80, 0.12);
  border: 1px solid rgba(180, 80, 80, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #b46060;
  flex-shrink: 0;
}
.quit-icon svg {
  width: 20px;
  height: 20px;
}

.quit-body {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.quit-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #c8c8dc;
  letter-spacing: 0.02em;
}

.quit-sub {
  margin: 0;
  font-size: 12px;
  color: #686880;
}

.quit-footer {
  display: flex;
  gap: 8px;
  width: 100%;
  margin-top: 8px;
}

.btn-cancel {
  flex: 1;
  padding: 8px 14px;
  background: transparent;
  border: 1px solid #484860;
  border-radius: 6px;
  color: #808096;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-cancel:active { transform: scale(0.96); }
.btn-cancel:hover {
  border-color: #60607a;
  color: #a8a8be;
}

.btn-confirm {
  flex: 1;
  padding: 8px 14px;
  background: #582828;
  border: 1px solid #703838;
  border-radius: 6px;
  color: #d08888;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-confirm:hover {
  background: #6a3030;
  border-color: #884040;
  color: #e0a0a0;
}
.btn-confirm:active { transform: scale(0.96); }
.btn-confirm:focus {
  outline: none;
  box-shadow: 0 0 0 2px rgba(180, 80, 80, 0.4);
}
</style>
