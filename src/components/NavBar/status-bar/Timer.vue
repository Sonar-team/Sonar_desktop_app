<template>
  <div class="timer" :title="title">
    <!-- clock icon -->
    <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
      <circle cx="6" cy="6" r="4.5"/>
      <path d="M6 3.5V6l1.5 1.5"/>
    </svg>
    <span class="timer-val">{{ formattedTime }}</span>
  </div>
</template>

<script>
import { watch } from 'vue'
import { useCaptureStore } from '../../../store/capture'

export default {
  name: 'Timer',
  props: {
    // Title for the tooltip
    title: {
      type: String,
      default: 'Durée d\'exécution'
    },
    // Icon to display before the time
    icon: {
      type: String,
      default: '⏱️'
    }
  },
  data() {
    return {
      startTime: Date.now(),
      elapsedTime: 0,
      timer: null
    }
  },
  computed: {
    formattedTime() {
      const totalSeconds = Math.floor(this.elapsedTime / 1000)
      const minutes = Math.floor(totalSeconds / 60)
      const seconds = totalSeconds % 60
      return `${minutes}:${seconds.toString().padStart(2, '0')}`
    },
    isRunning() {
      const captureStore = useCaptureStore()
      return captureStore.isRunning
    }
  },
  watch: {
    isRunning(newVal) {
      if (newVal) {
        this.startTimer()
      } else {
        this.stopTimer()
      }
    }
  },
  mounted() {
    // Start timer if capture is already running when component mounts
    if (this.isRunning) {
      this.startTimer()
    }
  },
  beforeDestroy() {
    this.stopTimer()
  },
  methods: {
    startTimer() {
      this.startTime = Date.now()
      this.elapsedTime = 0
      this.timer = setInterval(() => {
        this.elapsedTime = Date.now() - this.startTime
      }, 1000)
    },
    stopTimer() {
      if (this.timer) {
        clearInterval(this.timer)
        this.timer = null
      }
    }
  }
}
</script>

<style scoped>
.timer {
  display: flex;
  align-items: center;
  gap: 4px;
  margin: 0;
}

.stat-icon {
  width: 11px;
  height: 11px;
  flex-shrink: 0;
  color: #585870;
}

.timer-val {
  font-family: monospace;
  font-size: 11px;
  color: #6a6a80;
  min-width: 32px;
}
</style>