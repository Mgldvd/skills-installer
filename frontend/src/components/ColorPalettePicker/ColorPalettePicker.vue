<template>
  <fieldset class="color-palette-picker" :aria-label="ariaLabel">
    <button
      v-for="color in palette"
      :key="color"
      type="button"
      class="color-palette-picker__swatch"
      :class="{ 'is-selected': modelValue === color }"
      :style="{ backgroundColor: color }"
      :aria-pressed="modelValue === color"
      :aria-label="`Use color ${color}`"
      @click="emit('update:modelValue', color)" />
    <label
      class="color-palette-picker__swatch color-palette-picker__custom"
      :class="{ 'is-selected': isCustom }"
      :aria-pressed="isCustom"
      title="Custom color">
      <input
        type="color"
        class="color-palette-picker__custom-input"
        :value="isCustom ? modelValue : '#000000'"
        aria-label="Pick a custom color"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value.toUpperCase())" />
    </label>
  </fieldset>
</template>

<script setup lang="ts">
import { computed } from "vue";

import { PICKER_PALETTE } from "../../utils/color";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    ariaLabel?: string;
  }>(),
  {
    ariaLabel: "Color",
  },
);

const emit = defineEmits<{
  "update:modelValue": [color: string];
}>();

const palette = PICKER_PALETTE;
const isCustom = computed(() => !palette.includes(props.modelValue));
</script>

<style scoped lang="scss" src="./ColorPalettePicker.scss"></style>
