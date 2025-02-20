<template>
  <div class="image-container">
    <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">
  </div>
  <div class="center-container">
    <div class="capture-container">
      <div class="header">
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
        <h1 class="title-capture">3. Entrer le nom de la matrice</h1>
      </div>
      <div class="content">
        <input v-model="installationName" placeholder="Nom de la matrice" :class="{ 'invalid': !validation.installationNameValid }" @input="validateInstallationName" />
      </div>

      <div class="header">
        <h1 class="title-capture">4. Entrer la durée de relevé</h1>
      </div>
      <div class="content">
        <input v-model="time" type="text" step="1" placeholder="HH:MM:SS" :class="{ 'invalid': !validation.timeValid }" @input="validateTime" />
      </div>
        <button @click="goToAnalysePage">Lancer le relevé</button>
    </div>
  </div>
</template>
  
<script>
import { invoke } from '@tauri-apps/api/core';
import { message } from '@tauri-apps/plugin-dialog';
import { trace, attachConsole } from "@tauri-apps/plugin-log";

export default {
  data() {
    return {
      netInterfaces: [],
      confidentialités: ["NP","DR","TS","S","S-SF"],
      selectedNetInterface: '',
      confidentialite: 'NP',
      installationName: '',
      time: '48:00:00',
      currentTime: '',
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
    // Use regex or manual splitting to validate input
    const parts = this.time.split(':');
    if (parts.length === 3) {
      const hours = parseInt(parts[0], 10);
      const minutes = parseInt(parts[1], 10);
      const seconds = parseInt(parts[2], 10);
      this.validation.timeValid = 
        hours >= 0 && hours <= 48 &&
        minutes >= 0 && minutes < 60 &&
        seconds >= 0 && seconds < 60;
    } else {
      this.validation.timeValid = false;
    }
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
      if (interfaces.length > 0) {
        this.selectedNetInterface = interfaces[interfaces.length - 1]; // Set the last item as default
      }
    }).catch(error => {
      console.error("Failed to load interfaces:", error);
    });
    invoke('get_hostname_to_string').then((hostname) => {
      this.installationName = hostname;
    }).catch(error => {
      console.error("Failed to load hostname:", error);
    });
    detach();
  }
};
</script>

<style scoped>

.image-container {
  display: flex;
  justify-content: center; /* Centre horizontalement */
  align-items: center; /* Centre verticalement (si nécessaire) */
}
.center-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 70vh;
  border-radius: 15px;
  padding: 20px;
}

.capture-container {
  border: 2px solid #3a3a3a; /* Bordure plus sombre */
  border-radius: 10px;
  padding: 20px;
  width: 60%;
  max-width: 600px;
  text-align: center;
  color: #FFF; /* Texte en blanc */
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  background-color: #1a1a1a; /* Fond plus sombre */
}


.title-capture {
  font-size: 20px;
  margin: 10px 0;
  text-align: center;
  color: #bacbfa;
}

.content {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%; /* Utilisation de la largeur complète */
}

select, input {
  font-size: 1.5em;
  color: #FFF; /* Texte en blanc */
  background-color: #333; /* Fond plus sombre pour les champs */
  padding: 12px;
  margin-bottom: 15px;
  border: 2px solid #555; /* Bordure plus sombre */
  border-radius: 5px;
  width: 50%;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
}

.invalid {
  border-color: #e92525; /* Couleur rouge pour les champs invalides */
  border-width: 5px;
}

select:hover, input:hover {
  border-color: #cbdee5; /* Couleur de survol */
}

button {
  padding: 12px 20px;
  background-color: #444; /* Fond du bouton plus sombre */
  color: #FFF; /* Texte en blanc */
  border-radius: 5px;
  cursor: pointer;
  font-size: 1.2em;
}

button:hover {
  background-color: #555; /* Couleur de survol */
}

/* Responsive Design */
@media (max-width: 768px) {
  .content {
    width: 95%; /* Adaptation pour les écrans plus petits */
  }
}


</style>