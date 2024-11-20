<template>
  <div class="space-y-4 mb-6">
    <div class="grid grid-cols-5 gap-4">
      <div class="bg-blue-100 dark:bg-blue-900 p-4 rounded">
        <h4 class="font-bold dark:text-white">Total Logs</h4>
        <p class="dark:text-white">{{ parsedLogs.length }}</p>
      </div>
      <div class="bg-green-100 dark:bg-green-900 p-4 rounded">
        <h4 class="font-bold dark:text-white">Unique Programs</h4>
        <p class="dark:text-white">{{ uniqueProgramsCount }}</p>
      </div>
      <div class="bg-red-100 dark:bg-red-900 p-4 rounded">
        <h4 class="font-bold dark:text-white">Error Logs</h4>
        <p class="dark:text-white">{{ errorCount }}</p>
      </div>
      <div class="bg-emerald-100 dark:bg-emerald-900 p-4 rounded">
        <h4 class="font-bold dark:text-white">Info Logs</h4>
        <p class="dark:text-white">{{ infoCount }}</p>
      </div>
      <div class="bg-purple-100 dark:bg-purple-900 p-4 rounded">
        <h4 class="font-bold dark:text-white">Last Update</h4>
        <p class="dark:text-white">{{ lastUpdateTime }}</p>
      </div>
    </div>
    <div class="bg-surface-800 dark:bg-surface-50 p-4 rounded h-48">
      <Line :data="chartData"
            :options="chartOptions"
            class="w-full h-full"
      />
    </div>
  </div>
</template>

<script>
import { Line } from 'vue-chartjs';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend } from 'chart.js';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend);

export default {
  components: { Line },
  props: ['parsedLogs', 'uniqueProgramsCount', 'lastUpdateTime'],
  computed: {
    errorCount() {
      return this.parsedLogs.filter(log => log.level === 'Error').length;
    },
    infoCount() {
      return this.parsedLogs.filter(log => log.level === 'Info').length;
    },
    chartData() {
      const timeLabels = [...new Set(this.parsedLogs.map(log => log.timestamp))].slice(-10);
      const errorCounts = timeLabels.map(time =>
          this.parsedLogs.filter(log => log.timestamp === time && log.level === 'Error').length
      );

      return {
        labels: timeLabels,
        datasets: [{
          label: 'Errors Over Time',
          data: errorCounts,
          borderColor: '#ef4444',
          backgroundColor: '#fee2e2',
          tension: 0.4
        }]
      };
    },
    chartOptions() {
      return {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: true,
            labels: {
              color: this.isDarkMode ? '#fff' : '#000'
            }
          }
        },
        scales: {
          y: {
            beginAtZero: true,
            ticks: {
              color: this.isDarkMode ? '#fff' : '#000'
            }
          },
          x: {
            ticks: {
              color: this.isDarkMode ? '#fff' : '#000'
            }
          }
        }
      };
    },
  },

};
</script>

<style>
.dark .chartjs-render-monitor {
  filter: invert(1) hue-rotate(180deg);
}
</style>