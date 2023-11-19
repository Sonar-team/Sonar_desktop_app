<template>
  <div>
    <table>
      <thead>
        <tr>
          <th>Index</th>
          <th>Counter Value</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(frame, index) in frames" :key="index">
          <td>{{ index + 1 }}</td>
          <td>{{ frame }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { listen } from '@tauri-apps/api/event'

export default {
  data() {
    return {
      frames: []
    }
  },
  async mounted() {
    await listen('frame', (event) => {
      // Push the new counter to the array
      this.frames.push(event.payload);

      // Keep only the last 10 elements
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
  }

  th {
    background-color: #f2f2f2;
  }
</style>
