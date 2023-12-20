<template>
    <h1 class="titre-relevé">{{ getCurrentDate()+ '_' + niveauConfidentialite  + '_' + installationName }}</h1>
</template>


<script>
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
  },
  mounted() {
    console.log("Leftnvabar mounted");

    this.installationName = this.$route.params.installationName;

    this.niveauConfidentialite = this.$route.params.confidentialite;
  }
}
</script>