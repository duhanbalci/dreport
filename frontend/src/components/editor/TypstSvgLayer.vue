<script setup lang="ts">
defineProps<{
  svg: string | null
}>()
</script>

<template>
  <div class="typst-svg-layer" v-if="svg" v-html="svg" />
  <div class="typst-svg-layer typst-svg-layer--empty" v-else>
    <span>Derleniyor...</span>
  </div>
</template>

<style scoped>
.typst-svg-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  user-select: none;
}

.typst-svg-layer :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}

.typst-svg-layer--empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 14px;
}
</style>
