<template>
  <div>
    <table class="matrice">
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
//import { listen } from '@tauri-apps/api/event';

export default {
  data() {
    return {
      frames: [
      {
          mac_address_source: '00:1A:2B:3C:4D:5E',
          mac_address_destination: '5E:4D:3C:2B:1A:00',
          interface: 'eth0',
          layer_3_infos: {
            ip_source: '192.168.0.1',
            ip_destination: '192.168.0.2',
            l_4_protocol: 'TCP',
            layer_4_infos: {
              port_source: 443,
              port_destination: 80
            }
          }
        },
        {
        mac_address_source: '00:1A:2B:0C:4D:5E',
          mac_address_destination: '5E:0D:3C:2B:1A:00',
          interface: 'eth0',
          layer_3_infos: {
            ip_source: '192.108.0.1',
            ip_destination: '102.168.0.2',
            l_4_protocol: 'TCP',
            layer_4_infos: {
              port_source: 403,
              port_destination: 800
            }
          }
        }
      ]
    }
  },
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
    color: rgb(0, 0, 0);
    background-color: #ffffff;
  }

  th {
    background-color: #000000;
    color: rgb(255, 255, 255);
  }
</style>

