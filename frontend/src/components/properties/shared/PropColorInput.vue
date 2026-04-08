<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    modelValue: string | undefined
    defaultColor?: string
    clearable?: boolean
    dataTip?: string
  }>(),
  { defaultColor: '#000000', clearable: false },
)

const emit = defineEmits<{ 'update:modelValue': [value: string | undefined] }>()
</script>

<template>
  <div class="prop-row" :data-tip="dataTip">
    <label class="prop-label">{{ label }}</label>
    <div v-if="clearable" class="prop-row-inline">
      <input
        class="prop-input prop-color"
        type="color"
        :value="modelValue ?? defaultColor"
        @input="(e) => emit('update:modelValue', (e.target as HTMLInputElement).value)"
      />
      <button v-if="modelValue" class="prop-clear" @click="emit('update:modelValue', undefined)">
        x
      </button>
    </div>
    <input
      v-else
      class="prop-input prop-color"
      type="color"
      :value="modelValue ?? defaultColor"
      @input="(e) => emit('update:modelValue', (e.target as HTMLInputElement).value)"
    />
  </div>
</template>
