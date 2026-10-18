import { afterEach, vi } from "vitest";
import { clearMocks } from "@tauri-apps/api/mocks";

// Every test starts with no faked backend, real timers and an empty page.
afterEach(() => {
  clearMocks();
  vi.useRealTimers();
  document.body.innerHTML = "";
});
