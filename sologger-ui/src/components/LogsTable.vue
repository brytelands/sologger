<template>
  <div class="overflow-x-auto">
    <div class="pagination rounded-lg border border-[var(--p-card-border)]">
      <span class="text-sm text-[var(--p-text-muted)]">
        Showing {{ currentPage * pageSize + 1 }} - {{ Math.min((currentPage + 1) * pageSize, parsedLogs.length) }}
        of {{ parsedLogs.length }} entries
      </span>
      <div class="flex gap-2">
        <button
            @click="prevPage"
            :disabled="currentPage === 0"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>
        <button
            @click="nextPage"
            :disabled="currentPage >= totalPages - 1"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next
        </button>
      </div>
    </div>
    <div class="slick-grid-wrapper">
      <div ref="gridContainer" class="slick-grid-container" style="height: calc(100vh - 350px); width: 100%;"></div>
    </div>
    <div class="pagination rounded-lg border border-[var(--p-card-border)]">
      <span class="text-sm text-[var(--p-text-muted)]">
        Showing {{ currentPage * pageSize + 1 }} - {{ Math.min((currentPage + 1) * pageSize, parsedLogs.length) }}
        of {{ parsedLogs.length }} entries
      </span>
      <div class="flex gap-2">
        <button
            @click="prevPage"
            :disabled="currentPage === 0"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>
        <button
            @click="nextPage"
            :disabled="currentPage >= totalPages - 1"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next
        </button>
      </div>
    </div>
  </div>
</template>

<script>
import { SlickGrid } from 'slickgrid';
import 'slickgrid/dist/styles/css/slick.grid.css';
import 'slickgrid/dist/styles/css/slick-default-theme.css';

const columns = [
  { id: 'timestamp', name: 'Time', field: 'timestamp', width: 80 },
  { id: 'level', name: 'Level', field: 'level', width: 100 },
  {
    id: 'signature', name: 'Signature', field: 'signature', width: 70,
    formatter: (_, __, value) => {
      const sig = value?.signature ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      return `<a href="https://solscan.io/tx/${sig}${suffix}" target="_blank" class="truncate-cell" title="${sig}">${String(sig).substring(0, 8)}...</a>`;
    }
  },
  {
    id: 'slot', name: 'Slot', field: 'slot', width: 100,
    formatter: (_, __, value) => {
      const slot = value?.slot ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      return `<a href="https://solscan.io/block/${slot}${suffix}" target="_blank">${slot}</a>`;
    }
  },
  {
    id: 'programId', name: 'Program', field: 'programId', width: 100,
    formatter: (_, __, value) => {
      const pid = value?.programId ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      return `<a href="https://solscan.io/account/${pid}${suffix}" target="_blank" class="truncate-cell" title="${pid}">${String(pid).substring(0, 8)}...</a>`;
    }
  },
  {
    id: 'parentProgramId', name: 'Parent', field: 'parentProgramId', width: 60,
    formatter: (_, __, value) => {
      const pid = value?.parentProgramId ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      return `<a href="https://solscan.io/account/${pid}${suffix}" target="_blank" class="truncate-cell" title="${pid}">${String(pid).substring(0, 8)}...</a>`;
    }
  },
  { id: 'depth', name: 'Depth', field: 'depth', width: 60 },
  { id: 'instructionIndex', name: 'Idx', field: 'instructionIndex', width: 80 },
  { id: 'invokeResult', name: 'Result', field: 'invokeResult', width: 200 },
  {
    id: 'logMessages', name: 'Logs', field: 'logMessages', width: 150,
    formatter: (_, __, value) => {
      try {
        const logs = JSON.parse(value);
        const content = logs.join('\n');
        return `<div class="scrollable-cell" title="${content}">${content}</div>`;
      } catch (e) {
        return `<div class="scrollable-cell">${value ?? ''}</div>`;
      }
    }
  },
  {
    id: 'dataLogs', name: 'Data', field: 'dataLogs', width: 150,
    formatter: (_, __, value) =>
      `<div class="scrollable-cell" title="${value ?? ''}">${value ?? ''}</div>`
  },
  {
    id: 'rawLogs', name: 'Raw Logs', field: 'rawLogs', width: 150,
    formatter: (_, __, value) => {
      try {
        const logs = JSON.parse(value);
        const content = logs.join('\n');
        return `<div class="scrollable-cell" title="${content}">${content}</div>`;
      } catch (e) {
        return `<div class="scrollable-cell">${value ?? ''}</div>`;
      }
    }
  },
  {
    id: 'errors', name: 'Errors', field: 'errors', width: 150,
    formatter: (_, __, value) =>
      `<div class="scrollable-cell" title="${value ?? ''}">${value ?? ''}</div>`
  },
  {
    id: 'transactionError', name: 'TX Error', field: 'transactionError', width: 150,
    formatter: (_, __, value) =>
      `<div class="scrollable-cell" title="${value ?? ''}">${value ?? ''}</div>`
  }
];

const gridOptions = {
  enableCellNavigation: true,
  enableColumnReorder: false,
  forceFitColumns: false,
  rowHeight: 50,
  enableTextSelectionOnCells: true,
  enableHtmlRendering: true,
};

export default {
  props: ['parsedLogs', 'hotSettings'],
  data() {
    return {
      currentPage: 0,
      pageSize: 100,
      grid: null,
      dataView: null,
    };
  },
  computed: {
    paginatedLogs() {
      const start = this.currentPage * this.pageSize;
      return this.parsedLogs.slice(start, start + this.pageSize);
    },
    totalPages() {
      return Math.ceil(this.parsedLogs.length / this.pageSize);
    }
  },
  methods: {
    nextPage() {
      if (this.currentPage < this.totalPages - 1) {
        this.currentPage++;
      }
    },
    prevPage() {
      if (this.currentPage > 0) {
        this.currentPage--;
      }
    },
    goToPage(page) {
      if (page >= 0 && page < this.totalPages) {
        this.currentPage = page;
      }
    },
    initGrid() {
      const data = this.paginatedLogs.map((row, i) => ({ id: i, ...row }));
      this.grid = new SlickGrid(this.$refs.gridContainer, data, columns, gridOptions);
    },
    updateGridData() {
      if (this.grid) {
        const data = this.paginatedLogs.map((row, i) => ({ id: i, ...row }));
        this.grid.setData(data, true);
        this.grid.render();
      }
    }
  },
  mounted() {
    this.initGrid();
  },
  beforeUnmount() {
    if (this.grid) {
      this.grid.destroy();
      this.grid = null;
    }
  },
  watch: {
    paginatedLogs: {
      handler() {
        this.updateGridData();
      },
      deep: true
    }
  }
};
</script>

<style>
.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1rem;
  background: var(--p-card-background);
}

.btn {
  padding: 0.25rem 0.75rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  cursor: pointer;
  border: 1px solid var(--p-card-border);
  background: var(--p-card-background);
  color: var(--p-text-color);
}

.btn:hover:not(:disabled) {
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color);
}

.slick-grid-wrapper {
  border: 1px solid var(--p-card-border);
  border-radius: 0.5rem;
  overflow: hidden;
  margin-top: 8px;
  margin-bottom: 8px;
}

.slick-grid-container {
  width: 100%;
}

/* Override SlickGrid default theme to match app theme */
.slick-grid-container .slick-header {
  background: var(--p-surface-100, #1e1e2e) !important;
  border-bottom: 1px solid var(--p-card-border);
}

.slick-grid-container .slick-header-column {
  background: var(--p-surface-100, #1e1e2e) !important;
  color: var(--p-surface-50) !important;
  border-right: 1px solid var(--p-card-border) !important;
  font-weight: 600;
}

.slick-grid-container .slick-header-column:hover {
  background: var(--p-surface-200, #2a2a3e) !important;
}

.slick-grid-container .slick-row {
  background: var(--p-card-background);
  color: var(--p-text-color);
  border-bottom: 1px solid var(--p-card-border);
}

.slick-grid-container .slick-row.odd {
  background: var(--p-content-hover-background);
}

.slick-grid-container .slick-row:hover .slick-cell {
  background: var(--p-content-hover-background) !important;
}

.slick-grid-container .slick-cell {
  color: var(--p-text-color);
  border-right: 1px solid var(--p-card-border);
  padding: 4px 6px;
  overflow: hidden;
}

.slick-grid-container .slick-cell.selected {
  background: var(--p-primary-color) !important;
  color: var(--p-primary-contrast-color) !important;
}

.slick-grid-container a {
  color: var(--p-primary-color);
  text-decoration: none;
}

.slick-grid-container a:hover {
  text-decoration: underline;
}

.truncate-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.scrollable-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}
</style>
