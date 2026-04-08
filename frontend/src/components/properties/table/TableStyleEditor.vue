<script setup lang="ts">
import type { TableStyle } from '../../../core/types'
import '../../../styles/properties.css'

const props = defineProps<{
  style: TableStyle
  repeatHeader: boolean
}>()

const emit = defineEmits<{
  'update:style': [key: string, value: unknown]
  'update:repeatHeader': [value: boolean]
}>()
</script>

<template>
  <div class="ts-form">
    <!-- Font sizes -->
    <label class="ts-lbl" data-tip="Icerik ve header yazi boyutu (pt)">Yazi boyutu</label>
    <div class="ts-val ts-val--pair">
      <span class="ts-sep">Icerik</span>
      <span class="ts-tip-wrap" data-tip="Icerik yazi boyutu (pt)">
        <input
          class="ts-num"
          type="number"
          step="1"
          min="6"
          max="99"
          :value="style.fontSize ?? 10"
          @input="(e) => emit('update:style', 'fontSize', parseFloat((e.target as HTMLInputElement).value) || 10)"
        />
      </span>
      <span class="ts-sep">Header</span>
      <span class="ts-tip-wrap" data-tip="Header yazi boyutu (pt)">
        <input
          class="ts-num"
          type="number"
          step="1"
          min="6"
          max="99"
          :value="style.headerFontSize ?? style.fontSize ?? 10"
          @input="(e) => emit('update:style', 'headerFontSize', parseFloat((e.target as HTMLInputElement).value) || 10)"
        />
      </span>
    </div>

    <!-- Colors -->
    <label class="ts-lbl" data-tip="Header, metin ve zebra satirlari renkleri">Renkler</label>
    <div class="ts-val ts-val--colors">
      <div class="ts-color-item" data-tip="Header arkaplan rengi">
        <input
          class="ts-swatch"
          type="color"
          :value="style.headerBg ?? '#f0f0f0'"
          @input="(e) => emit('update:style', 'headerBg', (e.target as HTMLInputElement).value)"
        />
        <span class="ts-clbl">Arkaplan</span>
      </div>
      <div class="ts-color-item" data-tip="Header metin rengi">
        <input
          class="ts-swatch"
          type="color"
          :value="style.headerColor ?? '#000000'"
          @input="(e) => emit('update:style', 'headerColor', (e.target as HTMLInputElement).value)"
        />
        <span class="ts-clbl">Metin</span>
      </div>
      <div class="ts-color-item" data-tip="Zebra satir rengi — tek satirlar">
        <div class="ts-swatch-wrap">
          <input
            class="ts-swatch"
            type="color"
            :value="style.zebraOdd ?? '#fafafa'"
            @input="(e) => emit('update:style', 'zebraOdd', (e.target as HTMLInputElement).value)"
          />
          <button
            v-if="style.zebraOdd"
            class="ts-swatch-clr"
            @click="emit('update:style', 'zebraOdd', undefined)"
          >
            &times;
          </button>
        </div>
        <span class="ts-clbl">Tek</span>
      </div>
      <div class="ts-color-item" data-tip="Zebra satir rengi — cift satirlar">
        <div class="ts-swatch-wrap">
          <input
            class="ts-swatch"
            type="color"
            :value="style.zebraEven ?? '#ffffff'"
            @input="(e) => emit('update:style', 'zebraEven', (e.target as HTMLInputElement).value)"
          />
          <button
            v-if="style.zebraEven"
            class="ts-swatch-clr"
            @click="emit('update:style', 'zebraEven', undefined)"
          >
            &times;
          </button>
        </div>
        <span class="ts-clbl">Cift</span>
      </div>
    </div>

    <!-- Border -->
    <label class="ts-lbl" data-tip="Tablo kenarlik rengi ve kalinligi">Kenarlik</label>
    <div class="ts-val ts-val--pair">
      <div class="ts-swatch-wrap" data-tip="Kenarlik rengi">
        <input
          class="ts-swatch"
          type="color"
          :value="style.borderColor ?? '#cccccc'"
          @input="(e) => emit('update:style', 'borderColor', (e.target as HTMLInputElement).value)"
        />
        <button
          v-if="style.borderColor"
          class="ts-swatch-clr"
          @click="emit('update:style', 'borderColor', undefined)"
        >
          &times;
        </button>
      </div>
      <span class="ts-tip-wrap" data-tip="Kenarlik kalinligi (mm)">
        <input
          class="ts-num"
          type="number"
          step="0.1"
          min="0"
          max="99"
          :value="style.borderWidth ?? 0.5"
          @input="(e) => emit('update:style', 'borderWidth', parseFloat((e.target as HTMLInputElement).value) || 0)"
        />
      </span>
      <span class="ts-unit">mm</span>
    </div>

    <!-- Cell padding -->
    <label class="ts-lbl" data-tip="Hucre ic bosluklari — yatay ve dikey (mm)">Ic bosluk</label>
    <div class="ts-val ts-val--pair">
      <span class="ts-pad-icon" data-tip="Yatay bosluk (mm)">&#8596;</span>
      <span class="ts-tip-wrap" data-tip="Yatay ic bosluk (mm)">
        <input
          class="ts-num"
          type="number"
          step="0.5"
          min="0"
          max="99"
          :value="style.cellPaddingH ?? 2"
          @input="(e) => emit('update:style', 'cellPaddingH', parseFloat((e.target as HTMLInputElement).value) || 0)"
        />
      </span>
      <span class="ts-pad-icon" data-tip="Dikey bosluk (mm)">&#8597;</span>
      <span class="ts-tip-wrap" data-tip="Dikey ic bosluk (mm)">
        <input
          class="ts-num"
          type="number"
          step="0.5"
          min="0"
          max="99"
          :value="style.cellPaddingV ?? 1"
          @input="(e) => emit('update:style', 'cellPaddingV', parseFloat((e.target as HTMLInputElement).value) || 0)"
        />
      </span>
    </div>

    <!-- Header padding -->
    <label class="ts-lbl" data-tip="Header hucre bosluklari — yatay ve dikey (mm)">Header bosluk</label>
    <div class="ts-val ts-val--pair">
      <span class="ts-pad-icon" data-tip="Yatay bosluk (mm)">&#8596;</span>
      <span class="ts-tip-wrap" data-tip="Header yatay bosluk (mm)">
        <input
          class="ts-num"
          type="number"
          step="0.5"
          min="0"
          max="99"
          :value="style.headerPaddingH ?? style.cellPaddingH ?? 2"
          @input="(e) => emit('update:style', 'headerPaddingH', parseFloat((e.target as HTMLInputElement).value) || 0)"
        />
      </span>
      <span class="ts-pad-icon" data-tip="Dikey bosluk (mm)">&#8597;</span>
      <span class="ts-tip-wrap" data-tip="Header dikey bosluk (mm)">
        <input
          class="ts-num"
          type="number"
          step="0.5"
          min="0"
          max="99"
          :value="style.headerPaddingV ?? style.cellPaddingV ?? 1"
          @input="(e) => emit('update:style', 'headerPaddingV', parseFloat((e.target as HTMLInputElement).value) || 0)"
        />
      </span>
    </div>

    <!-- Repeat header -->
    <label class="ts-lbl" data-tip="Cok sayfali tablolarda header'i her sayfada tekrarla">Header tekrarla</label>
    <div class="ts-val">
      <label class="ts-toggle">
        <input
          type="checkbox"
          :checked="repeatHeader"
          @change="(e) => emit('update:repeatHeader', (e.target as HTMLInputElement).checked)"
        />
        <span class="ts-toggle__track"></span>
      </label>
    </div>
  </div>
</template>

<style scoped>
.ts-form {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 5px 8px;
  align-items: center;
}

.ts-lbl {
  font-size: 11px;
  color: #64748b;
  white-space: nowrap;
}

.ts-val {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.ts-val--pair {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
}

.ts-val--colors {
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  gap: 6px;
}

.ts-sep {
  font-size: 10px;
  color: #94a3b8;
}

.ts-num {
  width: 32px;
  padding: 2px 3px;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
  font-size: 11px;
  background: white;
  color: #334155;
  text-align: center;
  -moz-appearance: textfield;
}

.ts-num::-webkit-inner-spin-button,
.ts-num::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.ts-num:focus {
  outline: none;
  border-color: #93c5fd;
}

.ts-unit {
  font-size: 10px;
  color: #94a3b8;
}

.ts-color-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.ts-clbl {
  font-size: 9px;
  color: #94a3b8;
  white-space: nowrap;
}

.ts-swatch {
  width: 22px;
  height: 22px;
  padding: 0;
  cursor: pointer;
  border: 1px solid #e2e8f0;
  border-radius: 3px;
}

.ts-swatch-wrap {
  position: relative;
  display: inline-flex;
}

.ts-swatch-clr {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #f1f5f9;
  border: 1px solid #e2e8f0;
  font-size: 9px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: #94a3b8;
  padding: 0;
}

.ts-swatch-clr:hover {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}

.ts-pad-icon {
  font-size: 11px;
  color: #94a3b8;
  line-height: 1;
}

.ts-tip-wrap {
  position: relative;
  display: inline-flex;
}

.ts-toggle {
  position: relative;
  display: inline-block;
  cursor: pointer;
}

.ts-toggle input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.ts-toggle__track {
  display: block;
  width: 28px;
  height: 16px;
  background: #e2e8f0;
  border-radius: 8px;
  transition: background 0.15s;
  position: relative;
}

.ts-toggle__track::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  background: white;
  border-radius: 50%;
  transition: transform 0.15s;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.ts-toggle input:checked + .ts-toggle__track {
  background: #3b82f6;
}

.ts-toggle input:checked + .ts-toggle__track::after {
  transform: translateX(12px);
}
</style>
