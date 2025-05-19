<template>
    <div class="cpu">
    
      <img :src="icon" alt="CPU icon" class="cpu-icon" :title="title" />
      <p :title="title">{{ formattedCpuUsage }}</p>
    </div>
  </template>
  
  <script>  
  export default {
    name: 'Cpu',
    props: {
      title: {
        type: String,
        default: 'Utilisation du CPU'
      },
      icon: {
        type: String,
        default: 'src/assets/images/TablerCpu.svg'
      }
    },
    data() {
      return {
        cpuUsage: 0,
      }
    },
    computed: {
      formattedCpuUsage() {
        return `${this.cpuUsage.toFixed(2)}%`
      }
    },
    mounted() {

    },
    beforeUnmount() {
    }
  }
  </script>
  
  <style scoped>
  .cpu {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .cpu-icon {
    height: 20px;
    width: 20px;
  }
  </style>
  