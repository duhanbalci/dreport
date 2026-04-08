<script setup lang="ts">
defineProps<{
  label: string
  modelValue: string
  options: Array<{ value: string; label: string }>
  dataTip?: string
}>()

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <div class="prop-row" :data-tip="dataTip">
    <label class="prop-label">{{ label }}</label>
    <select
      class="prop-input prop-select"
      :value="modelValue"
      @change="(e) => emit('update:modelValue', (e.target as HTMLSelectElement).value)"
    >
      <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
    </select>
  </div>
</template>
