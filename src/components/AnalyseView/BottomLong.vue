<template>
  <div>
    <table class="trames">
      <thead>
        <tr>
          <th>MAC Source</th>
          <th>MAC Destination</th>
          <th>Interface</th>
          <th>L3</th>
          <th>IP Source</th>
          <th>IP Destination</th>
          <th>L4</th>
          <th>Port Source</th>
          <th>Port Destination</th>
          <th>L7</th>
          <th>Taille (o)</th>
          <th>Horodatage</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(frame, index) in frames" :key="index">
          <td>{{ frame.flow.mac_address_source }}</td>
          <td>{{ frame.flow.mac_address_destination }}</td>
          <td>{{ frame.flow.interface }}</td>
          <td>{{ frame.flow.l_3_protocol }}</td>
          <td>{{ frame.flow.layer_3_infos?.ip_source || '-' }}</td>
          <td>{{ frame.flow.layer_3_infos?.ip_destination || '-' }}</td>
          <td>{{ frame.flow.layer_3_infos?.l_4_protocol || '-' }}</td>
          <td>{{ frame.flow.layer_3_infos?.layer_4_infos?.port_source || '-' }}</td>
          <td>{{ frame.flow.layer_3_infos?.layer_4_infos?.port_destination || '-' }}</td>
          <td>{{ frame.flow.layer_3_infos?.layer_4_infos?.l_7_protocol || '-' }}</td>
          <td>{{ frame.len }}</td>
          <td>{{ frame.formatted_time }}</td>

        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { listen } from '@tauri-apps/api/event';
import { debug } from '@tauri-apps/plugin-log';

export default {
  data() {
    return {
      frames: [], // rempli directement depuis l'event backend
    };
  },

  async mounted() {
    await listen('frame', (event) => {
      console.log('[FRONT] Payload brut :', event.payload);
      if (Array.isArray(event.payload)) {
        console.log('[FRONT] Nombre de paquets :', event.payload.length);
      } else {
        console.warn('[FRONT] Payload non conforme', event.payload);
      }
      this.frames = event.payload || [];
    });

    this.$bus.on('reset', () => {
      this.frames = [];
    });
  },

  beforeUnmount() {
    this.$bus.off('reset');
  },
};
</script>

<style scoped>
.trames {
  display: block;
  height: 190px;
  flex-shrink: 0;
  background-color: #000;
  font-family: 'Courier New', Courier, monospace;
}

table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

td, th {
  padding: 8px;
  text-align: center;
  color: rgb(132, 195, 247);
  background-color: #000;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

tbody {
  display: block;
  overflow-y: auto;
}

thead, tbody tr {
  display: table;
  width: 100%;
  table-layout: fixed;
}
</style>
