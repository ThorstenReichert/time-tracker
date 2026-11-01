<script setup lang="ts">
// The ThorSoft logo (R25), used by both windows. Light mode shows the original artwork; dark mode
// a lighter variant (thorsoft-logo-dark.svg, the same drawing with the colours in
// lib/brand.ts darkLogoColours), chosen by the same media query as the theme.
import logoUrl from "../assets/thorsoft-logo.svg";
import darkLogoUrl from "../assets/thorsoft-logo-dark.svg";

withDefaults(defineProps<{ size?: number }>(), { size: 24 });
</script>

<template>
  <picture class="brand-logo">
    <source :srcset="darkLogoUrl" media="(prefers-color-scheme: dark)" />
    <img :src="logoUrl" alt="ThorSoft" :width="size" :height="size" />
  </picture>
</template>

<style scoped>
.brand-logo {
  display: block;
  flex: none;
}
.brand-logo img {
  display: block;
}
</style>
