<template>
  <div>
    <table>
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
import { listen } from '@tauri-apps/api/event'

export default {
  data() {
    return {
      frames: []
    }
  },
  async mounted() {
    console.log('mounted bottom')
    await invoke('get_selected_interface', { interface_name: 'all' })
    await listen('frame', (packet_info) => {
      console.log('Received event:', packet_info);      // Push the new counter to the array
      this.frames.push(packet_info.payload);

      // Keep only the last 5 elements
      if (this.frames.length > 5) {
        this.frames.shift();
      }
    });
  }
}
</script>

<style>
  table {
    width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    border: 1px solid black;
    padding: 8px;
    text-align: center;
    color: aliceblue;
  
  }

  th {
    background-color: #000000;
    text-align: center;
  }
</style>
 