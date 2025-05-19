<template>
  <div class="timer">
    <p :title="title">{{ icon }}: {{ formattedTime }}</p>
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
  display: inline-block;
  font-size: 12px;
  color: #ffffff;
  margin: 0;
}
</style>