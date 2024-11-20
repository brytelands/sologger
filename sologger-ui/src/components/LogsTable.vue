<template>
  <div>
    <div class="pagination flex items-center justify-between p-4 bg-surface-50 border-t border-surface-200">
      <span class="text-sm text-surface-600">
        Showing {{ currentPage * pageSize + 1 }} - {{ Math.min((currentPage + 1) * pageSize, parsedLogs.length) }}
        of {{ parsedLogs.length }} entries
      </span>
      <div class="flex gap-2">
        <button
            @click="prevPage"
            :disabled="currentPage === 0"
            class="px-3 py-1 text-sm rounded border border-surface-200 hover:bg-surface-100 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>
        <button
            @click="nextPage"
            :disabled="currentPage >= totalPages - 1"
            class="px-3 py-1 text-sm rounded border border-surface-200 hover:bg-surface-100 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next
        </button>
      </div>
    </div>
    <div class="hot">
      <hot-table :data="paginatedLogs" :settings="hotSettings" ref="hotTable" />
    </div>
    <div class="pagination flex items-center justify-between p-4 bg-surface-50 border-t border-surface-200">
      <span class="text-sm text-surface-600">
        Showing {{ currentPage * pageSize + 1 }} - {{ Math.min((currentPage + 1) * pageSize, parsedLogs.length) }}
        of {{ parsedLogs.length }} entries
      </span>
      <div class="flex gap-2">
        <button
            @click="prevPage"
            :disabled="currentPage === 0"
            class="px-3 py-1 text-sm rounded border border-surface-200 hover:bg-surface-100 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>
        <button
            @click="nextPage"
            :disabled="currentPage >= totalPages - 1"
            class="px-3 py-1 text-sm rounded border border-surface-200 hover:bg-surface-100 disabled:opacity-50 disabled:cursor-not-allowed"
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
</style>