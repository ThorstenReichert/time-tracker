<script setup lang="ts">
// Broken mode, startup warnings (e.g. a failed backup) and a failed reload (R22, AC-1).
import { storeToRefs } from "pinia";
import Message from "primevue/message";
import { useCatalogStore } from "../stores/catalog";

const { status, loadError } = storeToRefs(useCatalogStore());
</script>

<template>
  <div v-if="status?.broken || status?.warnings.length || loadError" class="banners">
    <Message v-if="status?.broken" severity="error">
      The app accepts no changes: {{ status.broken }}
      <div class="muted">Database: {{ status.dbPath }}</div>
    </Message>
    <Message v-for="warning in status?.warnings ?? []" :key="warning" severity="warn">{{ warning }}</Message>
    <Message v-if="loadError && !status?.broken" severity="error">Could not load data: {{ loadError }}</Message>
  </div>
</template>

<style scoped>
.banners {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem 1rem 0;
}
</style>
