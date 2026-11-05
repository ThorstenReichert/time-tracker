<script setup lang="ts">
// The main window's shell: the brand mark (R25), navigation, the running timer, the status banner and the views.
import { listen } from "@tauri-apps/api/event";
import ConfirmDialog from "primevue/confirmdialog";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";
import BrandMark from "./components/BrandMark.vue";
import RunningBar from "./components/RunningBar.vue";
import StatusBanner from "./components/StatusBanner.vue";

// A tray menu action (start, stop) failed; the backend has shown this window for it.
const toast = useToast();
void listen<string>("action-failed", (event) =>
  toast.add({ severity: "error", summary: "Nothing changed", detail: event.payload, life: 8000 }),
);

const nav = [
  { to: "/", label: "Today" },
  { to: "/overview", label: "Daily overview" },
  { to: "/clients", label: "Clients & projects" },
  { to: "/archive", label: "Archive" },
  { to: "/settings", label: "Settings" },
];
</script>

<template>
  <div class="shell">
    <nav class="nav">
      <BrandMark />
      <RouterLink v-for="item in nav" :key="item.to" :to="item.to" class="nav-link">{{ item.label }}</RouterLink>
    </nav>
    <RunningBar />
    <StatusBanner />
    <main class="content">
      <RouterView />
    </main>
    <Toast position="bottom-right" />
    <ConfirmDialog />
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.nav {
  display: flex;
  gap: 0.25rem;
  padding: 0 0.5rem;
  border-bottom: 1px solid var(--p-content-border-color);
}
.nav-link {
  padding: 0.75rem 0.9rem;
  color: var(--p-text-muted-color);
  text-decoration: none;
  border-bottom: 2px solid transparent;
}
.nav-link:hover {
  color: var(--p-text-color);
}
.nav-link.router-link-exact-active {
  color: var(--p-primary-color);
  border-bottom-color: var(--p-primary-color);
}
.content {
  flex: 1;
  overflow: auto;
}
</style>
