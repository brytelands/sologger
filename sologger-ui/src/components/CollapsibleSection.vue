<template>
  <div :class="bare ? '' : 'card'">
    <button
        type="button"
        class="collapsible-toggle"
        :class="{ 'collapsible-toggle--bare': bare }"
        :aria-expanded="open ? 'true' : 'false'"
        @click="toggle"
    >
      <span class="text-sm font-semibold uppercase tracking-wider text-[var(--p-text-muted)]">{{ title }}</span>
      <span class="flex items-center gap-2">
        <span v-if="badge" class="text-xs text-[var(--p-text-muted)]">{{ badge }}</span>
        <svg
            class="collapsible-chevron"
            :class="{ 'collapsible-chevron--open': open }"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            aria-hidden="true"
        >
          <path
              fill-rule="evenodd"
              d="M5.23 7.21a.75.75 0 011.06.02L10 11.17l3.71-3.94a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
              clip-rule="evenodd"
          />
        </svg>
      </span>
    </button>
    <div v-show="open" :class="bare ? '' : 'collapsible-body'">
      <slot/>
    </div>
  </div>
</template>

<script>
export default {
  props: {
    title: { type: String, required: true },
    // Persists open/closed across reloads under sologger_section_<storageKey>.
    storageKey: { type: String, default: '' },
    defaultOpen: { type: Boolean, default: true },
    // bare: no card chrome — for sections whose content brings its own cards.
    bare: { type: Boolean, default: false },
    // Optional hint shown next to the chevron (e.g. loaded-IDL filename).
    badge: { type: String, default: '' }
  },
  emits: [],
  data() {
    return { open: this.defaultOpen };
  },
  created() {
    if (this.storageKey) {
      const saved = localStorage.getItem('sologger_section_' + this.storageKey);
      if (saved !== null) this.open = saved === 'true';
    }
  },
  methods: {
    toggle() {
      this.open = !this.open;
      if (this.storageKey) {
        localStorage.setItem('sologger_section_' + this.storageKey, String(this.open));
      }
    }
  }
};
</script>

<style scoped>
.collapsible-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0.75rem 1rem;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--p-text-color);
  text-align: left;
}

.collapsible-toggle--bare {
  padding: 0.25rem 0;
}

.collapsible-body {
  padding: 0 1rem 1rem;
}

.collapsible-chevron {
  width: 1rem;
  height: 1rem;
  color: var(--p-text-muted);
  transition: transform 0.15s ease;
}

.collapsible-chevron--open {
  transform: rotate(180deg);
}
</style>
