<template>
  <div class="mb-4" v-if="programIds.length">
    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--p-text-muted)] mb-2">Monitored Programs</h3>
    <div class="flex flex-wrap gap-2">
      <div
        v-for="(id, index) in programIds"
        :key="index"
        class="flex items-center gap-2 bg-[var(--p-card-bg)] border border-[var(--p-card-border)] px-3 py-1.5 rounded-lg"
      >
        <span class="text-sm font-mono text-[var(--p-text-color)]">{{ id }}</span>
        <button
          @click="removeProgramId(index)"
          class="text-[var(--p-text-muted)] hover:text-red-500 transition-colors leading-none text-lg"
          title="Remove"
        >
          ×
        </button>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  props: ['programIds'],
  emits: ['removeProgramId'],
  methods: {
    removeProgramId(index) {
      this.$emit('removeProgramId', index);
    }
  }
};
</script>
