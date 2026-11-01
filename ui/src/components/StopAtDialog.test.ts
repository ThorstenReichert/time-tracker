import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { CommandError } from "../lib/api";
import { fakeBackend, uiError } from "../test/backend";
import { acmeWebReview, combo, runningEntry } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import StopAtDialog from "./StopAtDialog.vue";

type Save = (at: string) => Promise<void>;

async function open(save: Save, running = runningEntry(acmeWebReview, 4920)) {
  fakeBackend({});
  const wrapper = mountInApp(StopAtDialog, { props: { visible: false, running, save } });
  await wrapper.setProps({ visible: true });
  await flushPromises();
  return wrapper;
}

const time = () => dialog().get('[aria-label="Stopped at time"]');

describe("StopAtDialog", () => {
  it("stops at a typed time on the start's day (R11)", async () => {
    const save = vi.fn<Save>(async () => {});
    const wrapper = await open(save);
    expect(dialog().text()).toContain("Acme / Web / Review, started 2026-10-18 09:00");
    expect((dialog().get('[aria-label="Stopped at date"]').element as HTMLInputElement).value).toBe("2026-10-18");

    await time().setValue("17:30");
    await clickButton("Stop", dialog());
    expect(save).toHaveBeenCalledWith("2026-10-18T17:30:00");
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });

  it("names the running combo by its short label, the full one on hover (R24)", async () => {
    await open(vi.fn<Save>(), runningEntry(combo(9, 9, 9, ["Northwind Instruments", "adaptive signal calibration pipeline", "Review"], ["NWI", "ASCP", "Rev"]), 60));
    expect(dialog().text()).toContain("NWI / ASCP / Rev, started");
    expect(dialog().get("span[title]").attributes("title")).toBe(
      "Northwind Instruments / adaptive signal calibration pipeline / Review",
    );
  });

  it("asks for a readable time", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save);
    await time().setValue("half five");
    await clickButton("Stop", dialog());
    expect(dialog().text()).toContain("Type the time as HH:MM.");
    expect(save).not.toHaveBeenCalled();
  });

  it("shows a rejected time and stays open (R12)", async () => {
    const wrapper = await open(async () => {
      throw new CommandError(uiError("An entry can't end in the future"));
    });
    await time().setValue("23:59");
    await clickButton("Stop", dialog());
    expect(dialog().text()).toContain("An entry can't end in the future");
    expect(wrapper.emitted("update:visible")).toBeUndefined();
  });
});
