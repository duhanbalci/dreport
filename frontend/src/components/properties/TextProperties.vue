<script setup lang="ts">
import { computed } from 'vue'
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import { useSchemaStore } from '../../stores/schema'
import PropSection from './shared/PropSection.vue'
import PropFieldSelect from './shared/PropFieldSelect.vue'
import PropTextStyleGroup from './shared/PropTextStyleGroup.vue'
import type { StaticTextElement, TextElement, TextStyle, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const schemaStore = useSchemaStore()
const style = () => props.element.style as TextStyle

const isText = computed(() => props.element.type === 'text')
</script>

<template>
  <PropSection title="Metin">
    <div v-if="element.type === 'static_text'" class="prop-row" data-tip="Sabit metin icerigi">
      <label class="prop-label">Metin</label>
      <input
        class="prop-input"
        type="text"
        :value="(element as StaticTextElement).content"
        @input="(e) => update({ content: (e.target as HTMLInputElement).value } as any)"
      />
    </div>

    <template v-if="isText">
      <PropFieldSelect
        label="Veri Alani"
        :model-value="(element as TextElement).binding?.path ?? ''"
        :fields="schemaStore.scalarFields"
        data-tip="Metnin baglanacagi veri alani"
        @update:model-value="(v) => update({ binding: { type: 'scalar', path: v } } as any)"
      />
      <div class="prop-row" data-tip="Veri alaninin onune eklenecek sabit metin">
        <label class="prop-label">Ön Ek</label>
        <input
          class="prop-input"
          type="text"
          :value="(element as TextElement).content ?? ''"
          placeholder="ör: Fatura No: "
          @input="(e) => update({ content: (e.target as HTMLInputElement).value || undefined } as any)"
        />
      </div>
    </template>
  </PropSection>

  <PropSection title="Metin Stili">
    <PropTextStyleGroup
      :font-size="style().fontSize ?? 11"
      :font-weight="style().fontWeight ?? 'normal'"
      :font-family="style().fontFamily"
      :color="style().color ?? '#000000'"
      :align="style().align ?? 'left'"
      @update:font-size="(v) => updateStyle('fontSize', v)"
      @update:font-weight="(v) => updateStyle('fontWeight', v)"
      @update:font-family="(v) => updateStyle('fontFamily', v)"
      @update:color="(v) => updateStyle('color', v)"
      @update:align="(v) => updateStyle('align', v)"
    />
  </PropSection>
</template>
