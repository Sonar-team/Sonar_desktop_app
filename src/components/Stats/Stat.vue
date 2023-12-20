<template>
    <div class="chart-container">
      <BarChart />
      <PieChart />
    </div>
  </template>
  
  <script>
  import BarChart from './Stats/Bar.vue';
  import PieChart from './Stats/Pie.vue';
  
  export default {
    name: 'App',
    components: {
      BarChart,
      PieChart
    }
  }
  </script>
  
<style >
  .chart-container {
    background-color: aliceblue;
  }

</style>

  