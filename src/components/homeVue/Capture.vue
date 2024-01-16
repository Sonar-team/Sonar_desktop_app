<template>
  <div class="capture-container">
    <div class="header">
      <h1 class="title-capture">1. Choisir une interface réseau</h1>
      <h1 class="title-capture">1. Choisir une interface réseau</h1>
    </div>
    <div class="content">
      <select v-model="selectedNetInterface" :class="{ 'invalid': !validation.netInterfaceValid }" @change="validateNetInterface">
        <option v-for="netInterface in netInterfaces" :key="netInterface" :value="netInterface">
          {{ netInterface }}
        </option>
      </select>
      <router-view></router-view>
    </div>

    <div class="header">
      <h1 class="title-capture">2. Choisir une confidentialité</h1>
      <h1 class="title-capture">2. Choisir une confidentialité</h1>
    </div>
    <div class="content">
      <select v-model="confidentialite" :class="{ 'invalid': !validation.confidentialiteValid }" @change="validateConfidentialite">
        <option v-for="confidentialité in confidentialités" :key="confidentialité">
          {{ confidentialité }}
        </option>
      </select>
      <router-view></router-view>
    </div>

    <div class="header">
      <h1 class="title-capture">3. Entrer le nom de l'installation</h1>
      <h1 class="title-capture">3. Entrer le nom de l'installation</h1>
    </div>
    <div class="content">
      <input v-model="installationName" placeholder="Nom de l'installation" :class="{ 'invalid': !validation.installationNameValid }" @input="validateInstallationName" />
      <input v-model="installationName" placeholder="Nom de l'installation" :class="{ 'invalid': !validation.installationNameValid }" @input="validateInstallationName" />
    </div>

    <div class="header">
      <h1 class="title-capture">4. Entrer la durée de relevé</h1>
      <h1 class="title-capture">4. Entrer la durée de relevé</h1>
    </div>
    <div class="content">
      <input v-model="time" type="time" step="1" placeholder="HH:MM:SS" :class="{ 'invalid': !validation.timeValid }" @input="validateTime" />
      <input v-model="time" type="time" step="1" placeholder="HH:MM:SS" :class="{ 'invalid': !validation.timeValid }" @input="validateTime" />
    </div>
    <button @click="goToAnalysePage">Lancer le relevé</button>
    <button @click="goToAnalysePage">Lancer le relevé</button>
  </div>
</template>

  
<script>
import { invoke } from '@tauri-apps/api/tauri';
import { message } from '@tauri-apps/api/dialog';
import { trace, attachConsole } from "tauri-plugin-log-api";

export default {
  data() {
    return {
      netInterfaces: [],
      confidentialités: ["NP","DR","TS","S"],
      selectedNetInterface: '',
      selectedNetInterface: '',
      confidentialite: 'NP',
      installationName: '',
      installationName: '',
      time: '04:00:00',
      currentTime: '',
      validation: {
        netInterfaceValid: true,
        confidentialiteValid: true,
        installationNameValid: true,
        timeValid: true,
      },
      validation: {
        netInterfaceValid: true,
        confidentialiteValid: true,
        installationNameValid: true,
        timeValid: true,
      }
    };
  },
  methods: {
    validateNetInterface() {
      this.validation.netInterfaceValid = this.selectedNetInterface && this.netInterfaces.includes(this.selectedNetInterface);
    },
    
    validateConfidentialite() {
      this.validation.confidentialiteValid = this.confidentialités.includes(this.confidentialite);
    },
    
    validateInstallationName() {  this.validation.installationNameValid = this.installationName && this.installationName.trim().length > 0;
},

validateTime() {
  this.validation.timeValid = this.time && /^([0-1]?[0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]$/.test(this.time);
},

validateForm() {
  this.validateNetInterface();
  this.validateConfidentialite();
  this.validateInstallationName();
  this.validateTime();
  return this.validation.netInterfaceValid && this.validation.confidentialiteValid && this.validation.installationNameValid && this.validation.timeValid;
},
    validateNetInterface() {
      this.validation.netInterfaceValid = this.selectedNetInterface && this.netInterfaces.includes(this.selectedNetInterface);
    },
    
    validateConfidentialite() {
      this.validation.confidentialiteValid = this.confidentialités.includes(this.confidentialite);
    },
    
    validateInstallationName() {  this.validation.installationNameValid = this.installationName && this.installationName.trim().length > 0;
},

validateTime() {
  this.validation.timeValid = this.time && /^([0-1]?[0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]$/.test(this.time);
},

validateForm() {
  this.validateNetInterface();
  this.validateConfidentialite();
  this.validateInstallationName();
  this.validateTime();
  return this.validation.netInterfaceValid && this.validation.confidentialiteValid && this.validation.installationNameValid && this.validation.timeValid;
},

captureCurrentTime() {
  const now = new Date();
  this.currentTime = now.toISOString(); // Format the current time as needed
},

goToAnalysePage() {
  if (this.validateForm()) {
    this.captureCurrentTime();
    this.$router.push({
      name: 'Analyse',
      params: {
        netInterface: this.selectedNetInterface,
        confidentialite: this.confidentialite,
        installationName: this.installationName,
        time: this.time,
        currentTime: this.currentTime,
      },
    });
  } else {
    message('Remplissez les champs en rouges ...', { title: 'Champs non remplis', type: 'warning' });
  }
}
},
async mounted() {
  const detach = await attachConsole();
  trace("mounted capture");
  invoke('get_interfaces_tab').then((interfaces) => {
    this.netInterfaces = interfaces;
  });
  detach();
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
  font-size: 2em
;
margin: 0 0 10px 0;
text-align: left;
  font-size: 2em
;
margin: 0 0 10px 0;
text-align: left;
}

.content {
display: flex;
flex-direction: column;
align-items: flex-start;
display: flex;
flex-direction: column;
align-items: flex-start;
}

select, input {
color: black; /* Dark text for input and select content for readability /
background-color: white; / Light background for inputs and selects */
padding: 8px;
margin-bottom: 10px;
border: 1px solid #ddd;
border-radius: 4px;
color: black; /* Dark text for input and select content for readability /
background-color: white; / Light background for inputs and selects */
padding: 8px;
margin-bottom: 10px;
border: 1px solid #ddd;
border-radius: 4px;
}

.invalid {
border-color: red; /* Red border for invalid inputs */
}

.invalid {
border-color: red; /* Red border for invalid inputs */
}

select:hover, input:hover {
border-color: #0BA4DB; /* Hover effect for inputs */
border-color: #0BA4DB; /* Hover effect for inputs */
}

button {
padding: 10px 15px;
background-color: #333; /* Dark background for buttons /
color: white; / White text for buttons */
border: none;
border-radius: 4px;
cursor: pointer;
padding: 10px 15px;
background-color: #333; /* Dark background for buttons /
color: white; / White text for buttons */
border: none;
border-radius: 4px;
cursor: pointer;
}

button:hover {
background-color: #555; /* Slightly lighter hover state for buttons */
background-color: #555; /* Slightly lighter hover state for buttons */
}
</style>