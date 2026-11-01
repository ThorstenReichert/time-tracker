<script setup lang="ts">
// The brand mark at the left of the main window's navigation bar (R25): logo and product name.
import BrandLogo from "./BrandLogo.vue";
</script>

<template>
  <div class="brand-mark" title="ThorSoft TimeTracker">
    <BrandLogo :size="24" />
    <span class="wordmark">TimeTracker</span>
  </div>
</template>

<style scoped>
.brand-mark {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0 1rem 0 0.4rem;
  user-select: none;
}
.wordmark {
  font-weight: 600;
  color: var(--p-primary-800);
}
</style>
