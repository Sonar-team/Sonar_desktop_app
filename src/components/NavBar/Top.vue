<template>
    <div class="toolbar">
      <button class="toolbar-button" @click="startCapture">
        Play
      </button>
      <button class="toolbar-button" @click="pauseCapture">
        Pause
      </button>
      <button class="toolbar-button" @click="stopCapture">
        Stop
      </button>
      <button class="toolbar-button" @click="saveCapture">
        Save
      </button>
      <button class="toolbar-button" @click="captureOnline">
        Capture
      </button>
    </div>
  </template>
  
  <script>
  export default {
    methods: {
      startCapture() {
        console.log('Capture started');
      },
      pauseCapture() {
        console.log('Capture paused');
      },
      stopCapture() {
        console.log('Capture stopped');
      },
      saveCapture() {
        console.log('Capture saved');
      },
      captureOnline() {
        console.log('Capture online mode enabled');
      }
    }
  };
  </script>
  
  <style scoped>
  .toolbar {
    display: flex;
    justify-content: space-around;
    align-items: center;
    background-color: #0b1b25;
    padding: 8px;
    border-radius: 5px;
  }
  
  .toolbar-button {
    background: white;
    border: none;
    padding: 10px;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.3s ease;
  }
  
  .toolbar-button:hover {
    background: #ddd;
  }
  </style>