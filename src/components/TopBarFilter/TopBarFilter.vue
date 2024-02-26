<template>
    <div class="to-bar-filter">
      <label for="protocol-select">Protocole:</label>
      <select id="protocol-select" v-model="selectedProtocol" @change="applyFilter">
        <option value="">Tous</option>
        <option v-for="protocol in protocols" :key="protocol" :value="protocol">
          {{ protocol }}
        </option>
      </select>
  
      <template v-if="showTimeFilter">
        <label for="start-time">Heure de début:</label>
        <input type="datetime-local" id="start-time" v-model="startTime" />
  
        <label for="end-time">Heure de fin:</label>
        <input type="datetime-local" id="end-time" v-model="endTime" />
      </template>
  
      <button @click="applyFilter">Appliquer</button>
    </div>
  </template>
  
  <script>
  export default {
    props: {
      protocols: Array,
      applyFilter: Function,
      currentProtocol: String,
      startTime: String,
      endTime: String,
    },
    data() {
      return {
        selectedProtocol: this.currentProtocol || "", // Default to current protocol
        startTime: this.startTime || "", // Default to initial start time
        endTime: this.endTime || "", // Default to initial end time
        showTimeFilter: false, // Flag to control optional time filter visibility
      };
    },
    methods: {
      applyFilter() {
        this.$emit("filter-applied", {
          protocol: this.selectedProtocol,
          startTime: this.startTime, // Pass start time if applicable
          endTime: this.endTime, // Pass end time if applicable
        });
      },
    },
  };
  </script>
  
  <style scoped>
  /* Style the component as needed */
  </style>
  