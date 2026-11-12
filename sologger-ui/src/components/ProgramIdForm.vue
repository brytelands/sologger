<template>
  <form @submit.prevent="emitAddProgramId" class="mb-4">
    <div class="flex flex-col sm:flex-row gap-3">
      <select
          aria-label="Popular programs"
          title="Fill in the ID of a well-known program"
          class="input-base w-full sm:w-auto"
          @change="pickProgram"
      >
        <option value="">Popular programs…</option>
        <option v-for="program in popularPrograms" :key="program.id" :value="program.id">
          {{ program.name }}
        </option>
      </select>
      <div class="flex gap-3 flex-grow">
        <input
            :value="modelValue"
            @input="$emit('update:modelValue', $event.target.value)"
            type="text"
            placeholder="Enter Program ID"
            class="input-base flex-grow min-w-0"
        />
        <button
            type="submit"
            class="btn btn-primary"
        >
          Add Program
        </button>
      </div>
    </div>
  </form>
</template>

<script>
// Well-known programs with steady traffic on devnet, so the live stream shows logs soon
// after one is added. The IDs are the same on mainnet.
export const POPULAR_PROGRAMS = [
  {name: 'SPL Token', id: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'},
  {name: 'Token-2022', id: 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'},
  {name: 'Associated Token Account', id: 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'},
  {name: 'Memo', id: 'MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr'},
  {name: 'Metaplex Token Metadata', id: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'},
  {name: 'Orca Whirlpools', id: 'whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc'},
];

export default {
  props: {
    modelValue: {
      type: String,
      required: true
    }
  },
  emits: ['addProgramId', 'update:modelValue'],
  data() {
    return {popularPrograms: POPULAR_PROGRAMS};
  },
  methods: {
    emitAddProgramId() {
      this.$emit('addProgramId');
    },
    // Fills the Program ID field; the user still adds it with the button. The picker
    // resets so the same program can be picked again after editing the field.
    pickProgram(event) {
      if (event.target.value) {
        this.$emit('update:modelValue', event.target.value);
      }
      event.target.value = '';
    }
  }
};
</script>
