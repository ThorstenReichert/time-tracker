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
    rolldownOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        "quick-switch": fileURLToPath(new URL("./quick-switch.html", import.meta.url)),
      },
    },
  },
  test: { environment: "node" },
});
