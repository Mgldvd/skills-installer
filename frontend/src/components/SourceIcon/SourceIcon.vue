<template>
  <span
    class="source-icon"
    :class="local ? 'source-icon--local' : 'source-icon--remote'"
    :title="decorative ? undefined : local ? 'Local Skill' : 'Remote Skill'"
    :aria-label="decorative ? undefined : local ? 'Local Skill' : 'Remote Skill'"
    :aria-hidden="decorative || undefined"
  >
    <svg v-if="local" viewBox="0 0 16 16" aria-hidden="true">
      <path d="M8 1.5 14.5 13.5H1.5Z" />
      <text x="8" y="12" text-anchor="middle">L</text>
    </svg>
    <svg v-else viewBox="0 0 16 16" aria-hidden="true">
      <rect x="2" y="2" width="12" height="12" rx="2.5" />
      <text x="8" y="11.1" text-anchor="middle">R</text>
    </svg>
  </span>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    local: boolean;
    // For use beside a control that already has its own visible label (e.g.
    // the Local/Remote filter buttons) — the icon there is decoration, not
    // a second, redundant accessible name.
    decorative?: boolean;
  }>(),
  { decorative: false },
);
</script>

<style scoped lang="scss" src="./SourceIcon.scss"></style>
