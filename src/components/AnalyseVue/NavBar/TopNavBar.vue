<template>
    <div class="topbar">
        <h1 class="titre-relevé">{{ getCurrentDate()+ '_' + niveauConfidentialite  + '_' + installationName }}</h1>
        <div class="button-group">

          <button @click="emitChangeComponent('GraphComponent')">Vue Graphique</button>
          <button @click="emitChangeComponent('MatrixComponent')">Vue Matricielle</button>
          <button @click="emitChangeComponent('StatisticComponent')">Vue Statistique</button>

        </div>
    </div>
</template>
<style scoped>
/* ... existing styles ... */

.button-group {
  display: flex;
  justify-content: space-around; /* Adjust as needed */
  margin-top: 20px;
}

.button-group button {
  padding: 10px 15px;
  background-color: #333;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.button-group button:hover {
  background-color: #555;
}
.topbar {
    background-color: #444444;
}

.titre-relevé {
    text-align: center;
    color: aliceblue;
}
</style>

<script scoped>
export default {
  data() {
    return {
      tempsReleve: '01:00:00',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName:''
    };
  },
  methods: {
    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${this.padZero(now.getDate())}/${this.padZero(now.getMonth() + 1)}/${now.getFullYear()}`;
      return formattedDate;
    },

    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
    },
    emitChangeComponent(componentName) {
      this.$emit('change-component', componentName);
    }
  },
  mounted() {
    console.log("Leftnvabar mounted");

    this.installationName = this.$route.params.installationName;

    this.niveauConfidentialite = this.$route.params.confidentialite;
  }
}
</script>