<template>
    <div class="custom-table-container">

      <table class="custom-table">
        <thead>
          <tr>
            <th v-for="header in headers" :key="header.key || header.value">
              {{ header.title }}
              

            </th>
          </tr>
        </thead>
      </table>
    </div>
  </template>
  
  <script>
  export default {
    data() {
      return {
        headers: [
          { title: 'MAC Source', value: 'mac_address_source' },
          { title: 'MAC Destination', value: 'mac_address_destination' },
          { title: 'Internet', value: 'l_3_protocol' },
          { title: 'IP Source', value: 'layer_3_infos.ip_source' },
          { title: 'Type', value: 'layer_3_infos.ip_destination_type' },
          { title: 'IP Destination', value: 'layer_3_infos.ip_destination' },
          { title: 'Transport', value: 'layer_3_infos.l_4_protocol' },
          { title: 'Port Source', value: 'layer_3_infos.layer_4_infos.port_source' },
          { title: 'Port Destination', value: 'layer_3_infos.layer_4_infos.port_destination' },
          { title: 'Application', value: 'layer_3_infos.layer_4_infos.l_7_protocol' },
          { title: 'Trame (o)', value: 'packet_size_total' },
          { title: 'Occ', value: 'count' },
        ],
      };
    },
    
  };
  </script>
  
  <style scoped>
  .custom-table-container {
    width: 100%;
    position: fixed;
    background-color: #1B1B1B;
    z-index: 2000;
    top: 40px;
    left: 50%;
    transform: translate(-50%, 0);
    opacity: 0.95;
    border-radius: 10px;
  }
  
  .custom-table {
    width: auto%;
    border-collapse: collapse;
    font-family: 'Courier New', Courier, monospace;
    font-size: 14px;
  }

  
  /* Couleur de fond pour l’en-tête */
  .custom-table thead {
    background-color: rgb(185, 33, 33);
  }
  
  /* Style des colonnes (en-tête) */
  .custom-table thead th {
    position: sticky;
    top: 0;
    z-index: 10;
    background-color: #1B1B1B;
    color: #ECF0F1;
    padding: 8px 20px; /* ⬅️ plus d’espace entre les colonnes */
    font-weight: bold;
    text-align: left;
  }
  
  /* Style pour le corps du tableau si besoin */
  .custom-table tbody td {
    color: #fff;
    background-color: #2c2c2c;
  }
  </style>
  
  