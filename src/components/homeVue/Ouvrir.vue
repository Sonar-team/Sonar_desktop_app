<template>
  <div class="capture-container">
    <div class="header">
      <h1 class="title-capture">Ouvrir le projet</h1>
    </div>
    <div class="content">
      <div class="interface-list">
        <div
          class="interface-item"
          v-for="netInterface in listeFiles"
          :key="netInterface"
          @click="handleClick(netInterface)"
        >
          {{ netInterface }}
        </div>
      </div>
    </div>
  </div>
</template>
    
<script>
import { invoke } from '@tauri-apps/api/tauri';

export default {
  data() {
    return {
      netInterfaces: [],
      listeFiles: ["/home/cyprien/Documents/rust_projects/pcap_rust/test2.csv (88 Kb)",
    "/home/cyprien/Documents/rust_projects/pcap_rust/test3.csv (108 Kb)"],
    };
  },
  methods: {
      async handleClick(netInterface) {
      console.log(`You clicked on interface: ${netInterface}`);
      await invoke('print_selected_interface', { interface_name: netInterface });
      // Here you can put any code to handle the button click.
    }
  },
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
color: #333; /* Adjust the text color as needed */
border: none;
cursor: pointer;
transition: background-color 0.3s; /* Smooth transition for hover effect */
}

.interface-item:hover {
background-color: #0BA4DB; /* Adjust hover state background color as needed */
}

</style>