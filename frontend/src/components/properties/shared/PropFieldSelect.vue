<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    modelValue: string
    fields: Array<{ path?: string; key?: string; title?: string; type?: string }>
    placeholder?: string
    allowEmpty?: boolean
    emptyLabel?: string
    dataTip?: string
  }>(),
  { placeholder: 'Secin...', allowEmpty: false, emptyLabel: 'Yok' },
)

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
      <option v-if="allowEmpty" value="">{{ emptyLabel }}</option>
      <option v-else value="" disabled>{{ placeholder }}</option>
      <option
        v-for="field in fields"
        :key="field.path ?? field.key"
        :value="field.path ?? field.key"
      >
        {{ field.title ?? field.path ?? field.key }}
        <template v-if="field.path">({{ field.path }})</template>
      </option>
    </select>
  </div>
</template>
