<template>
    <div class="chart"></div>
    <Bar
      id="my-chart-id"
      :options="chartOptions"
      :data="chartData"
    />
</template>
  
<script>
import { Bar } from 'vue-chartjs'
import { Chart as ChartJS, Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale } from 'chart.js'

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale)

export default {
    name: 'BarChart',
    components: { Bar },
    data() {
    return {
        chartData: {
        labels: [ 'January', 'February', 'March' ],
        datasets: [ { 
            data: [40, 20, 12],          
            backgroundColor: ['rgba(255, 99, 132, 1)', 'rgba(54, 162, 235, 1)', 'rgba(255, 206, 86, 1)'] 
        } ]
        },
        chartOptions: {
        responsive: true
        }
    }
    }
}
</script>

<style>
    .chart {
        background-color: #ffffff;

    }
</style>


  