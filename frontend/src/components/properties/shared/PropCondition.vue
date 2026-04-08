<script setup lang="ts">
import { computed } from 'vue'
import { useSchemaStore } from '../../../stores/schema'
import PropFieldSelect from './PropFieldSelect.vue'
import PropSelect from './PropSelect.vue'
import PropSection from './PropSection.vue'
import type { Condition } from '../../../core/types'
import '../../../styles/properties.css'

const props = defineProps<{
  condition?: Condition
}>()

const emit = defineEmits<{
  'update:condition': [value: Condition | undefined]
}>()

const schemaStore = useSchemaStore()

const enabled = computed(() => !!props.condition)

const operatorOptions = [
  { value: 'eq', label: '= Esit' },
  { value: 'neq', label: '≠ Esit Degil' },
  { value: 'gt', label: '> Buyuk' },
  { value: 'gte', label: '>= Buyuk Esit' },
  { value: 'lt', label: '< Kucuk' },
  { value: 'lte', label: '<= Kucuk Esit' },
  { value: 'truthy', label: 'Dolu (truthy)' },
  { value: 'falsy', label: 'Bos (falsy)' },
]

const needsValue = computed(() => {
  const op = props.condition?.operator
  return op && op !== 'truthy' && op !== 'falsy'
})

function toggle(on: boolean) {
  if (on) {
    emit('update:condition', { path: '', operator: 'truthy' })
  } else {
    emit('update:condition', undefined)
  }
}

function updateField(key: keyof Condition, value: unknown) {
  emit('update:condition', { ...props.condition!, [key]: value })
}
</script>

<template>
  <PropSection title="Kosullu Gosterim">
    <div class="prop-row" data-tip="Elemani belirli bir kosulla goster/gizle">
      <label class="prop-label">Aktif</label>
      <input type="checkbox" :checked="enabled" @change="toggle(($event.target as HTMLInputElement).checked)" />
    </div>

    <template v-if="enabled">
      <PropFieldSelect
        label="Alan"
        :model-value="condition!.path"
        :fields="schemaStore.scalarFields"
        data-tip="Kosulun degerlendirilecegi veri alani"
        @update:model-value="(v) => updateField('path', v)"
      />
      <PropSelect
        label="Operator"
        :model-value="condition!.operator"
        :options="operatorOptions"
        data-tip="Karsilastirma operatoru"
        @update:model-value="(v) => updateField('operator', v)"
      />
      <div v-if="needsValue" class="prop-row" data-tip="Karsilastirilacak deger">
        <label class="prop-label">Deger</label>
        <input
          class="prop-input"
          type="text"
          :value="condition!.value ?? ''"
          @input="(e) => updateField('value', (e.target as HTMLInputElement).value)"
        />
      </div>
    </template>
  </PropSection>
</template>
