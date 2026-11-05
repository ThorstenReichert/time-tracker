import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import type { EntryDto } from "../bindings";
import { CommandError } from "../lib/api";
import { fakeBackend, uiError } from "../test/backend";
import { acmeWebReview, betaOps, finishedEntry, runningEntry } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { clickButton, dialog, dialogOpen, fakeSearchDelay, pickSuggestion, typeAhead } from "../test/page";
import EntryDialog, { type EntryFields } from "./EntryDialog.vue";

type Save = (fields: EntryFields) => Promise<void>;

async function open(save: Save, entry: EntryDto | null = null) {
  fakeBackend({
    quick_switch: () => [acmeWebReview, betaOps],
    task_matches: () => ({ tasks: [], offerCreate: null }),
  });
  const wrapper = mountInApp(EntryDialog, { props: { visible: false, header: "Entry", entry, day: "2026-10-17", save } });
  await wrapper.setProps({ visible: true });
  await flushPromises();
  return wrapper;
}

const field = (label: string) => dialog().get(`[aria-label="${label}"]`);
const value = (label: string) => (field(label).element as HTMLInputElement).value;
const comboInputs = () => dialog().findAll(".combo-picker input");

describe("EntryDialog", () => {
  beforeEach(fakeSearchDelay);

  it("adds an entry after the fact on the given day (R10, R13)", async () => {
    const save = vi.fn<Save>(async () => {});
    const wrapper = await open(save);
    expect(value("Start date")).toBe("2026-10-17");
    expect(value("End date")).toBe("2026-10-17");

    await typeAhead(comboInputs()[0]!, "rev");
    await pickSuggestion("Acme / Web / Review");
    await field("Start time").setValue("9");
    await field("End time").setValue("10:22");
    await dialog().get("textarea").setValue("Kickoff");
    await clickButton("Save", dialog());

    expect(save).toHaveBeenCalledWith({
      project: 1,
      task: { kind: "existing", value: 1 },
      start: "2026-10-17T09:00:00",
      end: "2026-10-17T10:22:00",
      note: "Kickoff",
    });
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });

  it("starts an edit with the entry's fields and keeps its seconds", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save, finishedEntry(7, acmeWebReview, "2026-10-16T22:00:13", "2026-10-17T01:30:00", "Late"));
    expect(comboInputs().map((i) => (i.element as HTMLInputElement).value)).toEqual(["Acme / Web", "Review"]);
    expect([value("Start date"), value("Start time"), value("End date"), value("End time")]).toEqual([
      "2026-10-16",
      "22:00:13",
      "2026-10-17",
      "01:30",
    ]);
    expect((dialog().get("textarea").element as HTMLTextAreaElement).value).toBe("Late");

    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith({
      project: 1,
      task: { kind: "existing", value: 1 },
      start: "2026-10-16T22:00:13",
      end: "2026-10-17T01:30:00",
      note: "Late",
    });
  });

  it("edits an entry on an archived project that the picker no longer offers (R2b, AC-8)", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save, finishedEntry(8, acmeWebReview, "2026-10-16T09:00:00", "2026-10-16T10:00:00"));
    // The archived combo isn't a suggestion any more (the tt-core quick-switch tests pin
    // that), but the entry still shows it and saves with it.
    fakeBackend({ quick_switch: () => [betaOps], task_matches: () => ({ tasks: [], offerCreate: null }) });
    expect(comboInputs().map((i) => (i.element as HTMLInputElement).value)).toEqual(["Acme / Web", "Review"]);
    await field("End time").setValue("10:30");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith({
      project: 1,
      task: { kind: "existing", value: 1 },
      start: "2026-10-16T09:00:00",
      end: "2026-10-16T10:30:00",
      note: "",
    });
  });

  it("keeps the running entry running with an empty end, and stops it with one (R11)", async () => {
    const save = vi.fn<Save>(async () => {});
    const wrapper = await open(save, runningEntry(betaOps, 600));
    expect(field("End time").attributes("placeholder")).toBe("running");
    expect(dialog().text()).toContain("Leave the end empty to keep the timer running");

    await field("Start time").setValue("8:30");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ start: "2026-10-18T08:30:00", end: null }));

    await wrapper.setProps({ visible: false });
    await wrapper.setProps({ visible: true });
    await field("End time").setValue("11:20");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenLastCalledWith(expect.objectContaining({ end: "2026-10-18T11:20:00" }));
  });

  it("asks for a combo and readable times before saving", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save);
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain("Pick a client / project.");

    await typeAhead(comboInputs()[0]!, "ops");
    await pickSuggestion("Beta / Ops");
    await field("Start time").setValue("nine");
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain("Type the start time as HH:MM.");

    await field("Start time").setValue("9:00");
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain("Type the end time as HH:MM.");
    expect(save).not.toHaveBeenCalled();
  });

  it("shows a rejected save and stays open (R12)", async () => {
    const wrapper = await open(async () => {
      throw new CommandError(uiError("Overlaps Acme / Web 09:00–10:22"));
    }, finishedEntry(7, betaOps, "2026-10-17T10:00:00", "2026-10-17T11:00:00"));
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain("Overlaps Acme / Web 09:00–10:22");
    expect(wrapper.emitted("update:visible")).toBeUndefined();
    expect(dialogOpen()).toBe(true);
  });
});
