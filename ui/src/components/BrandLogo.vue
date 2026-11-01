<script setup lang="ts">
// The ThorSoft logo (R25), used by both windows. The SVG is the original artwork.
import logoUrl from "../assets/thorsoft-logo.svg";

withDefaults(defineProps<{ size?: number }>(), { size: 24 });
</script>

<template>
  <img :src="logoUrl" alt="ThorSoft" :width="size" :height="size" class="brand-logo" />
</template>

<style scoped>
.brand-logo {
  display: block;
  flex: none;
}
</style>
