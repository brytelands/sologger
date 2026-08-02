<template>
  <div class="mb-4" v-if="programs.length">
    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)] mb-2">Monitored Programs</h3>
    <div class="flex flex-wrap gap-2">
      <div
        v-for="(program, index) in programs"
        :key="program.id"
        class="flex items-center gap-2 bg-[var(--p-card-bg)] border border-[var(--p-card-border)] px-3 py-1.5 rounded-lg"
      >
        <span class="status-dot" :class="'status-dot--' + program.status"></span>
        <span class="text-sm font-mono text-[var(--p-text-color)] truncate max-w-[150px] md:max-w-none">{{ program.id }}</span>
        <span
          class="text-xs text-[var(--p-text-muted)] whitespace-nowrap"
          :title="program.status === 'awaiting' ? 'It may take several seconds for the first logs to arrive' : null"
        >{{ statusLabel(program.status) }}</span>
        <button
          @click="$emit('removeProgramId', index)"
          class="text-[var(--p-text-muted)] hover:text-red-500 transition-colors leading-none text-lg"
          :aria-label="'Stop monitoring ' + program.id"
          title="Remove"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>

<script>
const STATUS_LABELS = {
  connecting: 'Connecting…',
  connected: 'Connected',
  awaiting: 'Awaiting logs…',
  disconnected: 'Disconnected'
};

export default {
  props: ['programs'],
  emits: ['removeProgramId'],
  methods: {
    statusLabel(status) {
      return STATUS_LABELS[status] ?? status;
    }
  }
};
</script>

<style scoped>
.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 9999px;
  flex-shrink: 0;
}

.status-dot--connected { background: #22c55e; }
.status-dot--awaiting { background: #f59e0b; }
.status-dot--disconnected { background: #6b7280; }

.status-dot--connecting {
  background: var(--p-primary-color);
  animation: status-pulse 1.2s ease-in-out infinite;
}

@keyframes status-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}
</style>
