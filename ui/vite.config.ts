import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

// Two pages: the main window and the quick-switch popup (spec §2 "Windows").
// Tauri expects a fixed dev port (tauri.conf.json build.devUrl).
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: {
    target: "es2022",
    // Vite's 500 kB default is meant for downloads. Tauri loads the assets from disk, and the
    // main chunk is mostly PrimeVue and its theme (~600 kB). The limit still flags a big jump.
    chunkSizeWarningLimit: 800,
    rolldownOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        "quick-switch": fileURLToPath(new URL("./quick-switch.html", import.meta.url)),
      },
    },
  },
  // Component tests render into happy-dom; src/test/setup.ts resets the faked backend.
  test: { environment: "happy-dom", setupFiles: ["src/test/setup.ts"] },
});
