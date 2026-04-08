<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    modelValue: number
    step?: number
    min?: number
    max?: number
    dataTip?: string
  }>(),
  { step: 1, min: 0 },
)

const emit = defineEmits<{ 'update:modelValue': [value: number] }>()

function onInput(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value)
  if (!isNaN(val)) emit('update:modelValue', val)
}
</script>

<template>
  <div class="prop-row" :data-tip="dataTip">
    <label class="prop-label">{{ label }}</label>
    <input
      class="prop-input"
      type="number"
      :step="step"
      :min="min"
      :max="max"
      :value="modelValue"
      @input="onInput"
    />
  </div>
</template>
