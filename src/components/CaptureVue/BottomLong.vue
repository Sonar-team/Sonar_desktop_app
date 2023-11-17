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
        <tr v-for="(counter, index) in counters" :key="index">
          <td>{{ index + 1 }}</td>
          <td>{{ counter }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { listengit } from '@tauri-apps/api/event'

export default {
  data() {
    return {
      counters: []
    }
  },
  async mounted() {
    await listen('counter', (event) => {
      // Push the new counter to the array
      this.counters.push(event.payload);

      // Keep only the last 10 elements
      if (this.counters.length > 5) {
        this.counters.shift();
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
