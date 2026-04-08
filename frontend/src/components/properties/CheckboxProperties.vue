<script setup lang="ts">
import { computed } from 'vue'
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import PropSection from './shared/PropSection.vue'
import PropNumberInput from './shared/PropNumberInput.vue'
import PropColorInput from './shared/PropColorInput.vue'
import PropCheckbox from './shared/PropCheckbox.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import type { CheckboxElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: CheckboxElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()

const booleanFields = computed(() =>
  schemaStore.scalarFields.filter((f) => f.type === 'boolean' || f.type === 'string'),
)
</script>

<template>
  <PropSection title="Onay Kutusu">
    <PropFieldSelect
      label="Veri Alani"
      :model-value="element.binding?.path ?? ''"
      :fields="booleanFields"
      :allow-empty="true"
      empty-label="Yok (statik)"
      data-tip="Onay durumunun gelecegi veri alani"
      @update:model-value="
        (v) =>
          update({
            binding: v ? { type: 'scalar', path: v } : undefined,
            checked: v ? undefined : element.checked ?? false,
          } as any)
      "
    />
    <PropCheckbox
      v-if="!element.binding"
      label="Isaretli"
      :model-value="element.checked ?? false"
      data-tip="Onay kutusunun varsayilan durumu"
      @update:model-value="(v) => update({ checked: v } as any)"
    />
    <PropNumberInput
      label="Boyut (mm)"
      :model-value="element.style.size ?? 4"
      :step="0.5"
      :min="1"
      data-tip="Onay kutusu boyutu (mm)"
      @update:model-value="(v) => updateStyle('size', v)"
    />
    <PropColorInput
      label="Isaret Rengi"
      :model-value="element.style.checkColor ?? '#000000'"
      data-tip="Isaret (tik) rengi"
      @update:model-value="(v) => updateStyle('checkColor', v)"
    />
    <PropColorInput
      label="Kenar Rengi"
      :model-value="element.style.borderColor ?? '#333333'"
      data-tip="Kutu kenarlik rengi"
      @update:model-value="(v) => updateStyle('borderColor', v)"
    />
    <PropNumberInput
      label="Kenar Kalinligi"
      :model-value="element.style.borderWidth ?? 0.3"
      :step="0.1"
      :min="0"
      data-tip="Kutu kenarlik kalinligi (mm)"
      @update:model-value="(v) => updateStyle('borderWidth', v)"
    />
  </PropSection>
</template>
