<template>
  <div class="toast-host" role="status" aria-live="polite">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      class="toast-host__toast"
      :class="`toast-host__toast--${toast.variant}`"
    >
      {{ toast.message }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { useToasts } from '../../composables/useToasts'

const { toasts } = useToasts()
</script>

<style scoped lang="scss" src="./ToastHost.scss"></style>
