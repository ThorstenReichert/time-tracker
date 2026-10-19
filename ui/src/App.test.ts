import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { emit } from "@tauri-apps/api/event";
import App from "./App.vue";
import { fakeBackend } from "./test/backend";
import { mountInApp } from "./test/mount";

describe("App", () => {
  it("shows a failed tray action as a toast", async () => {
    fakeBackend({});
    mountInApp(App, { global: { stubs: { RouterLink: true, RouterView: true } } });
    await flushPromises();
    await emit("action-failed", "Overlaps Acme / Web 09:00–10:22");
    await flushPromises();
    expect(document.body.textContent).toContain("Nothing changed");
    expect(document.body.textContent).toContain("Overlaps Acme / Web 09:00–10:22");
  });
});
