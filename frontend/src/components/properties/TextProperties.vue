<script setup lang="ts">
import { usePropertyUpdate } from '../../composables/usePropertyUpdate'
import PropSection from './shared/PropSection.vue'
import PropTextStyleGroup from './shared/PropTextStyleGroup.vue'
import type { StaticTextElement, TextStyle, TemplateElement } from '../../core/types'
import '../../styles/properties.css'

const props = defineProps<{ element: TemplateElement }>()
const { update, updateStyle } = usePropertyUpdate(() => props.element)
const style = () => props.element.style as TextStyle
</script>

<template>
  <PropSection title="Metin Stili">
    <div v-if="element.type === 'static_text'" class="prop-row" data-tip="Sabit metin icerigi">
      <label class="prop-label">Metin</label>
      <input
        class="prop-input"
        type="text"
        :value="(element as StaticTextElement).content"
        @input="(e) => update({ content: (e.target as HTMLInputElement).value } as any)"
      />
    </div>
    <PropTextStyleGroup
      :font-size="style().fontSize ?? 11"
      :font-weight="style().fontWeight ?? 'normal'"
      :color="style().color ?? '#000000'"
      :align="style().align ?? 'left'"
      @update:font-size="(v) => updateStyle('fontSize', v)"
      @update:font-weight="(v) => updateStyle('fontWeight', v)"
      @update:color="(v) => updateStyle('color', v)"
      @update:align="(v) => updateStyle('align', v)"
    />
  </PropSection>
</template>
