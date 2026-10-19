import { afterEach, vi } from "vitest";
import { clearMocks } from "@tauri-apps/api/mocks";
import { enableAutoUnmount } from "@vue/test-utils";

// Every test starts with no faked backend, real timers and an empty page.
afterEach(() => {
  clearMocks();
  vi.useRealTimers();
  document.body.innerHTML = "";
});

// Unmounts every test's components first (after-hooks run last-registered first). PrimeVue's
// confirmation service is a global event bus, so a ConfirmDialog left mounted by an earlier
// test would answer the next test's confirmation too.
enableAutoUnmount(afterEach);
