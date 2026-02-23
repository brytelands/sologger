<template>
  <div class="space-y-4 mb-6">
    <!-- Metric Cards -->
    <div class="grid grid-cols-2 md:grid-cols-6 gap-3">
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
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Failed TX Rate</h4>
        <p class="text-2xl font-bold" :class="failedTxRate > 0 ? 'text-red-500' : 'text-green-500'">
          {{ failedTxRate }}%
        </p>
      </div>
      <div class="card p-4 flex flex-col gap-1">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">Last Update</h4>
        <p class="text-sm font-medium text-[var(--p-text-color)] mt-1">{{ lastUpdateTime }}</p>
      </div>
    </div>

    <!-- Charts Row -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <!-- Errors Over Time line chart -->
      <div class="card p-4 h-52">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)] mb-2">Errors Over Time</h4>
        <div class="h-40">
          <Line :data="lineChartData" :options="lineChartOptions" class="w-full h-full" />
        </div>
      </div>

      <!-- Log Level Distribution donut chart -->
      <div class="card p-4 h-52">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)] mb-2">Log Level Distribution</h4>
        <div class="h-40">
          <Doughnut :data="donutChartData" :options="donutChartOptions" class="w-full h-full" />
        </div>
      </div>

      <!-- CU Consumption bar chart -->
      <div class="card p-4 h-52">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)] mb-2">CU Consumption (Top Instructions)</h4>
        <div class="h-40">
          <Bar :data="cuBarChartData" :options="cuBarChartOptions" class="w-full h-full" />
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { Line, Doughnut, Bar } from 'vue-chartjs';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend
);

export default {
  components: { Line, Doughnut, Bar },
  props: ['parsedLogs', 'uniqueProgramsCount', 'lastUpdateTime'],
  computed: {
    errorCount() {
      return this.parsedLogs.filter(log => log.level === 'Error').length;
    },
    infoCount() {
      return this.parsedLogs.filter(log => log.level === 'Info').length;
    },
    failedTxRate() {
      if (!this.parsedLogs.length) return 0;
      const failed = this.parsedLogs.filter(log => log.transactionError && log.transactionError !== '').length;
      return ((failed / this.parsedLogs.length) * 100).toFixed(1);
    },
    isDarkMode() {
      return document.documentElement.getAttribute('data-theme') === 'dark';
    },
    textColor() {
      return this.isDarkMode ? '#8cb3a2' : '#40614f';
    },
    gridColor() {
      return this.isDarkMode ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.06)';
    },

    // --- Errors Over Time (line) ---
    lineChartData() {
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
    lineChartOptions() {
      return {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: true, labels: { color: this.textColor, font: { size: 11 } } }
        },
        scales: {
          y: { beginAtZero: true, ticks: { color: this.textColor, font: { size: 10 } }, grid: { color: this.gridColor } },
          x: { ticks: { color: this.textColor, font: { size: 10 } }, grid: { color: this.gridColor } }
        }
      };
    },

    // --- Log Level Distribution (donut) ---
    donutChartData() {
      const levels = {};
      for (const log of this.parsedLogs) {
        const lvl = log.level || 'Unknown';
        levels[lvl] = (levels[lvl] || 0) + 1;
      }
      const labels = Object.keys(levels);
      const palette = {
        Info: '#22c55e',
        Error: '#ef4444',
        Warning: '#f59e0b',
        Unknown: '#6b7280',
      };
      return {
        labels,
        datasets: [{
          data: labels.map(l => levels[l]),
          backgroundColor: labels.map(l => palette[l] ?? '#818cf8'),
          borderWidth: 1,
          borderColor: this.isDarkMode ? '#1e1e2e' : '#ffffff',
        }]
      };
    },
    donutChartOptions() {
      return {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: true,
            position: 'right',
            labels: { color: this.textColor, font: { size: 11 }, boxWidth: 12 }
          },
          tooltip: {
            callbacks: {
              label: ctx => ` ${ctx.label}: ${ctx.parsed} (${((ctx.parsed / this.parsedLogs.length) * 100).toFixed(1)}%)`
            }
          }
        }
      };
    },

    // --- CU Consumption (bar) ---
    cuBarChartData() {
      // Group by programId, take max CU per program, show top 8
      const cuByProgram = {};
      for (const log of this.parsedLogs) {
        if (log.computeUnits == null) continue;
        const pid = log.programId?.programId ?? log.programId ?? 'Unknown';
        const label = String(pid).substring(0, 8);
        if (!cuByProgram[label] || log.computeUnits > cuByProgram[label]) {
          cuByProgram[label] = log.computeUnits;
        }
      }
      const sorted = Object.entries(cuByProgram)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 8);
      return {
        labels: sorted.map(([k]) => k),
        datasets: [{
          label: 'Max CU Consumed',
          data: sorted.map(([, v]) => v),
          backgroundColor: sorted.map(([, v]) =>
            v > 100000 ? 'rgba(239,68,68,0.7)' : v > 50000 ? 'rgba(245,158,11,0.7)' : 'rgba(34,197,94,0.7)'
          ),
          borderRadius: 4,
        }]
      };
    },
    cuBarChartOptions() {
      return {
        responsive: true,
        maintainAspectRatio: false,
        indexAxis: 'y',
        plugins: {
          legend: { display: false },
          tooltip: {
            callbacks: {
              label: ctx => ` ${ctx.parsed.x.toLocaleString()} CU`
            }
          }
        },
        scales: {
          x: { beginAtZero: true, ticks: { color: this.textColor, font: { size: 10 } }, grid: { color: this.gridColor } },
          y: { ticks: { color: this.textColor, font: { size: 10 } }, grid: { color: this.gridColor } }
        }
      };
    },
  },
};
</script>
