import { createApp } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";
import ConfirmationService from "primevue/confirmationservice";
import ToastService from "primevue/toastservice";
import { listen } from "@tauri-apps/api/event";
import App from "./App.vue";
import { ThorSoftPreset } from "./lib/brand";
import { router } from "./router";
import { useCatalogStore } from "./stores/catalog";
import { useTimerStore } from "./stores/timer";
import "./style.css";

const app = createApp(App)
  .use(createPinia())
  .use(router)
  // The ThorSoft theme, following Windows' light or dark mode (R25).
  .use(PrimeVue, { theme: { preset: ThorSoftPreset, options: { darkModeSelector: "system" } } })
  .use(ToastService)
  .use(ConfirmationService);

// The backend is the single source of truth: reload whenever it reports a change.
const catalog = useCatalogStore();
const timer = useTimerStore();
async function reload() {
  await catalog.refresh();
  await timer.refresh().catch(() => {});
}
void listen("data-changed", reload);
void reload();

app.mount("#app");
