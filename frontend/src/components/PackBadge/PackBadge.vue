<template>
  <component
    :is="interactive ? 'button' : 'span'"
    class="pack-badge"
    :class="{
      'pack-badge--interactive': interactive,
      'pack-badge--selected': selected,
      'pack-badge--partial': partial,
      'pack-badge--compact': compact,
      'pack-badge--muted': muted,
    }"
    :type="interactive ? 'button' : undefined"
    :aria-pressed="interactive ? pressedState : undefined"
    :aria-label="ariaLabel"
    :disabled="interactive ? disabled : undefined"
    :title="title"
    :style="{ '--pack-color': color }"
    @click="handleClick">
    <span class="pack-badge__dot" aria-hidden="true" />
    <span class="pack-badge__label">{{ name }}</span>
    <span v-if="selected || partial" class="pack-badge__state" aria-hidden="true">
      <svg v-if="selected" viewBox="0 0 12 12"><path d="m2.2 6.2 2.3 2.3 5.3-5.3" /></svg>
      <svg v-else viewBox="0 0 12 12"><path d="M2.5 6h7" /></svg>
    </span>
    <slot name="trailing" />
  </component>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    name: string;
    color: string;
    interactive?: boolean;
    selected?: boolean;
    partial?: boolean;
    compact?: boolean;
    muted?: boolean;
    disabled?: boolean;
    ariaLabel?: string;
    title?: string;
  }>(),
  {
    interactive: false,
    selected: false,
    partial: false,
    compact: false,
    muted: false,
    disabled: false,
    ariaLabel: undefined,
    title: undefined,
  },
);

const emit = defineEmits<{ click: [event: MouseEvent] }>();
const pressedState = computed(() => (props.partial ? "mixed" : String(props.selected)));

function handleClick(event: MouseEvent) {
  if (props.interactive && !props.disabled) emit("click", event);
}
</script>

<style scoped lang="scss" src="./PackBadge.scss"></style>
