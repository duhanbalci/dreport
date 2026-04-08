<script setup lang="ts">
import { computed } from 'vue'
import { useTemplateStore } from '../../../stores/template'
import PropNumberInput from './PropNumberInput.vue'
import PropColorInput from './PropColorInput.vue'
import PropSelect from './PropSelect.vue'

const props = withDefaults(
  defineProps<{
    fontSize: number
    fontWeight?: string
    fontFamily?: string
    color: string
    align: string
    showWeight?: boolean
  }>(),
  { fontWeight: 'normal', fontFamily: undefined, showWeight: true },
)

defineEmits<{
  'update:fontSize': [value: number]
  'update:fontWeight': [value: string]
  'update:fontFamily': [value: string | undefined]
  'update:color': [value: string]
  'update:align': [value: string]
}>()

const templateStore = useTemplateStore()

const fontOptions = computed(() =>
  templateStore.template.fonts.map((f) => ({ value: f, label: f })),
)

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
  <PropSelect
    v-if="fontOptions.length > 1"
    label="Font"
    :model-value="fontFamily ?? fontOptions[0]?.value ?? ''"
    :options="fontOptions"
    data-tip="Yazi tipi ailesi"
    @update:model-value="$emit('update:fontFamily', $event)"
  />
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
