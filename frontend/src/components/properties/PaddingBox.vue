<script setup lang="ts">
const props = defineProps<{
  top: number
  right: number
  bottom: number
  left: number
}>()

const emit = defineEmits<{
  update: [side: 'top' | 'right' | 'bottom' | 'left', value: number]
}>()

function onInput(side: 'top' | 'right' | 'bottom' | 'left', e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value) || 0
  emit('update', side, val)
}
</script>

<template>
  <div class="pb">
    <span class="pb__label">Padding</span>
    <div class="pb__box">
      <input class="pb__in pb__in--t" type="number" step="1" min="0" :value="props.top" @input="(e) => onInput('top', e)" />
      <input class="pb__in pb__in--r" type="number" step="1" min="0" :value="props.right" @input="(e) => onInput('right', e)" />
      <input class="pb__in pb__in--b" type="number" step="1" min="0" :value="props.bottom" @input="(e) => onInput('bottom', e)" />
      <input class="pb__in pb__in--l" type="number" step="1" min="0" :value="props.left" @input="(e) => onInput('left', e)" />
      <div class="pb__center" />
    </div>
  </div>
</template>

<style scoped>
.pb {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.pb__label {
  font-size: 12px;
  color: #475569;
  flex-shrink: 0;
}

.pb__box {
  position: relative;
  width: 80px;
  flex-shrink: 0;
  height: 80px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  background: #f8fafc;
}

.pb__center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 20px;
  height: 20px;
  border: 1px dashed #cbd5e1;
  border-radius: 2px;
}

.pb__in {
  position: absolute;
  width: 28px;
  height: 16px;
  border: none;
  border-radius: 2px;
  background: transparent;
  text-align: center;
  font-size: 10px;
  color: #64748b;
  padding: 0;
  outline: none;
  font-family: inherit;
  -moz-appearance: textfield;
}

.pb__in::-webkit-inner-spin-button,
.pb__in::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.pb__in:hover { background: #f1f5f9; }
.pb__in:focus { background: white; box-shadow: 0 0 0 1px #93c5fd; }

.pb__in--t { top: 1px; left: 50%; transform: translateX(-50%); }
.pb__in--b { bottom: 1px; left: 50%; transform: translateX(-50%); }
.pb__in--l { left: 2px; top: 50%; transform: translateY(-50%); }
.pb__in--r { right: 2px; top: 50%; transform: translateY(-50%); }
</style>
