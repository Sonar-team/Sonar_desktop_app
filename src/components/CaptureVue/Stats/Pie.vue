<template>
    <div class="chart"></div>
    <Pie
      id="my-pi-chart-id"
      :options="chartOptions"
      :data="chartData"
    />
</template>
  
<script>
import { Pie } from 'vue-chartjs'
import { Chart as ChartJS, ArcElement, Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale } from 'chart.js'

ChartJS.register(ArcElement, Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale)

export default {
    name: 'PieChart',
    components: { Pie },
    data() {
        return {
            chartData: {
                    labels: [ 'TCP', 'UDP', 'HTTP', 'HTTPS', 'FTP' ],
                    datasets: [ { 
                        data: [30, 25, 15, 20, 10], // Example data          
                        backgroundColor: [
                            'rgba(255, 99, 132, 1)',
                            'rgba(54, 162, 235, 1)',
                            'rgba(255, 206, 86, 1)',
                            'rgba(75, 192, 192, 1)',
                            'rgba(153, 102, 255, 1)'
                        ],
                        fontColor: 'black', 
                            data: [40, 20, 12, 39, 10]
                    } ]
                },
            chartOptions: { 
            responsive: true,
            legend: { display: false },
        scales: {
          yAxes: [{
            ticks: {
              fontColor: '#eaeaea'
            },
          }],
          xAxes: [{
            ticks: {
              fontColor: '#eaeaea'
            },
          }]
        }
            }
        }
    }
}

</script>

<style>

</style>


  