<template>
  <div class="logs-table-root" style="position: relative;">
    <!-- Column Picker + Pagination top bar -->
    <div class="pagination rounded-lg border border-[var(--p-card-border)]">
      <span class="text-sm text-[var(--p-text-muted)]">
        Showing {{ currentPage * pageSize + 1 }} - {{ Math.min((currentPage + 1) * pageSize, parsedLogs.length) }}
        of {{ parsedLogs.length }} entries
      </span>
      <div class="flex gap-2 items-center">
        <label class="flex items-center gap-1 text-sm cursor-pointer select-none">
          <input type="checkbox" v-model="autoScroll" class="accent-[var(--p-primary-color)]" />
          Auto-scroll
        </label>
        <button @click="toggleColumnPicker" class="btn btn-secondary">Columns ▾</button>
        <button
            @click="prevPage"
            :disabled="currentPage === 0"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >Previous</button>
        <button
            @click="nextPage"
            :disabled="currentPage >= totalPages - 1"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >Next</button>
      </div>
    </div>

    <!-- Column picker dropdown -->
    <div v-if="showColumnPicker" class="column-picker-dropdown">
      <div v-for="col in allColumns" :key="col.id" class="column-picker-item">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="isColumnVisible(col.id)" @change="toggleColumn(col.id)" />
          {{ col.name }}
        </label>
      </div>
    </div>

    <div v-if="!isMobile" class="slick-grid-wrapper">
      <div ref="gridContainer" class="slick-grid-container" style="height: calc(100vh - 350px); width: 100%;"></div>
    </div>

    <!-- Bottom pagination -->
    <div class="pagination rounded-lg border border-[var(--p-card-border)] flex-wrap gap-2">
      <span class="text-sm text-[var(--p-text-muted)]">
        Showing {{ currentPage * pageSize + 1 }} - {{ Math.min((currentPage + 1) * pageSize, parsedLogs.length) }}
        of {{ parsedLogs.length }} entries
      </span>
      <div class="flex gap-2 flex-wrap">
        <button
            @click="prevPage"
            :disabled="currentPage === 0"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >Previous</button>
        <button
            @click="nextPage"
            :disabled="currentPage >= totalPages - 1"
            class="btn btn-secondary disabled:opacity-50 disabled:cursor-not-allowed"
        >Next</button>
      </div>
    </div>

    <!-- Mobile Card View (< 768px) -->
    <div v-if="isMobile" class="mobile-card-list">
      <div
          v-for="(log, idx) in paginatedLogs"
          :key="idx"
          class="mobile-log-card"
          :class="{ 'mobile-log-card--error': log.level === 'Error' }"
          @click="selectedRow = log"
      >
        <div class="mobile-log-card__header">
          <span class="mobile-log-card__time">{{ log.timestamp }}</span>
          <span class="mobile-log-card__level" :class="'level-' + (log.level || 'unknown').toLowerCase()">{{ log.level || 'Unknown' }}</span>
          <span v-if="log.computeUnits" class="mobile-log-card__cu" :class="log.computeUnits > 100000 ? 'cu-high' : log.computeUnits > 50000 ? 'cu-mid' : 'cu-low'">{{ log.computeUnits.toLocaleString() }} CU</span>
        </div>
        <div class="mobile-log-card__program">
          <span class="mobile-log-card__label">Program:</span>
          <span class="mobile-log-card__value">{{ formatProgramId(log.programId) }}</span>
          <span v-if="log.depth" class="cpi-depth-badge" :class="'depth-' + Math.min(log.depth, 5)">d{{ log.depth }}</span>
        </div>
        <div v-if="log.invokeResult" class="mobile-log-card__result">
          <span class="mobile-log-card__label">Result:</span>
          <span class="mobile-log-card__value">{{ log.invokeResult }}</span>
        </div>
        <div v-if="log.transactionError" class="mobile-log-card__error">
          <span class="mobile-log-card__label">TX Error:</span>
          <span class="mobile-log-card__value text-red-500">{{ log.transactionError }}</span>
        </div>
        <div class="mobile-log-card__sig">
          <span class="mobile-log-card__label">Sig:</span>
          <span class="mobile-log-card__value font-mono text-xs">{{ formatSig(log.signature) }}</span>
        </div>
      </div>
      <div v-if="paginatedLogs.length === 0" class="mobile-empty">No logs to display.</div>
    </div>

    <!-- Row detail modal -->
    <div v-if="selectedRow" class="modal-overlay" @click.self="selectedRow = null">
      <div class="modal-panel">
        <div class="modal-header">
          <span class="font-semibold text-base">Log Detail</span>
          <div class="flex gap-2">
            <button
              v-if="uploadedIdl"
              @click="$emit('decode-with-idl', selectedRow); selectedRow = null"
              class="btn btn-info"
            >🔍 Decode with IDL</button>
            <button @click="selectedRow = null" class="btn btn-secondary">✕ Close</button>
          </div>
        </div>
        <div class="modal-body">
          <div v-for="col in allColumns" :key="col.id" class="detail-row">
            <span class="detail-label">{{ col.name }}</span>
            <span class="detail-value">{{ formatDetailValue(col.field, selectedRow[col.field]) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { SlickGrid, SlickRowSelectionModel } from 'slickgrid';
import 'slickgrid/dist/styles/css/slick.grid.css';
import 'slickgrid/dist/styles/css/slick-default-theme.css';

// TODO fix URL query params for different explorers
const ALL_COLUMNS = [
  { id: 'timestamp', name: 'Time', field: 'timestamp', width: 80, sortable: true, resizable: true },
  { id: 'level', name: 'Level', field: 'level', width: 100, sortable: true, resizable: true },
  {
    id: 'signature', name: 'Signature', field: 'signature', width: 110, resizable: true,
    formatter: (_, __, value, _col, dataContext) => {
      const sig = value?.signature ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'tx');
      return `<a href="${base}${sig}${suffix}" target="_blank" class="truncate-cell" title="${sig}">${String(sig).substring(0, 8)}...</a>`;
    }
  },
  {
    id: 'slot', name: 'Slot', field: 'slot', width: 100, sortable: true, resizable: true,
    formatter: (_, __, value) => {
      const slot = value?.slot ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'block');
      return `<a href="${base}${slot}${suffix}" target="_blank">${slot}</a>`;
    }
  },
  {
    id: 'programId', name: 'Program', field: 'programId', width: 110, resizable: true,
    formatter: (_, __, value, _col, dataContext) => {
      const pid = value?.programId ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'account');
      const depth = dataContext.depth ?? 0;
      const indentStyle = depth > 0 ? `padding-left:${depth * 12}px;` : '';
      const childClass = depth > 0 ? ' cpi-child' : '';
      return `<a href="${base}${pid}${suffix}" target="_blank" class="truncate-cell${childClass}" style="${indentStyle}" title="${pid}">${String(pid).substring(0, 8)}...</a>`;
    }
  },
  {
    id: 'parentProgramId', name: 'Parent', field: 'parentProgramId', width: 110, resizable: true,
    formatter: (_, __, value) => {
      const pid = value?.parentProgramId ?? value ?? '';
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'account');
      if (!pid) return '<span class="cu-na">—</span>';
      return `<a href="${base}${pid}${suffix}" target="_blank" class="truncate-cell cpi-parent" title="${pid}">${String(pid).substring(0, 8)}...</a>`;
    }
  },
  {
    id: 'depth', name: 'Depth', field: 'depth', width: 60, sortable: true, resizable: true,
    formatter: (_, __, value, _col, dataContext) => {
      const depth = value ?? 0;
      const indent = '&nbsp;&nbsp;&nbsp;'.repeat(depth);
      const badge = `<span class="cpi-depth-badge depth-${Math.min(depth, 5)}">${depth}</span>`;
      return `${indent}${badge}`;
    }
  },
  { id: 'instructionIndex', name: 'Idx', field: 'instructionIndex', width: 60, sortable: true, resizable: true },
  { id: 'invokeResult', name: 'Result', field: 'invokeResult', width: 200, resizable: true },
  { id: 'computeUnits', name: 'CU Used', field: 'computeUnits', width: 90, sortable: true, resizable: true,
    formatter: (_, __, value) => {
      if (value === null || value === undefined) return '<span class="cu-na">—</span>';
      const cu = Number(value);
      let cls = 'cu-low';
      if (cu > 100000) cls = 'cu-high';
      else if (cu > 50000) cls = 'cu-mid';
      return `<span class="cu-value ${cls}">${cu.toLocaleString()}</span>`;
    }
  },
  {
    id: 'logMessages', name: 'Logs', field: 'logMessages', width: 200, resizable: true,
    asyncPostRender: (cellNode, row, dataContext) => {
      try {
        const logs = JSON.parse(dataContext.logMessages);
        const content = logs.join(' | ');
        cellNode.innerHTML = `<div class="scrollable-cell" title="${content.replace(/"/g, '&quot;')}">${content}</div>`;
      } catch {
        cellNode.innerHTML = `<div class="scrollable-cell">${dataContext.logMessages ?? ''}</div>`;
      }
    }
  },
  {
    id: 'dataLogs', name: 'Data', field: 'dataLogs', width: 150, resizable: true,
    formatter: (_, __, value) => `<div class="scrollable-cell" title="${value ?? ''}">${value ?? ''}</div>`
  },
  {
    id: 'rawLogs', name: 'Raw Logs', field: 'rawLogs', width: 150, resizable: true,
    asyncPostRender: (cellNode, row, dataContext) => {
      try {
        const logs = JSON.parse(dataContext.rawLogs);
        const content = logs.join(' | ');
        cellNode.innerHTML = `<div class="scrollable-cell" title="${content.replace(/"/g, '&quot;')}">${content}</div>`;
      } catch {
        cellNode.innerHTML = `<div class="scrollable-cell">${dataContext.rawLogs ?? ''}</div>`;
      }
    }
  },
  {
    id: 'errors', name: 'Errors', field: 'errors', width: 150, resizable: true,
    formatter: (_, __, value) => `<div class="scrollable-cell" title="${value ?? ''}">${value ?? ''}</div>`
  },
  {
    id: 'transactionError', name: 'TX Error', field: 'transactionError', width: 150, resizable: true,
    formatter: (_, __, value) => `<div class="scrollable-cell" title="${value ?? ''}">${value ?? ''}</div>`
  }
];

const DEFAULT_HIDDEN = new Set(['rawLogs', 'dataLogs', 'parentProgramId']);

const EXPLORER_URLS = {
  solscan:  { tx: 'https://solscan.io/tx/', block: 'https://solscan.io/block/', account: 'https://solscan.io/account/' },
  solana:   { tx: 'https://explorer.solana.com/tx/', block: 'https://explorer.solana.com/block/', account: 'https://explorer.solana.com/address/' },
  solanafm: { tx: 'https://solana.fm/tx/', block: 'https://solana.fm/block/', account: 'https://solana.fm/address/', env: 'cluster=devnet-solana' },
};

function explorerBase(explorer, type) {
  return (EXPLORER_URLS[explorer] ?? EXPLORER_URLS.solscan)[type];
}

export default {
  props: ['parsedLogs', 'hotSettings', 'selectedExplorer', 'uploadedIdl'],
  emits: ['decode-with-idl'],
  data() {
    return {
      currentPage: 0,
      pageSize: 100,
      grid: null,
      autoScroll: true,
      showColumnPicker: false,
      hiddenColumns: new Set(DEFAULT_HIDDEN),
      selectedRow: null,
      allColumns: ALL_COLUMNS,
      sortColumns: [],
      isMobile: window.innerWidth < 768,
    };
  },
  computed: {
    paginatedLogs() {
      const start = this.currentPage * this.pageSize;
      return this.parsedLogs.slice(start, start + this.pageSize);
    },
    totalPages() {
      return Math.ceil(this.parsedLogs.length / this.pageSize);
    },
    visibleColumns() {
      return ALL_COLUMNS.filter(c => !this.hiddenColumns.has(c.id));
    }
  },
  methods: {
    formatProgramId(programId) {
      const pid = programId?.programId ?? programId ?? '';
      return String(pid).substring(0, 12) + (String(pid).length > 12 ? '...' : '');
    },
    formatSig(signature) {
      const sig = signature?.signature ?? signature ?? '';
      return String(sig).substring(0, 16) + (String(sig).length > 16 ? '...' : '');
    },
    onResize() {
      this.isMobile = window.innerWidth < 768;
      if (!this.isMobile && !this.grid) {
        this.$nextTick(() => this.initGrid());
      }
    },
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
    isColumnVisible(id) {
      return !this.hiddenColumns.has(id);
    },
    toggleColumn(id) {
      const hidden = new Set(this.hiddenColumns);
      if (hidden.has(id)) {
        hidden.delete(id);
      } else {
        hidden.add(id);
      }
      this.hiddenColumns = hidden;
      if (this.grid) {
        this.grid.setColumns(this.visibleColumns);
        this.grid.render();
      }
    },
    toggleColumnPicker() {
      this.showColumnPicker = !this.showColumnPicker;
    },
    formatDetailValue(field, value) {
      if (value === null || value === undefined) return '';
      if (typeof value === 'object') {
        // Extract the meaningful string from nested objects (signature, slot, programId, parentProgramId)
        return value[field] ?? JSON.stringify(value);
      }
      try {
        const parsed = JSON.parse(value);
        if (Array.isArray(parsed)) return parsed.join('\n');
      } catch { /* not JSON */ }
      return String(value);
    },
    initGrid() {
      const data = this.paginatedLogs.map((row, i) => ({ id: i, ...row }));
      const options = {
        enableCellNavigation: true,
        enableColumnReorder: false,
        forceFitColumns: false,
        frozenColumn: 1,
        rowHeight: 50,
        enableTextSelectionOnCells: true,
        enableHtmlRendering: true,
        multiColumnSort: true,
        enableAsyncPostRender: true,
        asyncPostRenderDelay: 0,
      };

      this.grid = new SlickGrid(this.$refs.gridContainer, data, this.visibleColumns, options);

      // Row selection model
      const selectionModel = new SlickRowSelectionModel();
      this.grid.setSelectionModel(selectionModel);
      this.grid.onSelectedRowsChanged.subscribe((e, args) => {
        if (args.rows.length > 0) {
          this.selectedRow = this.grid.getDataItem(args.rows[0]);
        }
      });

      // Column sorting
      this.grid.onSort.subscribe((e, args) => {
        this.sortColumns = args.sortCols;
        const data = this.getSortedData();
        this.grid.setData(data, true);
        this.grid.render();
      });
    },
    getSortedData() {
      const data = this.paginatedLogs.map((row, i) => ({ id: i, ...row }));
      if (!this.sortColumns || this.sortColumns.length === 0) return data;
      return data.sort((a, b) => {
        for (const sc of this.sortColumns) {
          const field = sc.sortCol.field;
          let av = a[field], bv = b[field];
          // unwrap nested objects
          if (av && typeof av === 'object') av = av[field] ?? '';
          if (bv && typeof bv === 'object') bv = bv[field] ?? '';
          const sign = sc.sortAsc ? 1 : -1;
          if (av < bv) return -sign;
          if (av > bv) return sign;
        }
        return 0;
      });
    },
    updateGridData() {
      if (this.grid) {
        const data = this.getSortedData();
        this.grid.setData(data, true);
        this.grid.render();
        if (this.autoScroll && data.length > 0) {
          this.grid.scrollRowIntoView(data.length - 1, false);
        }
      }
    }
  },
  mounted() {
    window.addEventListener('resize', this.onResize);
    if (!this.isMobile) this.initGrid();
    // Close the column picker when clicking outside
    this._outsideClick = (e) => {
      if (!this.$el.querySelector('.column-picker-dropdown')?.contains(e.target) &&
          !e.target.closest('.btn')) {
        this.showColumnPicker = false;
      }
    };
    document.addEventListener('click', this._outsideClick);
  },
  beforeUnmount() {
    window.removeEventListener('resize', this.onResize);
    document.removeEventListener('click', this._outsideClick);
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
  background: var(--p-card-bg);
}

.btn {
  padding: 0.25rem 0.75rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  cursor: pointer;
  border: 1px solid var(--p-card-border);
  background: var(--p-card-bg);
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

/* Column picker dropdown */
.column-picker-dropdown {
  position: absolute;
  z-index: 9999;
  background: var(--p-card-bg);
  border: 1px solid var(--p-card-border);
  border-radius: 0.5rem;
  padding: 0.5rem;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.25rem 1rem;
  box-shadow: 0 4px 16px rgba(0,0,0,0.2);
}

.column-picker-item {
  font-size: 0.875rem;
  color: var(--p-text-color);
  padding: 0.2rem 0.25rem;
  white-space: nowrap;
}

/* Row detail modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-panel {
  background: var(--p-card-bg);
  border: 1px solid var(--p-card-border);
  border-radius: 0.75rem;
  width: min(700px, 95vw);
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--p-card-border);
  color: var(--p-text-color);
}

.modal-body {
  overflow-y: auto;
  padding: 0.75rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.detail-row {
  display: grid;
  grid-template-columns: 130px 1fr;
  gap: 0.5rem;
  font-size: 0.875rem;
  border-bottom: 1px solid var(--p-card-border);
  padding-bottom: 0.4rem;
}

.detail-label {
  font-weight: 600;
  color: var(--p-primary-color);
  white-space: nowrap;
}

.detail-value {
  color: var(--p-text-color);
  word-break: break-all;
  white-space: pre-wrap;
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

/* Sort indicator styling */
.slick-grid-container .slick-sort-indicator {
  color: var(--p-primary-color) !important;
}

.slick-grid-container .slick-row {
  background: var(--p-card-bg);
  color: var(--p-text-color);
  border-bottom: 1px solid var(--p-card-border);
}

.slick-grid-container .slick-row.odd {
  background: var(--p-input-border);
}

.slick-grid-container .slick-row:hover .slick-cell {
  background: var(--p-input-border) !important;
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

/* Frozen column separator */
.slick-grid-container .slick-pane-left {
  border-right: 2px solid var(--p-primary-color);
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

/* CU value coloring */
.cu-na { color: var(--p-text-muted); }
.cu-value { font-variant-numeric: tabular-nums; font-weight: 600; }
.cu-low  { color: #22c55e; }
.cu-mid  { color: #f59e0b; }
.cu-high { color: #ef4444; }

/* CPI depth badges */
.cpi-depth-badge {
  display: inline-block;
  min-width: 20px;
  text-align: center;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 700;
  padding: 1px 5px;
  background: var(--p-surface-200, #2a2a3e);
  color: var(--p-text-muted);
}
.cpi-depth-badge.depth-0 { background: rgba(99,102,241,0.15); color: #818cf8; }
.cpi-depth-badge.depth-1 { background: rgba(34,197,94,0.15);  color: #4ade80; }
.cpi-depth-badge.depth-2 { background: rgba(245,158,11,0.15); color: #fbbf24; }
.cpi-depth-badge.depth-3 { background: rgba(239,68,68,0.15);  color: #f87171; }
.cpi-depth-badge.depth-4 { background: rgba(168,85,247,0.15); color: #c084fc; }
.cpi-depth-badge.depth-5 { background: rgba(20,184,166,0.15); color: #2dd4bf; }

/* CPI child/parent link styling */
.cpi-child  { border-left: 2px solid var(--p-primary-color); padding-left: 4px; }
.cpi-parent { opacity: 0.75; font-style: italic; }

/* Mobile card view */
.mobile-card-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.5rem 0;
}

.mobile-log-card {
  background: var(--p-card-bg);
  border: 1px solid var(--p-card-border);
  border-radius: 0.5rem;
  padding: 0.75rem;
  cursor: pointer;
  transition: border-color 0.15s;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.mobile-log-card:hover {
  border-color: var(--p-primary-color);
}

.mobile-log-card--error {
  border-left: 3px solid #ef4444;
}

.mobile-log-card__header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.mobile-log-card__time {
  font-size: 0.75rem;
  color: var(--p-text-muted);
  font-variant-numeric: tabular-nums;
}

.mobile-log-card__level {
  font-size: 0.7rem;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  text-transform: uppercase;
}

.level-info    { background: rgba(34,197,94,0.15);  color: #22c55e; }
.level-error   { background: rgba(239,68,68,0.15);  color: #ef4444; }
.level-warning { background: rgba(245,158,11,0.15); color: #f59e0b; }
.level-unknown { background: rgba(107,114,128,0.15); color: #6b7280; }

.mobile-log-card__cu {
  font-size: 0.7rem;
  font-weight: 600;
  margin-left: auto;
  font-variant-numeric: tabular-nums;
}

.mobile-log-card__program,
.mobile-log-card__result,
.mobile-log-card__error,
.mobile-log-card__sig {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  flex-wrap: wrap;
}

.mobile-log-card__label {
  color: var(--p-text-muted);
  font-weight: 600;
  white-space: nowrap;
  min-width: 50px;
}

.mobile-log-card__value {
  color: var(--p-text-color);
  word-break: break-all;
}

.mobile-empty {
  text-align: center;
  color: var(--p-text-muted);
  padding: 2rem;
  font-size: 0.9rem;
}

/* Pagination wrapping fix for mobile */
@media (max-width: 768px) {
  .pagination {
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .logs-table-root {
    overflow-x: hidden;
  }
}
</style>
