import { describe, expect, it } from "vitest";
import { useCatalogStore } from "../stores/catalog";
import { healthy } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import StatusBanner from "./StatusBanner.vue";

async function banner(status: typeof healthy | null, loadError: string | null = null) {
  const wrapper = mountInApp(StatusBanner);
  const store = useCatalogStore();
  store.status = status;
  store.loadError = loadError;
  await wrapper.vm.$nextTick();
  return wrapper;
}

describe("StatusBanner", () => {
  it("shows nothing while the app works", async () => {
    expect((await banner(healthy)).find(".banners").exists()).toBe(false);
  });

  it("explains Broken mode with the database path", async () => {
    const wrapper = await banner({ ...healthy, broken: "an entry overlaps another" });
    expect(wrapper.text()).toContain("The app accepts no changes: an entry overlaps another");
    expect(wrapper.text()).toContain("Database: C:\\demo\\timetracker.db");
  });

  it("shows each warning, e.g. a failed backup or an unavailable hotkey (R22, AC-1)", async () => {
    const warnings = ["The backup failed: disk full", "The hotkey Ctrl+Alt+Space is not available"];
    const wrapper = await banner({ ...healthy, warnings });
    expect(wrapper.findAll(".p-message").map((m) => m.text())).toEqual(warnings);
  });

  it("shows a failed reload, unless the app is broken anyway", async () => {
    expect((await banner(healthy, "IPC down")).text()).toBe("Could not load data: IPC down");
    const broken = await banner({ ...healthy, broken: "locked" }, "IPC down");
    expect(broken.text()).not.toContain("Could not load data");
  });
});
