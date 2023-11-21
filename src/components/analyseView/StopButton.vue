<template>
    <button @click="stopAction">Arrêter</button>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
export default {
    name: "StopButton",
    methods: {
        stopAction() {
        // Logique pour arrêter une action
        console.log("Action arrêtée");
        invoke('save_to_csv');
        // Appel de la commande Tauri avec les données
        invoke('save_file_from_frontend');
        //invoke('stop_recording');
        }
    }
}
</script>

<style>
button {
    /* Styles de base pour le bouton d'arrêt */
    background-color: red;
    color: white;
    border: none;
    padding: 10px 20px;
    cursor: pointer;
    border-radius: 5px;
    font-weight: bold;
}
</style>
