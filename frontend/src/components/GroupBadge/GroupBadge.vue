<template>
  <span class="group-badge" :class="`group-badge--${size}`">
    <span class="group-badge__dot" :style="{ backgroundColor: color }" aria-hidden="true" />
    <span class="group-badge__label">{{ name }}</span>
  </span>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    name: string;
    color: string;
    size?: "sm" | "md";
  }>(),
  { size: "md" },
);
</script>

<style scoped lang="scss" src="./GroupBadge.scss"></style>
