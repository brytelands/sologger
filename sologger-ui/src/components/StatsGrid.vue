<template>
  <div class="space-y-4 mb-6">
    <div class="grid grid-cols-2 md:grid-cols-5 gap-3">
      <div class="card p-4 flex flex-col gap-1">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Total Logs</h4>
        <p class="text-2xl font-bold text-[var(--p-text-color)]">{{ parsedLogs.length }}</p>
      </div>
      <div class="card p-4 flex flex-col gap-1">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Unique Progs</h4>
        <p class="text-2xl font-bold text-[var(--p-text-color)]">{{ uniqueProgramsCount }}</p>
      </div>
      <div class="card p-4 flex flex-col gap-1">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Error Logs</h4>
        <p class="text-2xl font-bold text-red-500">{{ errorCount }}</p>
      </div>
      <div class="card p-4 flex flex-col gap-1">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Info Logs</h4>
        <p class="text-2xl font-bold text-green-500">{{ infoCount }}</p>
      </div>
      <div class="card p-4 flex flex-col gap-1">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Last Update</h4>
        <p class="text-sm font-medium text-[var(--p-text-color)] mt-1">{{ lastUpdateTime }}</p>
      </div>
    </div>
    <div class="card p-4 h-48">
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
    isDarkMode() {
      return document.documentElement.getAttribute('data-theme') === 'dark';
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
          backgroundColor: 'rgba(239, 68, 68, 0.1)',
          tension: 0.4,
          pointBackgroundColor: '#ef4444',
          pointRadius: 3,
        }]
      };
    },
    chartOptions() {
      const textColor = this.isDarkMode ? '#8cb3a2' : '#40614f';
      const gridColor = this.isDarkMode ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
      return {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: true,
            labels: {
              color: textColor,
              font: { size: 12 }
            }
          }
        },
        scales: {
          y: {
            beginAtZero: true,
            ticks: { color: textColor, font: { size: 11 } },
            grid: { color: gridColor }
          },
          x: {
            ticks: { color: textColor, font: { size: 11 } },
            grid: { color: gridColor }
          }
        }
      };
    },
  },
};
</script>
