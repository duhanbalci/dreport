<script setup lang="ts">
import PropNumberInput from './PropNumberInput.vue'
import PropColorInput from './PropColorInput.vue'
import PropSelect from './PropSelect.vue'

withDefaults(
  defineProps<{
    fontSize: number
    fontWeight?: string
    color: string
    align: string
    showWeight?: boolean
  }>(),
  { fontWeight: 'normal', showWeight: true },
)

defineEmits<{
  'update:fontSize': [value: number]
  'update:fontWeight': [value: string]
  'update:color': [value: string]
  'update:align': [value: string]
}>()

const weightOptions = [
  { value: 'normal', label: 'Normal' },
  { value: 'bold', label: 'Kalin' },
]

const alignOptions = [
  { value: 'left', label: 'Sol' },
  { value: 'center', label: 'Orta' },
  { value: 'right', label: 'Sag' },
]
</script>

<template>
  <PropNumberInput
    label="Boyut (pt)"
    :model-value="fontSize"
    :step="1"
    :min="1"
    data-tip="Yazi tipi boyutu (point)"
    @update:model-value="$emit('update:fontSize', $event)"
  />
  <PropSelect
    v-if="showWeight"
    label="Kalinlik"
    :model-value="fontWeight!"
    :options="weightOptions"
    data-tip="Yazi tipi kalinligi"
    @update:model-value="$emit('update:fontWeight', $event)"
  />
  <PropColorInput
    label="Renk"
    :model-value="color"
    data-tip="Metin rengi"
    @update:model-value="$emit('update:color', $event!)"
  />
  <PropSelect
    label="Hizalama"
    :model-value="align"
    :options="alignOptions"
    data-tip="Metnin yatay hizalamasi"
    @update:model-value="$emit('update:align', $event)"
  />
</template>
