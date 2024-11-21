<template>
  <div class="mb-6">
    <h3 class="text-lg font-bold mb-2">Monitored Programs:</h3>
    <div class="flex flex-wrap gap-2">
      <div
        v-for="(id, index) in programIds"
        :key="index"
        class="flex items-center gap-2 bg-gray-100 px-3 py-1 rounded"
      >
        <span class="text-sm">{{ id }}</span>
        <button
          @click="removeProgramId(index)"
          class="text-red-500 hover:text-red-700"
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