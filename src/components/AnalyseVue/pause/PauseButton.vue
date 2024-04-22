<template>
    <button @click="pause">Pause</button>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri';

export default {
  name: 'PauseButton',
  methods: {
    async pause() {
        console.log("Pause button clicked"); // Pour vérifier que le clic est bien capturé

        this.$emit('click'); // Émet un événement click, utile si le parent doit réagir aussi
        console.log("Événement click émis");

        // Tente d'invoquer la commande 'toggle_pause' et d'attendre sa réponse
        console.log("Tentative d'invocation de la commande 'toggle_pause'");
        const message = await invoke('toggle_pause')
          .then((message) => {
              console.log("Réponse reçue de 'toggle_pause':", message);
              return message; // S'assure que le message est renvoyé pour une utilisation future
          })
          .catch((error) => {
              console.error("Erreur lors de l'invocation de 'toggle_pause':", error);
              throw error; // Permet de propager l'erreur pour une gestion plus avancée si nécessaire
          });

        // Si tu as besoin de faire quelque chose avec `message` après que le .then() ait fini
        // Note que cela ne sera exécuté que si aucun erreur n'est attrapée
        console.log("Message final:", message);
    }
  }
}
</script>
  
<style scoped>
    button {
        padding: 10px 20px;
        font-size: 16px;
        cursor: pointer;
        background-color: #f0f0f0;
        border: 1px solid #d3d3d3;
        border-radius: 5px;
        transition: background-color 0.3s;
    }

    button:hover {
        background-color: #e9e9e9;
    }
</style>
