<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(defineProps<{ title: string; defaultOpen?: boolean }>(), {
  defaultOpen: true,
})

const open = ref(props.defaultOpen)
</script>

<template>
  <div class="prop-section">
    <div class="prop-section__title prop-section__title--collapsible" @click="open = !open">
      <span class="prop-section__chevron" :class="{ 'prop-section__chevron--closed': !open }"
        >&#9662;</span
      >
      {{ title }}
      <span class="prop-section__actions" @click.stop><slot name="actions" /></span>
    </div>
    <template v-if="open"><slot /></template>
  </div>
</template>

<style scoped>
.prop-section__title--collapsible {
  cursor: pointer;
  user-select: none;
  display: flex;
  align-items: center;
  gap: 4px;
}

.prop-section__chevron {
  font-size: 8px;
  transition: transform 0.15s;
  display: inline-block;
}

.prop-section__chevron--closed {
  transform: rotate(-90deg);
}

.prop-section__actions {
  margin-left: auto;
}
</style>
