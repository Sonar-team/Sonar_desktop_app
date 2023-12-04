<template>
  <div class="capture-container">
    <div class="header">
      <h1 class="title-capture">Lancer la Capture</h1>
    </div>
    <div class="content">
      <div class="interface-list">
        <div
          class="interface-item"
          v-for="netInterface in netInterfaces"
          :key="netInterface"
          @click="goToAnalysePage"
        >
          {{ netInterface }}
        </div>
        <router-view></router-view>
      </div>
    </div>
  </div>
</template>
  
<script>
import { invoke } from '@tauri-apps/api/tauri';

export default {
  data() {
    return {
      netInterfaces: []
    };
  },
  methods: {
      async handleClick(netInterface) {
      //console.log(`You clicked on interface: ${netInterface}`);
      goToAnalysePage();

    },
    goToAnalysePage() {
      this.$router.push("/analyse");
    }
  },
  mounted() {
    console.log("mounted capture");
    invoke('get_interfaces_tab').then((interfaces) => {
      this.netInterfaces = interfaces;
    });
  }
};
</script>
  
<style scoped>
.capture-container {
  margin: 20px;
  display: flex;
  flex-direction: column;
  text-align: left;
}

.title-capture {
  color: #888;
  font-family: Oxanium;
  font-size: 40px;
  font-style: normal;
  font-weight: 500;
  line-height: normal;  
  font-size: 2em; /* Adjust the font size as needed */
  color: #DB770B; /* Adjust the text color as needed */
  margin: 0 0 10px 0; /* Adjust the margin as needed */
  text-align: left;
}

.content {
  display: flex;
  flex-direction: column;
  align-items: flex-start; /* Aligns items to the start of the container */
}

.interface-list {
  list-style: none; /* Removes default list styling */
  padding: 0; /* Removes default padding */
  margin: 0; /* Removes default margin */
}

.interface-item {
  font-family: Oxanium; 
  color: #333; /* Adjust the text color as needed */
  border: none;
  cursor: pointer;
  transition: background-color 0.3s; /* Smooth transition for hover effect */
}

.interface-item:hover {
  background-color: #0BA4DB; /* Adjust hover state background color as needed */
}

</style>
  
  