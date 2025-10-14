<template>
  <div>
    <table class="trames">
      <thead>
        <tr>
          <th>MAC Source</th>
          <th>MAC Destination</th>
          <th>Data Link</th>
          <th>IP Source</th>
          <th>IP Destination</th>
          <th>Proto T.</th>
          <th>Port Source</th>
          <th>Port Dest.</th>
          <th>App Proto</th>
          <th>Len</th>
          <th>Timestamp</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(packet, index) in packets" :key="index">
          <td>{{ packet.flow?.source_mac ?? '-' }}</td>
          <td>{{ packet.flow?.destination_mac ?? '-' }}</td>
          <td>{{ packet.flow?.ethertype ?? '-' }}</td>
          <td>{{ packet.flow?.source ?? '-' }}</td>
          <td>{{ packet.flow?.destination ?? '-' }}</td>
          <td>{{ packet.flow?.protocol ?? '-' }}</td>
          <td>{{ packet.flow?.source_port ?? '-' }}</td>
          <td>{{ packet.flow?.destination_port ?? '-' }}</td>
          <td>{{ packet.flow?.application_protocol ?? '-' }}</td>
          <td>{{ packet.len }}</td>
          <td>{{ formatTimestamp(packet.ts_sec, packet.ts_usec) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { useCaptureStore } from '../../store/capture';
import { PacketMinimal } from '../../types/capture';



export default defineComponent({
  data() {
    return {
      packets: [] as PacketMinimal[],
      
    };
  },
  computed: {
    captureStore() {
      return useCaptureStore();
    },
  },
  async mounted() {
    this.captureStore.onPacket((packet) => {
      this.packets.push(packet);
      if (this.packets.length > 5) this.packets.shift(); // garde les 100 derniers
    });

    this.$bus?.on?.('reset', () => {
      this.packets = [];
    });
  },

  beforeUnmount() {


    this.$bus?.off?.('reset');
  },

  methods: {
    formatTimestamp(sec: number, usec: number): string {
      if (typeof sec !== 'number' || typeof usec !== 'number') return '-';
      const date = new Date(sec * 1000 + Math.floor(usec / 1000));
      return date.toLocaleTimeString('fr-FR', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        fractionalSecondDigits: 3,
      });
    },
  },
});
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
