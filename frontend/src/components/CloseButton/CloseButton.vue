<template>
  <button type="button" class="close-button" :aria-label="ariaLabel" :title="ariaLabel" @click="emit('click', $event)">
    <svg viewBox="0 0 20 20" aria-hidden="true">
      <path d="m5 5 10 10M15 5 5 15" />
    </svg>
  </button>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ ariaLabel?: string }>(), { ariaLabel: 'Close' })
const emit = defineEmits<{ click: [event: MouseEvent] }>()
</script>

<style scoped lang="scss" src="./CloseButton.scss"></style>
