import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { useTimerStore } from "../stores/timer";
import { fakeBackend, uiError } from "../test/backend";
import { acmeWebReview, combo, runningEntry } from "../test/fixtures";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import RunningBar from "./RunningBar.vue";

async function bar(running: boolean) {
  const wrapper = mountInApp(withOverlays(RunningBar));
  useTimerStore().running = running ? runningEntry(acmeWebReview, 4920) : null;
  await flushPromises();
  return wrapper;
}

describe("RunningBar", () => {
  it("says when no timer runs", async () => {
    fakeBackend({});
    const wrapper = await bar(false);
    expect(wrapper.text()).toContain("No timer running");
    expect(wrapper.find("button").exists()).toBe(false);
  });

  it("shows the running combo and counts its elapsed time in the UI (R5)", async () => {
    // Only the clock and the timer store's interval are faked; flushPromises needs real timeouts.
    vi.useFakeTimers({ toFake: ["Date", "setInterval"] });
    vi.setSystemTime(new Date("2026-10-18T10:00:00Z"));
    fakeBackend({});
    const wrapper = await bar(true);
    expect(wrapper.text()).toContain("Acme / Web / Review");
    expect(wrapper.get(".elapsed").text()).toBe("1:22:00");
    await vi.advanceTimersByTimeAsync(2000);
    expect(wrapper.get(".elapsed").text()).toBe("1:22:02");
  });

  it("shows the running combo's short label, the full one on hover (R24)", async () => {
    fakeBackend({});
    const wrapper = mountInApp(withOverlays(RunningBar));
    useTimerStore().running = runningEntry(combo(9, 9, 9, ["Northwind Instruments", "adaptive signal calibration pipeline", "Review"], ["NWI", "ASCP", "Rev"]), 60);
    await flushPromises();
    const label = wrapper.get(".label");
    expect(label.text()).toBe("NWI / ASCP / Rev");
    expect(label.attributes("title")).toBe("Northwind Instruments / adaptive signal calibration pipeline / Review");
  });

  it("stops the timer now", async () => {
    const backend = fakeBackend({ stop: () => null });
    await bar(true);
    await clickButton("Stop");
    expect(backend.called("stop")).toEqual([{ at: null }]);
  });

  it("shows a failed stop as a toast", async () => {
    fakeBackend({
      stop: () => {
        throw uiError("The app accepts no changes", "broken");
      },
    });
    await bar(true);
    await clickButton("Stop");
    expect(document.body.textContent).toContain("Nothing changed");
    expect(document.body.textContent).toContain("The app accepts no changes");
  });

  it("stops a forgotten timer at a typed time (R11)", async () => {
    const backend = fakeBackend({ stop: () => null });
    await bar(true);
    await clickButton("Stop at…");
    await dialog().get('[aria-label="Stopped at time"]').setValue("17:30");
    await clickButton("Stop", dialog());
    expect(backend.called("stop")).toEqual([{ at: "2026-10-18T17:30:00" }]);
  });

  it("edits the running entry's start and keeps it running (R11)", async () => {
    const backend = fakeBackend({ edit_entry: () => null });
    await bar(true);
    await clickButton("Edit");
    expect(dialog().text()).toContain("Edit the running entry");
    await dialog().get('[aria-label="Start time"]').setValue("8:45");
    await clickButton("Save", dialog());
    expect(backend.called("edit_entry")).toEqual([
      {
        id: 1,
        project: 1,
        task: { kind: "existing", value: 1 },
        start: "2026-10-18T08:45:00",
        end: null,
        note: "",
      },
    ]);
  });
});
