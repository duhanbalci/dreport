<script setup lang="ts">
defineProps<{
  label: string
  modelValue: boolean
  dataTip?: string
}>()

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>

<template>
  <div class="prop-row" :data-tip="dataTip">
    <label class="prop-label">{{ label }}</label>
    <input
      type="checkbox"
      :checked="modelValue"
      @change="(e) => emit('update:modelValue', (e.target as HTMLInputElement).checked)"
    />
  </div>
</template>
