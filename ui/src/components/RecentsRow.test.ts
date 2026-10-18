import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import type { ComboDto } from "../bindings";
import { useTimerStore } from "../stores/timer";
import { fakeBackend, uiError, type Handler } from "../test/backend";
import { acmeWeb, acmeWebReview, betaOps, runningEntry } from "../test/fixtures";
import { mountInApp, withToast } from "../test/mount";
import RecentsRow from "./RecentsRow.vue";

async function recentsRow(recents: ComboDto[], running: ComboDto | null, start: Handler = () => null) {
  const backend = fakeBackend({ start });
  const wrapper = mountInApp(withToast(RecentsRow));
  const timer = useTimerStore();
  timer.recents = recents;
  timer.running = running === null ? null : runningEntry(running, 60);
  await flushPromises();
  const button = (label: string) => wrapper.findAll("button").find((b) => b.text() === label)!;
  return { backend, wrapper, button };
}

describe("RecentsRow", () => {
  it("explains the empty row", async () => {
    const { wrapper } = await recentsRow([], null);
    expect(wrapper.text()).toContain("Combos you track appear here");
  });

  it("lists the recents and marks the running one (R7)", async () => {
    const { wrapper, button } = await recentsRow([acmeWebReview, betaOps], acmeWebReview);
    expect(wrapper.findAll("button").map((b) => b.text())).toEqual(["Acme / Web / Review", "Beta / Ops"]);
    expect(button("Acme / Web / Review").attributes("title")).toBe("Running");
    expect(button("Beta / Ops").attributes("title")).toBe("Start Beta / Ops");
  });

  it("starts or switches with one click, with or without a task (R5, R7)", async () => {
    const { backend, button } = await recentsRow([acmeWebReview, acmeWeb], betaOps);
    await button("Acme / Web / Review").trigger("click");
    await button("Acme / Web").trigger("click");
    await flushPromises();
    expect(backend.called("start")).toEqual([
      { project: 1, task: { kind: "existing", value: 1 } },
      { project: 1, task: { kind: "none" } },
    ]);
  });

  it("shows a rejected start as a toast", async () => {
    const { button } = await recentsRow([betaOps], null, () => {
      throw uiError("Overlaps Acme / Web 09:00–10:22");
    });
    await button("Beta / Ops").trigger("click");
    await flushPromises();
    expect(document.body.textContent).toContain("Overlaps Acme / Web 09:00–10:22");
  });
});
