<template>
  <div class="cu-flame" v-if="cells.length">
    <svg
        :viewBox="`0 0 1000 ${svgHeight}`"
        preserveAspectRatio="none"
        class="cu-flame__svg"
        :style="{height: svgHeight + 'px'}"
        role="img"
        aria-label="Compute unit breakdown by program and CPI depth"
        @mouseleave="hovered = null"
    >
      <g v-for="cell in cells" :key="cell.key">
        <rect
            :x="cell.x"
            :y="cell.y"
            :width="cell.width"
            :height="rowHeight - 4"
            rx="3"
            :class="['cu-flame__cell', `cu-flame__cell--s${cell.slot}`]"
            @mousemove="onHover(cell, $event)"
            @mouseleave="hovered = null"
        />
        <text
            v-if="cell.width > 90"
            :x="cell.x + 6"
            :y="cell.y + (rowHeight - 4) / 2 + 4"
            class="cu-flame__label"
        >{{ cell.label }}</text>
      </g>
    </svg>

    <!-- Legend: identity is never color-alone -->
    <div v-if="legend.length > 1" class="cu-flame__legend">
      <span v-for="entry in legend" :key="entry.program" class="cu-flame__legend-item">
        <span :class="['cu-flame__swatch', `cu-flame__cell--s${entry.slot}`]"></span>
        <span class="cu-flame__legend-text">{{ entry.short }}</span>
      </span>
    </div>

    <!-- Hover tooltip -->
    <div
        v-if="hovered"
        class="cu-flame__tooltip"
        :style="{left: tooltipX + 'px', top: tooltipY + 'px'}"
    >
      <div class="cu-flame__tooltip-title">
        {{ hovered.failed ? '✗ ' : '' }}{{ hovered.program }}
      </div>
      <div v-if="hovered.instruction" class="cu-flame__tooltip-row">{{ hovered.instruction }}</div>
      <div class="cu-flame__tooltip-row">
        {{ hovered.cu === null ? 'CU not reported' : hovered.cu.toLocaleString() + ' CU (' + hovered.share + '% of tx)' }}
      </div>
      <div class="cu-flame__tooltip-row">depth {{ hovered.depth }}</div>
      <div v-for="(err, i) in hovered.errors" :key="i" class="cu-flame__tooltip-row cu-flame__tooltip-row--error">
        ✗ {{ err }}
      </div>
    </div>
  </div>
  <div v-else class="cu-flame cu-flame--empty">No compute-unit data for this transaction yet.</div>
</template>

<script>
// Icicle-style CU breakdown of one transaction's CPI tree. Depth is the row, width is
// consumed compute units (a parent's CU includes its children, so children nest inside
// the parent's span). Hue follows the program — fixed first-appearance order, folding
// into a neutral "Other" past eight — and failures are marked with a ✗ glyph, never by
// repainting the identity hue. Layout math lives in useCuFlamegraph.js.
import {buildFlamegraphCells, flamegraphLegend, FLAME_ROW_HEIGHT} from '../composables/useCuFlamegraph';

export default {
  name: 'CuFlamegraph',
  props: {
    // LogsTable-shaped rows of a single transaction, in invoke order
    rows: {type: Array, required: true}
  },
  data() {
    return {
      hovered: null,
      tooltipX: 0,
      tooltipY: 0,
    };
  },
  computed: {
    rowHeight() {
      return FLAME_ROW_HEIGHT;
    },
    cells() {
      return buildFlamegraphCells(this.rows);
    },
    legend() {
      return flamegraphLegend(this.rows);
    },
    svgHeight() {
      const maxDepth = this.cells.reduce((max, cell) => Math.max(max, cell.depth), 1);
      return maxDepth * FLAME_ROW_HEIGHT;
    },
  },
  methods: {
    onHover(cell, event) {
      this.hovered = cell;
      const bounds = this.$el.getBoundingClientRect();
      this.tooltipX = Math.min(event.clientX - bounds.left + 12, bounds.width - 240);
      this.tooltipY = event.clientY - bounds.top + 14;
    },
  },
};
</script>

<style>
/* Unscoped with a .cu-flame namespace: palette slots follow the palette.md pattern —
   values swap per theme in one place, marks reference roles. Both palettes validated
   against the app's card surfaces (#f0f7f3 light / #021b17 dark). Light mode's
   sub-3:1 slots rely on the shipped relief: direct labels, legend, surface gaps. */
.cu-flame {
  position: relative;
  /* surface gap color = the card the chart sits on */
  --cu-surface: var(--p-card-bg);
  --cu-text: var(--p-text-color);
  --cu-s1: #2a78d6;
  --cu-s2: #eb6834;
  --cu-s3: #1baf7a;
  --cu-s4: #eda100;
  --cu-s5: #e87ba4;
  --cu-s6: #008300;
  --cu-s7: #4a3aa7;
  --cu-s8: #e34948;
  --cu-s0: #6f6e6a; /* "Other" fold bucket */
  --cu-label: #0b0b0b;
}

:root[data-theme="dark"] .cu-flame {
  --cu-s1: #3987e5;
  --cu-s2: #d95926;
  --cu-s3: #199e70;
  --cu-s4: #c98500;
  --cu-s5: #d55181;
  --cu-s6: #008300;
  --cu-s7: #9085e9;
  --cu-s8: #e66767;
  --cu-s0: #8f8e88;
  --cu-label: #ffffff;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) .cu-flame {
    --cu-s1: #3987e5;
    --cu-s2: #d95926;
    --cu-s3: #199e70;
    --cu-s4: #c98500;
    --cu-s5: #d55181;
    --cu-s6: #008300;
    --cu-s7: #9085e9;
    --cu-s8: #e66767;
    --cu-s0: #8f8e88;
    --cu-label: #ffffff;
  }
}

.cu-flame__svg {
  width: 100%;
  display: block;
}

/* 2px surface stroke = the mandated gap between touching fills */
.cu-flame__cell {
  stroke: var(--cu-surface);
  stroke-width: 2;
  cursor: pointer;
}

.cu-flame__cell:hover {
  filter: brightness(1.12);
}

.cu-flame__cell--s0 { fill: var(--cu-s0); }
.cu-flame__cell--s1 { fill: var(--cu-s1); }
.cu-flame__cell--s2 { fill: var(--cu-s2); }
.cu-flame__cell--s3 { fill: var(--cu-s3); }
.cu-flame__cell--s4 { fill: var(--cu-s4); }
.cu-flame__cell--s5 { fill: var(--cu-s5); }
.cu-flame__cell--s6 { fill: var(--cu-s6); }
.cu-flame__cell--s7 { fill: var(--cu-s7); }
.cu-flame__cell--s8 { fill: var(--cu-s8); }

.cu-flame__label {
  font-size: 11px;
  font-weight: 600;
  fill: var(--cu-label);
  pointer-events: none;
}

.cu-flame__legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 1rem;
  margin-top: 0.5rem;
}

.cu-flame__legend-item {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.cu-flame__swatch {
  width: 10px;
  height: 10px;
  border-radius: 3px;
  display: inline-block;
}

.cu-flame__legend-text {
  font-size: 0.75rem;
  color: var(--cu-text);
  font-family: ui-monospace, monospace;
}

.cu-flame__tooltip {
  position: absolute;
  z-index: 50;
  max-width: 240px;
  pointer-events: none;
  background: var(--p-card-bg);
  border: 1px solid var(--p-card-border);
  border-radius: 0.5rem;
  padding: 0.5rem 0.65rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
}

.cu-flame__tooltip-title {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--cu-text);
  font-family: ui-monospace, monospace;
  word-break: break-all;
}

.cu-flame__tooltip-row {
  font-size: 0.72rem;
  color: var(--p-text-muted);
}

.cu-flame__tooltip-row--error {
  color: #e34948;
}

.cu-flame--empty {
  font-size: 0.8rem;
  color: var(--p-text-muted);
  padding: 0.75rem 0;
}
</style>
