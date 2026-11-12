<template>
  <div class="modal-overlay" @click.self="close">
    <div
        ref="panel"
        class="modal-panel"
        :class="{'modal-panel--wide': wide}"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="titleId"
        tabindex="-1"
    >
      <div class="modal-header">
        <h2 :id="titleId" class="font-semibold text-base">
          <span v-if="icon" aria-hidden="true">{{ icon }}&nbsp;</span>{{ title }}
        </h2>
        <div class="flex gap-2">
          <slot name="actions"/>
          <button type="button" class="btn btn-secondary" aria-label="Close" @click="close">{{ closeText }}</button>
        </div>
      </div>
      <div class="modal-body" :class="bodyClass">
        <slot/>
      </div>
    </div>
  </div>
</template>

<script>
// A modal dialog, rendered while the parent keeps it mounted (v-if). It is announced as a
// dialog named by its title, moves keyboard focus inside on open, keeps Tab / Shift+Tab
// inside, closes on Escape or a click on the backdrop (emitting `close`), and returns
// focus to whatever had it before opening.
const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])'
].join(', ');

// Open dialogs, newest last: only the top one reacts to the keyboard
const openDialogs = [];
let nextId = 0;

export default {
  props: {
    title: {type: String, required: true},
    // Decorative prefix such as an emoji; hidden from screen readers
    icon: {type: String, default: ''},
    wide: {type: Boolean, default: false},
    bodyClass: {type: String, default: ''},
    closeText: {type: String, default: '✕ Close'}
  },
  emits: ['close'],
  data() {
    return {titleId: `modal-title-${++nextId}`};
  },
  mounted() {
    this.returnFocus = document.activeElement;
    openDialogs.push(this);
    document.addEventListener('keydown', this.onKeydown);
    (this.focusables()[0] ?? this.$refs.panel).focus();
  },
  beforeUnmount() {
    document.removeEventListener('keydown', this.onKeydown);
    openDialogs.splice(openDialogs.indexOf(this), 1);
    if (this.returnFocus?.isConnected && typeof this.returnFocus.focus === 'function') {
      this.returnFocus.focus();
    }
  },
  methods: {
    close() {
      this.$emit('close');
    },
    focusables() {
      return [...this.$refs.panel.querySelectorAll(FOCUSABLE)]
          .filter(element => element.getClientRects().length > 0);
    },
    onKeydown(event) {
      if (openDialogs.at(-1) !== this) return;
      if (event.key === 'Escape') {
        event.preventDefault();
        this.close();
        return;
      }
      if (event.key !== 'Tab') return;

      const items = this.focusables();
      const panel = this.$refs.panel;
      if (!items.length) {
        event.preventDefault();
        panel.focus();
        return;
      }
      const first = items[0];
      const last = items.at(-1);
      const active = document.activeElement;
      const outside = !panel.contains(active) || active === panel;
      if (event.shiftKey && (active === first || outside)) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && (active === last || outside)) {
        event.preventDefault();
        first.focus();
      }
    }
  }
};
</script>
