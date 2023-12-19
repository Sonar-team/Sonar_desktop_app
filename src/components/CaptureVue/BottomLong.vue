<template>
  <div>
    <table class="trames">
      <thead>
        <tr>
          <th>MAC Source</th>
          <th>MAC Destination</th>
          <th>Interface</th>
          <th>IP Source</th>
          <th>IP Destination</th>
          <th>Protocol</th>
          <th>Port Source</th>
          <th>Port Destination</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(frame, index) in frames" :key="index">
          <td>{{ frame.mac_address_source }}</td>
          <td>{{ frame.mac_address_destination }}</td>
          <td>{{ frame.interface }}</td>
          <td>{{ frame.layer_3_infos.ip_source }}</td>
          <td>{{ frame.layer_3_infos.ip_destination }}</td>
          <td>{{ frame.layer_3_infos.l_4_protocol }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.port_source }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.port_destination }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';

export default {
  data() {
    return {
      frames: [],
      counter: 0 // Initialisez le compteur à 0
    }
  },
  async mounted() {
    console.log('mounted bottom')
    // TODO: ne doit pas lancer all mais l'interface selectioné ...
    invoke('get_selected_interface', { interface_name: 'all' })
    console.log('get_selected_interface');
    await listen('frame', (packet_info) => {
      this.incrementAndEmit()
      //console.log('Received event:', packet_info);      // Push the new counter to the array
      this.frames.push(packet_info.payload);

      // Keep only the last 5 elements
      if (this.frames.length > 5) {
        this.frames.shift();
      }
    });
  },
  methods: {
    incrementAndEmit() {
      // Emit the custom event without specifying a value
      this.$emit('incremented');
    }
  }
}
</script>

<style scoped>
  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  th, td {
    width: 120px; /* Example fixed width */
    overflow: hidden; /* Hides content that overflows */
    white-space: nowrap; /* Prevents text from wrapping to the next line */
    text-overflow: ellipsis; /* Truncates with an ellipsis */
    border: 1px solid rgb(59, 81, 121);
    padding: 8px;
    text-align: center;
    color: rgb(255, 255, 255);
    background-color: #000000;
  }

  th {
    background-color: #000000;
  }
</style>
