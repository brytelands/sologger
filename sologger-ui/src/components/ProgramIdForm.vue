<template>
  <form @submit.prevent="emitAddProgramId" class="mb-4">
    <div class="flex gap-3">
      <input
          :value="modelValue"
          @input="$emit('update:modelValue', $event.target.value)"
          type="text"
          placeholder="Enter Program ID"
          class="input-base flex-grow"
      />
      <button
          type="submit"
          class="btn btn-primary"
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
