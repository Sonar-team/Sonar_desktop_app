<template>
    <div class="chart"></div>
    <Bar
      id="my-bar-chart-id"
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
                labels: [ '192.168.1.1', '192.168.1.2', '192.168.1.3', '192.168.1.4' ], // Replace with actual IP addresses
                datasets: [ { 
                    data: [120, 90, 75, 60], // Replace with actual packet quantities          
                    backgroundColor: [
                        'rgba(255, 99, 132, 1)',
                        'rgba(54, 162, 235, 1)',
                        'rgba(255, 206, 86, 1)',
                        'rgba(75, 192, 192, 1)'
                    ] 
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

</style>


  