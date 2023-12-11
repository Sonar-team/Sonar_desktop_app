<template>
  <div class="capture-container">
    <div class="header">
      <h1 class="title-capture">Choisir une interface réseau</h1>
    </div>
    <div class="content">
      <select v-model="selectedNetInterface">
        <option disabled value="all">Toutes</option>
        <option v-for="netInterface in netInterfaces" :key="netInterface" :value="netInterface">
          {{ netInterface }}
        </option>
      </select>
      <router-view></router-view>
    </div>

    <div class="header">
      <h1 class="title-capture">Choisir une confidentialité</h1>
    </div>
    <div class="content">
      <select v-model="confidentialite">
        <option v-for="confidentialité in confidentialités" :key="confidentialité">
          {{ confidentialité }}
        </option>
      </select>
      <router-view></router-view>
    </div>

    <div class="header">
      <h1 class="title-capture">Entrer le nom de la machine</h1>
    </div>
    <div class="content">
      <input v-model="installationName" placeholder="Nom de l'installation" />
    </div>

    <div class="header">
      <h1 class="title-capture">Entrer le temps de relevé</h1>
    </div>
    <div class="content">
      <input v-model="time" type="time" placeholder="HH:MM" />
    </div>
  <button @click="goToAnalysePage">Lancer le relevé</button>

  </div>
</template>

  
<script>
import { invoke } from '@tauri-apps/api/tauri';

export default {
  data() {
    return {
      netInterfaces: [],
      confidentialités: ["NP","DR","TS","S"],
      confidentialite: '',
      time: '',
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
  text-align: center;

  color: white; /* White text color for the entire container */
}

.title-capture {
  font-family: Oxanium;
  font-size: 2em;
  margin: 0 0 10px 0;
  text-align: left;
}

.content {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

select, input {
  color: black; /* Dark text for input and select content for readability */
  background-color: white; /* Light background for inputs and selects */
  padding: 8px;
  margin-bottom: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

select:hover, input:hover {
  border-color: #0BA4DB; /* Hover effect for inputs */
}

button {
  padding: 10px 15px;
  background-color: #333; /* Dark background for buttons */
  color: white; /* White text for buttons */
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background-color: #555; /* Slightly lighter hover state for buttons */
}

/* Remove unused styles related to .interface-list and .interface-item */
</style>
  
  