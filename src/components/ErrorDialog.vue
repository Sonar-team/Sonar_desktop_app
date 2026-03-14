<template>
  <Teleport to="body">
    <div class="error-backdrop" v-if="visible">
      <div class="error-dialog" role="alertdialog" aria-modal="true" aria-labelledby="err-title">

        <!-- Header -->
        <div class="err-header">
          <div class="err-icon" :class="iconClass">
            <svg v-if="isCapability" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16" stroke-width="2"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="9"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16" stroke-width="2"/>
            </svg>
          </div>
          <div class="err-title-block">
            <p id="err-title" class="err-title">{{ title }}</p>
            <p class="err-kind">{{ kindLabel }}</p>
          </div>
          <button class="err-close" @click="close" title="Fermer">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <line x1="4" y1="4" x2="12" y2="12"/>
              <line x1="12" y1="4" x2="4" y2="12"/>
            </svg>
          </button>
        </div>

        <!-- Message -->
        <div class="err-body">
          <p class="err-message">{{ mainMessage }}</p>

          <!-- CAP_NET_RAW block -->
          <div class="err-cap-block" v-if="isCapability">
            <div class="cap-label">
              <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
                <rect x="1.5" y="1.5" width="11" height="11" rx="2"/>
                <path d="M4 7h6M4 4.5h4M4 9.5h3"/>
              </svg>
              Résolution — accorder les droits réseau au binaire
            </div>
            <div class="cap-command-wrap">
              <code class="cap-command">{{ capCommand }}</code>
              <button class="cap-copy" @click="copyCommand" :class="{ copied }" title="Copier">
                <svg v-if="!copied" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="4.5" y="4.5" width="8" height="8" rx="1.5"/>
                  <path d="M9.5 4.5V3a1.5 1.5 0 0 0-1.5-1.5H3A1.5 1.5 0 0 0 1.5 3v5A1.5 1.5 0 0 0 3 9.5h1.5"/>
                </svg>
                <svg v-else viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="2,7 5.5,10.5 12,3"/>
                </svg>
              </button>
            </div>
            <p class="cap-hint">
              Sur NixOS : remplacez <code>nom_du_binaire</code> par
              <code>target/debug/sonar</code> ou le chemin du binaire compilé.
            </p>
          </div>

          <!-- Raw detail (collapsible) -->
          <details class="err-detail" v-if="detail">
            <summary>Détail technique</summary>
            <pre>{{ detail }}</pre>
          </details>
        </div>

        <!-- Footer -->
        <div class="err-footer">
          <button class="btn-close" @click="close" ref="closeBtn">Fermer</button>
        </div>

      </div>
    </div>
  </Teleport>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export interface ErrorPayload {
  title: string
  kind: string
  message: string
  detail?: string
}

const CAP_KINDS = ['deviceListError', 'captureInitError']

export default defineComponent({
  name: 'ErrorDialog',
  data() {
    return {
      visible: false,
      title: '',
      kindLabel: '',
      mainMessage: '',
      detail: '',
      isCapability: false,
      capCommand: 'sudo setcap cap_net_raw,cap_net_admin=eip nom_du_binaire',
      copied: false,
    }
  },
  computed: {
    iconClass(): string {
      return this.isCapability ? 'icon-warn' : 'icon-err'
    },
  },
  mounted() {
    document.addEventListener('show-error', this.onShowError as EventListener)
  },
  beforeUnmount() {
    document.removeEventListener('show-error', this.onShowError as EventListener)
  },
  watch: {
    visible(val) {
      if (val) this.$nextTick(() => (this.$refs.closeBtn as HTMLButtonElement)?.focus())
    },
  },
  methods: {
    onShowError(e: CustomEvent<ErrorPayload>) {
      const p = e.detail
      this.title       = p.title
      this.kindLabel   = p.kind
      this.mainMessage = p.message
      this.detail      = p.detail ?? ''
      this.isCapability = CAP_KINDS.includes(p.kind) || p.message.toLowerCase().includes('cap_net')
      this.copied      = false
      this.visible     = true
    },
    close() {
      this.visible = false
    },
    async copyCommand() {
      try {
        await navigator.clipboard.writeText(this.capCommand)
        this.copied = true
        setTimeout(() => { this.copied = false }, 2000)
      } catch {}
    },
  },
})
</script>

<style scoped>
.error-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
}

.error-dialog {
  background: #363648;
  border: 1px solid #4a4a60;
  border-radius: 10px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  width: 480px;
  max-width: 96vw;
  display: flex;
  flex-direction: column;
  animation: pop-in 0.2s cubic-bezier(0.25, 1, 0.5, 1);
}

@keyframes pop-in {
  from { transform: scale(0.92) translateY(6px); opacity: 0; }
  to   { transform: scale(1)    translateY(0);   opacity: 1; }
}

/* Header */
.err-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 18px 18px 14px;
  border-bottom: 1px solid #4a4a60;
}

.err-icon {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.err-icon svg { width: 18px; height: 18px; }
.icon-err  { background: rgba(160, 60, 60, 0.14); border: 1px solid rgba(160,60,60,0.3); color: #c07070; }
.icon-warn { background: rgba(160, 120, 40, 0.14); border: 1px solid rgba(160,120,40,0.3); color: #b89050; }

.err-title-block { flex: 1; min-width: 0; }
.err-title {
  margin: 0 0 2px;
  font-size: 13px;
  font-weight: 600;
  color: #c8c8dc;
}
.err-kind {
  margin: 0;
  font-size: 11px;
  color: #585870;
  font-family: monospace;
}

.err-close {
  width: 26px; height: 26px;
  background: transparent; border: none; border-radius: 4px;
  color: #686884; cursor: pointer; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.15s, color 0.15s;
}
.err-close svg { width: 11px; height: 11px; }
.err-close:hover { background: #484860; color: #b0b0c8; }

/* Body */
.err-body {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.err-message {
  margin: 0;
  font-size: 12px;
  color: #9898b0;
  line-height: 1.6;
  word-break: break-word;
}

/* CAP block */
.err-cap-block {
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 6px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cap-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 500;
  color: #808096;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.cap-label svg { width: 12px; height: 12px; flex-shrink: 0; }

.cap-command-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #1e1e2c;
  border: 1px solid #383850;
  border-radius: 4px;
  padding: 8px 10px;
}

.cap-command {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
  color: #a0c0d8;
  word-break: break-all;
  user-select: all;
}

.cap-copy {
  width: 26px; height: 26px;
  background: transparent; border: none; border-radius: 4px;
  color: #686884; cursor: pointer; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.15s, color 0.15s;
}
.cap-copy svg { width: 13px; height: 13px; }
.cap-copy:hover { background: #383850; color: #b0b0c8; }
.cap-copy.copied { color: #60a870; }

.cap-hint {
  margin: 0;
  font-size: 11px;
  color: #585870;
  line-height: 1.5;
}
.cap-hint code {
  font-family: monospace;
  color: #7898a8;
  background: rgba(255,255,255,0.04);
  padding: 1px 4px;
  border-radius: 3px;
}

/* Detail */
.err-detail {
  font-size: 11px;
  color: #585870;
}
.err-detail summary {
  cursor: pointer;
  color: #686884;
  user-select: none;
  padding: 2px 0;
}
.err-detail summary:hover { color: #9090a8; }
.err-detail pre {
  margin: 6px 0 0;
  font-family: monospace;
  font-size: 11px;
  color: #686884;
  background: #2c2c3a;
  border: 1px solid #383850;
  border-radius: 4px;
  padding: 8px 10px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

/* Footer */
.err-footer {
  padding: 12px 18px 16px;
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid #4a4a60;
}

.btn-close {
  padding: 8px 20px;
  background: transparent;
  border: 1px solid #484860;
  border-radius: 6px;
  color: #808096;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, transform 0.1s;
}
.btn-close:hover  { border-color: #60607a; color: #a8a8be; }
.btn-close:active { transform: scale(0.96); }
.btn-close:focus  { outline: none; box-shadow: 0 0 0 2px rgba(90,90,128,0.35); }
</style>
