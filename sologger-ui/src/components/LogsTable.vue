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
        <button
            ref="columnsButton"
            type="button"
            @click="toggleColumnPicker"
            @keydown.esc="closeColumnPicker"
            :aria-expanded="String(showColumnPicker)"
            aria-controls="logs-column-picker"
            class="btn btn-secondary"
        >Columns<span aria-hidden="true"> ▾</span></button>
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
    <div v-if="showColumnPicker" id="logs-column-picker" class="column-picker-dropdown" @keydown.esc="closeColumnPicker">
      <div v-for="col in allColumns" :key="col.id" class="column-picker-item">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="isColumnVisible(col.id)" @change="toggleColumn(col.id)" />
          {{ col.name }}
        </label>
      </div>
    </div>

    <!-- What the keyboard is on in the grid, for screen readers: SlickGrid keeps focus on a
         hidden element rather than on the cell itself -->
    <div class="sr-only" aria-live="polite">{{ gridAnnouncement }}</div>

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
          role="button"
          tabindex="0"
          @click="selectedRow = log"
          @keydown.enter.prevent="selectedRow = log"
          @keydown.space.prevent="selectedRow = log"
      >
        <div class="mobile-log-card__header">
          <span class="mobile-log-card__time">{{ log.timestamp }}</span>
          <span class="level-badge" :class="'level-' + (log.level || 'unknown').toLowerCase()">{{ log.level || 'Unknown' }}</span>
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
    <ModalDialog v-if="selectedRow" title="Log Detail" wide body-class="modal-body--rows" @close="closeDetail">
      <template #actions>
        <button
          v-if="uploadedIdl"
          type="button"
          @click="$emit('decode-with-idl', selectedRow); closeDetail()"
          class="btn btn-info"
        ><span aria-hidden="true">🔍 </span>Decode with IDL</button>
      </template>
      <div v-for="col in allColumns" :key="col.id" class="detail-row">
        <span class="detail-label">{{ col.name }}</span>
        <span class="detail-value">
          <a
              v-if="detailLink(col.field, selectedRow[col.field])"
              :href="detailLink(col.field, selectedRow[col.field])"
              target="_blank"
              rel="noopener"
          >{{ formatDetailValue(col.field, selectedRow[col.field]) }}</a>
          <template v-else>{{ formatDetailValue(col.field, selectedRow[col.field]) }}</template>
        </span>
      </div>
    </ModalDialog>
  </div>
</template>

<script>
import { SlickGrid, SlickRowSelectionModel } from 'slickgrid';
import 'slickgrid/dist/styles/css/slick.grid.css';
import 'slickgrid/dist/styles/css/slick-default-theme.css';
import ModalDialog from './ModalDialog.vue';

const ALL_COLUMNS = [
  {
    id: 'timestamp', name: 'Time', field: 'timestamp', width: 80, sortable: true, resizable: true,
    toolTip: 'Arrival time in this browser, not on-chain time'
  },
  {
    id: 'level', name: 'Level', field: 'level', width: 100, sortable: true, resizable: true,
    formatter: (_, __, value) => {
      const level = String(value ?? 'Unknown');
      return `<span class="level-badge level-${escapeHtml(level.toLowerCase())}">${escapeHtml(level)}</span>`;
    }
  },
  {
    id: 'signature', name: 'Signature', field: 'signature', width: 110, resizable: true,
    formatter: (_, __, value) => {
      const sig = String(value?.signature ?? value ?? '');
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'tx');
      return `<a href="${escapeHtml(base + sig + suffix)}" tabindex="-1" target="_blank" rel="noopener" class="truncate-cell" title="${escapeHtml(sig)}">${escapeHtml(sig.substring(0, 8))}...</a>`;
    }
  },
  {
    id: 'slot', name: 'Slot', field: 'slot', width: 100, sortable: true, resizable: true,
    formatter: (_, __, value) => {
      const slot = String(value?.slot ?? value ?? '');
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'block');
      return `<a href="${escapeHtml(base + slot + suffix)}" tabindex="-1" target="_blank" rel="noopener">${escapeHtml(slot)}</a>`;
    }
  },
  {
    id: 'programId', name: 'Program', field: 'programId', width: 110, resizable: true,
    formatter: (_, __, value, _col, dataContext) => {
      const pid = String(value?.programId ?? value ?? '');
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'account');
      const depth = Number(dataContext.depth) || 0;
      const indentStyle = depth > 0 ? `padding-left:${depth * 12}px;` : '';
      const childClass = depth > 0 ? ' cpi-child' : '';
      return `<a href="${escapeHtml(base + pid + suffix)}" tabindex="-1" target="_blank" rel="noopener" class="truncate-cell${childClass}" style="${indentStyle}" title="${escapeHtml(pid)}">${escapeHtml(pid.substring(0, 8))}...</a>`;
    }
  },
  {
    id: 'parentProgramId', name: 'Parent', field: 'parentProgramId', width: 110, resizable: true,
    formatter: (_, __, value) => {
      const pid = String(value?.parentProgramId ?? value ?? '');
      const suffix = value?.linkSuffix ?? '';
      const base = explorerBase(value?.explorer, 'account');
      if (!pid) return '<span class="cu-na">—</span>';
      return `<a href="${escapeHtml(base + pid + suffix)}" tabindex="-1" target="_blank" rel="noopener" class="truncate-cell cpi-parent" title="${escapeHtml(pid)}">${escapeHtml(pid.substring(0, 8))}...</a>`;
    }
  },
  {
    id: 'depth', name: 'Depth', field: 'depth', width: 60, sortable: true, resizable: true,
    formatter: (_, __, value) => {
      const depth = Number(value) || 0;
      const indent = '&nbsp;&nbsp;&nbsp;'.repeat(depth);
      const badge = `<span class="cpi-depth-badge depth-${Math.min(depth, 5)}">${depth}</span>`;
      return `${indent}${badge}`;
    }
  },
  { id: 'instructionIndex', name: 'Idx', field: 'instructionIndex', width: 60, sortable: true, resizable: true },
  {
    id: 'instructionName', name: 'Instruction', field: 'instructionName', width: 130, sortable: true, resizable: true,
    toolTip: 'Anchor instruction name parsed from "Program log: Instruction: ..."',
    formatter: (_, __, value) => {
      const name = String(value ?? '');
      if (!name) return '<span class="cu-na">—</span>';
      return `<span class="truncate-cell" title="${escapeHtml(name)}">${escapeHtml(name)}</span>`;
    }
  },
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
    asyncPostRender: (cellNode, _row, dataContext) => renderJsonArrayCell(cellNode, dataContext.logMessages)
  },
  {
    id: 'decodedEvents', name: 'Events', field: 'decodedEvents', width: 200, resizable: true,
    toolTip: 'Anchor events decoded from Program data logs via the IDL',
    asyncPostRender: (cellNode, _row, dataContext) => renderJsonArrayCell(cellNode, dataContext.decodedEvents)
  },
  {
    id: 'dataLogs', name: 'Data', field: 'dataLogs', width: 150, resizable: true,
    formatter: (_, __, value) => {
      const text = escapeHtml(value);
      return `<div class="scrollable-cell" title="${text}">${text}</div>`;
    }
  },
  {
    id: 'rawLogs', name: 'Raw Logs', field: 'rawLogs', width: 150, resizable: true,
    asyncPostRender: (cellNode, _row, dataContext) => renderJsonArrayCell(cellNode, dataContext.rawLogs)
  },
  {
    id: 'errors', name: 'Errors', field: 'errors', width: 150, resizable: true,
    formatter: (_, __, value) => {
      const text = escapeHtml(value);
      return `<div class="scrollable-cell" title="${text}">${text}</div>`;
    }
  },
  {
    id: 'errorCode', name: 'Error Code', field: 'errorCode', width: 130, sortable: true, resizable: true,
    toolTip: 'Numeric code from "custom program error: 0x…", with the IDL-resolved name when available',
    formatter: (_, __, value, _col, dataContext) => {
      if (value === null || value === undefined) return '<span class="cu-na">—</span>';
      const code = Number(value);
      const name = dataContext.errorName ? ` ${escapeHtml(dataContext.errorName)}` : '';
      return `<span class="cu-value cu-high" title="0x${code.toString(16)}">${code}${name}</span>`;
    }
  },
  {
    id: 'errorName', name: 'Error Name', field: 'errorName', width: 130, sortable: true, resizable: true,
    toolTip: 'Error name resolved from the IDL errors array',
    formatter: (_, __, value) => {
      if (!value) return '<span class="cu-na">—</span>';
      return `<span class="cu-value cu-high">${escapeHtml(value)}</span>`;
    }
  },
  {
    id: 'transactionError', name: 'TX Error', field: 'transactionError', width: 150, resizable: true,
    formatter: (_, __, value) => {
      const text = escapeHtml(value);
      return `<div class="scrollable-cell" title="${text}">${text}</div>`;
    }
  }
];

const DEFAULT_HIDDEN = new Set(['rawLogs', 'dataLogs', 'parentProgramId', 'errorName']);

const EXPLORER_URLS = {
  solscan:  { tx: 'https://solscan.io/tx/', block: 'https://solscan.io/block/', account: 'https://solscan.io/account/' },
  solana:   { tx: 'https://explorer.solana.com/tx/', block: 'https://explorer.solana.com/block/', account: 'https://explorer.solana.com/address/' },
  orb:      { tx: 'https://orbmarkets.io/tx/', block: 'https://orbmarkets.io/block/', account: 'https://orbmarkets.io/address/' },
};

function explorerBase(explorer, type) {
  return (EXPLORER_URLS[explorer] ?? EXPLORER_URLS.solscan)[type];
}

// Fields shown as explorer links in the grid, and so in the detail dialog too
const DETAIL_LINK_TYPES = {signature: 'tx', slot: 'block', programId: 'account', parentProgramId: 'account'};

// Cell content comes from on-chain program logs — attacker-controlled strings. Everything a
// formatter interpolates into grid HTML must pass through here.
function escapeHtml(value) {
  return String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

// asyncPostRender cells bypass string formatting entirely: textContent can't be injected.
function renderJsonArrayCell(cellNode, value) {
  let content;
  try {
    content = JSON.parse(value).join(' | ');
  } catch {
    content = value ?? '';
  }
  const div = document.createElement('div');
  div.className = 'scrollable-cell';
  div.title = content;
  div.textContent = content;
  cellNode.replaceChildren(div);
}

export default {
  components: {ModalDialog},
  props: ['parsedLogs', 'uploadedIdl'],
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
      gridAnnouncement: '',
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
    closeDetail() {
      this.selectedRow = null;
    },
    openDetail(row) {
      const item = this.grid.getDataItem(row);
      if (item) this.selectedRow = item;
    },
    detailLink(field, value) {
      const type = DETAIL_LINK_TYPES[field];
      const id = value?.[field];
      if (!type || id === undefined || id === null || id === '') return null;
      return explorerBase(value.explorer, type) + id + (value.linkSuffix ?? '');
    },
    closeColumnPicker() {
      if (!this.showColumnPicker) return;
      this.showColumnPicker = false;
      this.$refs.columnsButton.focus();
    },
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
        this.decorateHeaders();
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

      // Row selection model: highlights the active row. Details open on a click or Enter,
      // not on selection, which also follows the arrow keys.
      const selectionModel = new SlickRowSelectionModel();
      this.grid.setSelectionModel(selectionModel);
      this.grid.onClick.subscribe((e, args) => {
        const target = (e.getNativeEvent?.() ?? e).target;
        if (target?.closest?.('a')) return; // an explorer link opens on its own
        this.openDetail(args.row);
      });
      this.grid.onKeyDown.subscribe((e, args) => {
        if (e.key === 'Enter' && args.row !== undefined && args.row !== null && args.row >= 0) {
          this.openDetail(args.row);
          e.stopImmediatePropagation();
        }
      });
      this.grid.onActiveCellChanged.subscribe((e, args) => this.announceCell(args.row, args.cell));

      // Column sorting
      this.grid.onSort.subscribe((e, args) => {
        this.sortColumns = args.sortCols;
        const data = this.getSortedData();
        this.grid.setData(data, true);
        this.grid.render();
        this.decorateHeaders();
      });

      this.setUpKeyboardAccess();
      this.decorateHeaders();
    },
    // Keyboard model (WAI-ARIA grid): sortable headers are Tab stops, the rows are one Tab
    // stop, arrow keys move between cells, Enter opens the row's details, Tab leaves.
    // SlickGrid makes its panes, viewports, canvases and cell links tabbable and moves
    // between cells on Tab, so a keyboard user could not get past it.
    setUpKeyboardAccess() {
      const container = this.$refs.gridContainer;
      // SlickGrid's focus catchers are the container's first and last children; the last
      // one is the rows' Tab stop
      const [leadingSink, ...rest] = [...container.children].filter(el => !el.classList.contains('slick-pane'));
      const rowsStop = rest.at(-1);
      for (const element of container.querySelectorAll('.slick-pane, .slick-viewport, .grid-canvas')) {
        element.tabIndex = -1;
      }
      leadingSink.tabIndex = -1;
      leadingSink.dataset.gridFocus = 'true';
      rowsStop.dataset.gridFocus = 'true';

      // Keep Tab from SlickGrid so the browser moves focus normally. SlickGrid parks focus
      // on the leading catcher after Up/Left; treat it as the rows stop.
      container.addEventListener('keydown', event => {
        if (event.key !== 'Tab') return;
        event.stopPropagation();
        if (document.activeElement === leadingSink) rowsStop.focus();
      }, true);

      // Arriving by keyboard from outside the rows (a header or the page): start on a cell
      // (arrow keys need an active one) and say how to use the grid. Ignore SlickGrid moving
      // focus between its catchers while navigating.
      rowsStop.addEventListener('focus', event => {
        const fromRows = event.relatedTarget === leadingSink || event.relatedTarget?.closest?.('.grid-canvas');
        if (fromRows || !rowsStop.matches(':focus-visible')) return;
        if (!this.grid.getActiveCell() && this.grid.getDataLength() > 0) this.grid.setActiveCell(0, 0);
        const cell = this.grid.getActiveCell();
        this.announceCell(cell?.row, cell?.cell, 'Log rows: arrow keys move between cells, Enter opens details. ');
      });
    },
    announceCell(row, cell, prefix = '') {
      if (row === undefined || row === null || row < 0) {
        this.gridAnnouncement = prefix.trim();
        return;
      }
      const column = this.grid.getColumns()[cell];
      const text = this.grid.getCellNode(row, cell)?.textContent.trim() || 'empty';
      this.gridAnnouncement = `${prefix}${column?.name ?? ''}: ${text}. Row ${row + 1} of ${this.grid.getDataLength()}.`;
    },
    // Focusing a header scrolls the header strip by itself, leaving the columns misaligned
    // with the rows. Scroll the rows to match; SlickGrid then re-syncs the headers from them.
    alignRowsWithHeader(node) {
      const side = node.closest('.slick-pane-right') ? 'right' : 'left';
      const header = node.closest('.slick-header');
      const rows = this.$refs.gridContainer.querySelector(`.slick-pane-top.slick-pane-${side} .slick-viewport`);
      if (header && rows && rows.scrollLeft !== header.scrollLeft) rows.scrollLeft = header.scrollLeft;
    },
    // Sortable headers: focusable, sorted with Enter or Space (as a click), and aria-sort
    // tells screen readers the current order. Headers are rebuilt when columns change.
    decorateHeaders() {
      const uid = this.grid.getUID();
      for (const node of this.$refs.gridContainer.querySelectorAll('.slick-header-column')) {
        const column = ALL_COLUMNS.find(c => uid + c.id === node.id);
        if (!column?.sortable) continue;
        node.tabIndex = 0;
        const sort = this.sortColumns.find(s => (s.sortCol?.id ?? s.columnId) === column.id);
        node.setAttribute('aria-sort', sort ? (sort.sortAsc ? 'ascending' : 'descending') : 'none');
        if (node.dataset.keyboardSort) continue;
        node.dataset.keyboardSort = 'true';
        node.addEventListener('focus', () => requestAnimationFrame(() => this.alignRowsWithHeader(node)));
        node.addEventListener('keydown', event => {
          if (event.key !== 'Enter' && event.key !== ' ') return;
          event.preventDefault();
          node.click();
        });
      }
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
        // Newest logs are unshifted to index 0, so following the stream means scrolling to the top.
        if (this.autoScroll && data.length > 0) {
          this.grid.scrollRowIntoView(0, false);
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
/* Unscoped on purpose: SlickGrid renders cells via innerHTML, which scoped styles cannot reach. */

/* Keyboard focus: sortable headers, the active cell while the rows have focus, cards */
.slick-header-column:focus-visible {
  outline: 2px solid var(--p-primary-color);
  outline-offset: -2px;
}

/* The active cell sits in the highlighted (primary-colored) row, so outline it in the
   row's contrast color */
.slick-grid-container:has([data-grid-focus]:focus-visible) .slick-cell.active {
  outline: 2px solid var(--p-primary-contrast-color);
  outline-offset: -4px;
}

/* Links and colored values in the highlighted row take its contrast color; in their own
   colors they can match the row's background */
.slick-grid-container .slick-cell.selected a,
.slick-grid-container .slick-cell.selected .cu-value,
.slick-grid-container .slick-cell.selected .cu-na {
  color: inherit !important;
}

.slick-grid-container .slick-cell.selected a {
  text-decoration: underline;
}

/* Badges have translucent tints; give them the row's usual base in the highlighted row */
.slick-grid-container .slick-cell.selected .level-badge,
.slick-grid-container .slick-cell.selected .cpi-depth-badge {
  background-color: var(--p-card-bg);
}

.mobile-log-card:focus-visible {
  outline: 2px solid var(--p-primary-color);
  outline-offset: 2px;
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

/* Override SlickGrid default theme to match app theme */
.slick-grid-container .slick-header {
  background: var(--p-table-header-bg) !important;
  border-bottom: 1px solid var(--p-card-border);
}

.slick-grid-container .slick-header-column {
  background: var(--p-table-header-bg) !important;
  color: var(--p-table-header-text) !important;
  border-right: 1px solid var(--p-card-border) !important;
  font-weight: 600;
}

.slick-grid-container .slick-header-column:hover {
  background: var(--p-table-header-hover) !important;
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
  background: var(--p-row-stripe);
}

.slick-grid-container .slick-row:hover .slick-cell {
  background: var(--p-row-hover) !important;
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

.level-badge {
  display: inline-block;
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

@media (max-width: 768px) {
  .logs-table-root {
    overflow-x: hidden;
  }
}
</style>
