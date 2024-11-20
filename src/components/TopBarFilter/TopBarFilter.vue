<template>
    <div class="to-bar-filter">
      <label for="protocol-select">Filtrer IPv6:</label>
      <input type="checkbox" id="ipv6-filter" @change="toggleIPv6Filter" />
    </div>
</template>
  
  <script>
import { invoke } from '@tauri-apps/api/core'
  export default {
    methods: {
      toggleIPv6Filter() {
        invoke("toggle_ipv6_filter")
      },
    },
  };
  </script>
  
  <style scoped>
.to-bar-filter {
    display: flex;
    width: 100%;
    align-items: center;
    padding: 10px;
    color: #f2f2f2;
    background-color: #2A2A2A;
    border-radius: 5px;
    margin-bottom: 10px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* Ombre */
}

.to-bar-filter label {
    margin-right: 10px;
    font-weight: bold;
}

#ipv6-filter {
    accent-color: #3a80cf; /* This sets the color of the checkbox */
}

  </style>
  