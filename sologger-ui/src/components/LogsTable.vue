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
    <div class="hot" style="margin-top: 8px; margin-bottom: 8px;">
      <hot-table :data="paginatedLogs" :settings="hotSettings" ref="hotTable" />
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
import { HotTable } from '@handsontable/vue3';
import 'handsontable/dist/handsontable.full.min.css';
import 'handsontable/dist/handsontable.full.min.css';

export default {
  components: {
    HotTable
  },
  props: ['parsedLogs', 'hotSettings'],
  data() {
    return {
      currentPage: 0,
      pageSize: 100,
      localHotSettings: {
        ...this.hotSettings,
        height: 'calc(100vh - 350px)', // Adjusted for pagination controls
        width: '100%',
        renderAllRows: false,
        viewportRowRenderingOffset: 70,
        autoRowSize: false,
        autoColumnSize: false,
        rowHeights: 50,
        rowHeaders: false,
        columnHeaders: false,
        currentRowClassName: 'current-row',
        preventOverflow: 'horizontal',
        outsideClickDeselects: false,
        colWidths: [
          80, 100, 70, 100, 100, 60, 60, 80, 200, 150, 150, 150
        ],
        afterUpdateSettings: true,
        afterRender: true,
        afterChange: true,
        columns: [
          { data: 'timestamp', title: 'Time' },
          { data: 'level', title: 'Level' },
          {
            data: 'signature',
            title: 'Signature',
            renderer: (_, td, __, ___, prop, value) => {
              td.innerHTML = `<div class="truncate-cell" title="${value}">${value?.substring(0, 8)}...</div>`;
              return td;
            }
          },
          { data: 'slot', title: 'Slot' },
          {
            data: 'programId',
            title: 'Program',
            renderer: (_, td, __, ___, prop, value) => {
              td.innerHTML = `<div class="truncate-cell" title="${value}">${value?.substring(0, 8)}...</div>`;
              return td;
            }
          },
          {
            data: 'parentProgramId',
            title: 'Parent',
            renderer: (_, td, __, ___, prop, value) => {
              td.innerHTML = `<div class="truncate-cell" title="${value}">${value?.substring(0, 8)}...</div>`;
              return td;
            }
          },
          { data: 'depth', title: 'Depth' },
          { data: 'instructionIndex', title: 'Idx' },
          { data: 'invokeResult', title: 'Result' },
          {
            data: 'logMessages',
            title: 'Logs',
            renderer: (_, td, __, ___, prop, value) => {
              try {
                const logs = JSON.parse(value);
                const content = logs.join('\n');
                td.innerHTML = `<div class="scrollable-cell" title="${content}">${content}</div>`;
              } catch (e) {
                td.innerHTML = `<div class="scrollable-cell">${value}</div>`;
              }
              return td;
            }
          },
          {
            data: 'dataLogs',
            title: 'Data',
            renderer: (_, td, __, ___, prop, value) => {
              td.innerHTML = `<div class="scrollable-cell" title="${value}">${value}</div>`;
              return td;
            }
          },
          {
            data: 'rawLogs',
            title: 'Raw Logs',
            renderer: (_, td, __, ___, prop, value) => {
              try {
                const logs = JSON.parse(value);
                const content = logs.join('\n');
                td.innerHTML = `<div class="scrollable-cell" title="${content}">${content}</div>`;
              } catch (e) {
                td.innerHTML = `<div class="scrollable-cell">${value}</div>`;
              }
              return td;
            }
          },
          {
            data: 'errors',
            title: 'Errors',
            renderer: (_, td, __, ___, prop, value) => {
              td.innerHTML = `<div class="scrollable-cell" title="${value}">${value}</div>`;
              return td;
            }
          },
          {
            data: 'transactionError',
            title: 'TX Error',
            renderer: (_, td, __, ___, prop, value) => {
              td.innerHTML = `<div class="scrollable-cell" title="${value}">${value}</div>`;
              return td;
            }
          }
        ]
      }
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
    }
  },
  watch: {
    paginatedLogs: {
      handler(newLogs) {
        if (this.$refs.hotTable) {
          this.$refs.hotTable.hotInstance.loadData(newLogs);
        }
      },
      deep: true
    }
  }
};
</script>

<style>
@import 'handsontable/dist/handsontable.full.min.css';

.hot .handsontable .wtHolder {
  border-top: none;
  border-bottom: none;
}

.hot .ht_master .wtHolder {
  border-left: 1px solid var(--p-card-border);
  border-right: 1px solid var(--p-card-border);
}

.hot .ht_clone_left { display: none; } /* Hides the row header column */

</style>