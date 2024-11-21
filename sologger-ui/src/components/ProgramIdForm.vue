<template>
  <form @submit.prevent="emitAddProgramId" class="mb-6">
    <div class="flex gap-4">
      <input
          :value="modelValue"
          @input="$emit('update:modelValue', $event.target.value)"
          type="text"
          placeholder="Enter Program ID"
          class="flex-grow p-2 border rounded"
      />
      <button
          type="submit"
          class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
      >
        Add Program
      </button>
    </div>
  </form>
</template>

<script>
export default {
  props: {
    modelValue: {
      type: String,
      required: true
    }
  },
  emits: ['addProgramId', 'update:modelValue'],
  methods: {
    emitAddProgramId() {
      this.$emit('addProgramId');
    }
  }
};
</script>