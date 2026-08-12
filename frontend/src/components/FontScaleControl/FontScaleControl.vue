<template>
  <div class="font-scale-control" :class="`font-scale-control--${variant}`">
    <template v-if="variant === 'compact'">
      <button
        type="button"
        class="font-scale-control__step"
        aria-label="Decrease font size"
        :disabled="presetIndex <= 0"
        @click="step(-1)"
      >
        A−
      </button>
      <span class="font-scale-control__value">{{ Math.round(modelValue * 100) }}%</span>
      <button
        type="button"
        class="font-scale-control__step"
        aria-label="Increase font size"
        :disabled="presetIndex >= presets.length - 1"
        @click="step(1)"
      >
        A+
      </button>
    </template>
    <template v-else>
      <div class="font-scale-control__presets" role="radiogroup" aria-label="Interface size">
        <button
          v-for="preset in presets"
          :key="preset.value"
          type="button"
          class="font-scale-control__preset"
          :class="{ 'is-active': preset.value === modelValue }"
          role="radio"
          :aria-checked="preset.value === modelValue"
          @click="emit('update:modelValue', preset.value)"
        >
          {{ preset.label }}
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { FONT_SCALE_PRESETS } from '../../types'

const props = withDefaults(
  defineProps<{
    modelValue: number
    variant?: 'compact' | 'full'
  }>(),
  { variant: 'compact' },
)

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const presets = FONT_SCALE_PRESETS

const presetIndex = computed(() => presets.findIndex((p) => p.value === props.modelValue))

function step(direction: 1 | -1) {
  const currentIndex = presetIndex.value === -1 ? 1 : presetIndex.value
  const nextIndex = Math.min(presets.length - 1, Math.max(0, currentIndex + direction))
  emit('update:modelValue', presets[nextIndex].value)
}
</script>

<style scoped lang="scss" src="./FontScaleControl.scss"></style>
