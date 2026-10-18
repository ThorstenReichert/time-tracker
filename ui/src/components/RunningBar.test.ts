import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { useTimerStore } from "../stores/timer";
import { fakeBackend, uiError } from "../test/backend";
import { acmeWebReview, runningEntry } from "../test/fixtures";
import { mountInApp, withToast } from "../test/mount";
import RunningBar from "./RunningBar.vue";

async function bar(running: boolean) {
  const wrapper = mountInApp(withToast(RunningBar));
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

  it("stops the timer now", async () => {
    const backend = fakeBackend({ stop: () => null });
    const wrapper = await bar(true);
    await wrapper.get("button").trigger("click");
    await flushPromises();
    expect(backend.called("stop")).toEqual([{ at: null }]);
  });

  it("shows a failed stop as a toast", async () => {
    fakeBackend({
      stop: () => {
        throw uiError("The app accepts no changes", "broken");
      },
    });
    const wrapper = await bar(true);
    await wrapper.get("button").trigger("click");
    await flushPromises();
    expect(document.body.textContent).toContain("Nothing changed");
    expect(document.body.textContent).toContain("The app accepts no changes");
  });
});
