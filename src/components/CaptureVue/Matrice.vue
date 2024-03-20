<template>
  <v-theme-provider theme="dark">
    <template v-slot:text>
      <v-text-field
        v-model="search"
        label="Search"
        prepend-inner-icon="mdi-magnify"
        variant="outlined"
        hide-details
        single-line
      ></v-text-field>
    </template>
    <v-data-table
      :headers="headers"
      :items="processedPackets"
      item-key="id"
      class="elevation-1 matrice"
      :fixed-header="true"
      :hide-default-footer="true"
    ></v-data-table>
  </v-theme-provider>
</template>


<script>
import { invoke } from '@tauri-apps/api/tauri';

export default {
  data() {
    return {
      packets: [],
      intervalId: null,
      headers: [
        { title: 'MAC Source', value: 'info.mac_address_source',key: 'info.mac_address_source',sortable: true },
        { title: 'MAC Destination', value: 'info.mac_address_destination' ,sortable: true},
        { title: 'Interface', value: 'info.interface' ,sortable: true},
        { title: 'Protocole L3', value: 'info.l_3_protocol' ,sortable: true},
        { title: 'IP Source', value: 'info.layer_3_infos.ip_source',sortable: true },
        { title: 'IP Source Type', value: 'info.layer_3_infos.ip_source_type' ,sortable: true},
        { title: 'IP Destination', value: 'info.layer_3_infos.ip_destination' ,sortable: true},
        { title: 'IP Destination Type', value: 'info.layer_3_infos.ip_destination_type',sortable: true ,removable: true,},
        { title: 'Protocole L4', value: 'info.layer_3_infos.l_4_protocol' ,sortable: true},
        { title: 'Port Source', value: 'info.layer_3_infos.layer_4_infos.port_source' ,sortable: true},
        { title: 'Port Destination', value: 'info.layer_3_infos.layer_4_infos.port_destination' ,sortable: true},
        { title: 'Protocole L7', value: 'info.layer_3_infos.layer_4_infos.l_7_protocol' ,sortable: true},
        { title: 'Taille Total', value: 'info.packet_size' ,sortable: true},
        { title: 'Occurences', value: 'count' ,sortable: true},
      ],
    };
  },
  computed: {
    processedPackets() {
      return this.processData(this.packets);
    }
  },
  mounted() {
    this.intervalId = setInterval(this.fetchPacketInfos, 1000);
  },
  beforeDestroy() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
  },
  methods: {
    async fetchPacketInfos() {
      try {
        const jsonString = await invoke('get_matrice', {});
        this.packets = JSON.parse(jsonString);
        // Emitting event for packet count update
        this.$bus.emit('update-packet-count', this.packets.length);
      } catch (error) {
        console.error("Error fetching packet infos:", error);
      }
    },
    processData(data) {
      return data.map(([packetInfo, count], index) => ({
        id: index,
        info: packetInfo,
        count: count
      }));
    }
  },
};
</script>

<style scoped>
/* Your CSS styles */
</style>
