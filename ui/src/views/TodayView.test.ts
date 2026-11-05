import { beforeEach, describe, expect, it, vi } from "vitest";
import { DOMWrapper, flushPromises, type VueWrapper } from "@vue/test-utils";
import { emit } from "@tauri-apps/api/event";
import type { EntryDto } from "../bindings";
import { useTimerStore } from "../stores/timer";
import { fakeBackend, uiError, type Handler } from "../test/backend";
import { acmeWeb, acmeWebReview, betaApi, betaOps, finishedEntry, runningEntry } from "../test/fixtures";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton, dialog, dialogOpen, fakeSearchDelay, pickSuggestion, pressEnter, typeAhead, typeKeys } from "../test/page";
import TodayView from "./TodayView.vue";

const TODAY = "2026-10-18";

// The worked day on the 17th, newest first as the backend lists it (R10), with a note; on the
// 18th, Beta / Ops is running.
const workedDay: EntryDto[] = [
  finishedEntry(3, betaOps, "2026-10-17T11:07:00", "2026-10-17T11:20:00"),
  finishedEntry(2, betaApi, "2026-10-17T10:22:00", "2026-10-17T11:07:00"),
  finishedEntry(1, acmeWeb, "2026-10-17T09:00:00", "2026-10-17T10:22:00", "Kickoff\nwith the team"),
];
const running = { ...runningEntry(betaOps, 600), id: 4 };

function backend(handlers: Record<string, Handler> = {}) {
  return fakeBackend({
    get_today: () => TODAY,
    get_day_entries: ({ day }: { day: string }) => (day === TODAY ? [running] : day === "2026-10-17" ? workedDay : []),
    quick_switch: () => [acmeWebReview, betaOps],
    task_matches: () => ({ tasks: [], offerCreate: null }),
    ...handlers,
  });
}

async function mountToday() {
  const wrapper = mountInApp(withOverlays(TodayView));
  await flushPromises();
  return wrapper;
}

/** Each entry row's cells: times, combo and note, duration. */
function rows(wrapper: VueWrapper): string[][] {
  return wrapper.findAll("[data-entry]").map((row) =>
    row
      .findAll("td")
      .slice(0, 3)
      .map((c) => c.text()),
  );
}

const confirmation = () => document.querySelector("[role=alertdialog]");

describe("TodayView", () => {
  beforeEach(fakeSearchDelay);

  it("starts the picked combo and clears the picker (R5)", async () => {
    const fake = backend({ start: () => null });
    const wrapper = await mountToday();
    useTimerStore().recents = [betaOps];
    const startButton = () => wrapper.findAll("button").find((b) => b.text() === "Start")!;
    expect(startButton().attributes("disabled")).toBeDefined();

    await typeAhead(wrapper.get(".start input"), "rev");
    await pickSuggestion("Acme / Web / Review");
    expect(startButton().attributes("disabled")).toBeUndefined();
    await clickButton("Start");

    expect(fake.called("start")).toEqual([{ project: 1, task: { kind: "existing", value: 1 } }]);
    expect((wrapper.get(".start input").element as HTMLInputElement).value).toBe("");
    expect(startButton().attributes("disabled")).toBeDefined();
  });

  it("starts with a task typed and Entered before its search answered (R3)", async () => {
    const fake = backend({
      start: () => null,
      quick_switch: () => [acmeWeb],
      task_matches: ({ text }: { text: string }) => ({ tasks: [], offerCreate: text === "" ? null : text }),
    });
    const wrapper = await mountToday();
    await typeAhead(wrapper.get(".start input"), "web");
    await pickSuggestion("Acme / Web");

    // Enter within AutoComplete's 300 ms search delay.
    const taskInput = wrapper.findAll(".start input")[1]!;
    await typeKeys(taskInput, "Docs");
    await pressEnter(taskInput);
    expect(fake.called("start")).toEqual([{ project: 1, task: { kind: "new", value: "Docs" } }]);
  });

  it("lists a day's entries with their notes (R10, R13)", async () => {
    const fake = backend();
    const wrapper = await mountToday();
    expect(fake.called("get_day_entries")).toEqual([{ day: TODAY }]);
    expect(rows(wrapper)).toEqual([["09:00–now running", "Beta / Ops", "0:10"]]);

    await clickButton("Previous day");
    expect(rows(wrapper)).toEqual([
      ["11:07–11:20", "Beta / Ops", "0:13"],
      ["10:22–11:07", "Beta / API", "0:45"],
      ["09:00–10:22", "Acme / WebKickoff\nwith the team", "1:22"],
    ]);

    await clickButton("Previous day");
    expect(wrapper.text()).toContain("No entries on this day.");
  });

  it("counts the running entry up with the timer (R10)", async () => {
    vi.useFakeTimers({ toFake: ["Date", "setInterval", "setTimeout", "clearTimeout"] });
    vi.setSystemTime(new Date("2026-10-18T10:00:00Z"));
    backend();
    const wrapper = await mountToday();
    useTimerStore().running = { ...running, start: Date.now() / 1000 - 600 };
    await vi.advanceTimersByTimeAsync(60_000);
    expect(rows(wrapper)[0]![2]).toBe("0:11");
  });

  it("adds an entry on the shown day (R10)", async () => {
    const fake = backend({ add_entry: () => null });
    await mountToday();
    await clickButton("Previous day");
    await clickButton("Add entry");

    await typeAhead(dialog().findAll(".combo-picker input")[0]!, "rev");
    await pickSuggestion("Acme / Web / Review");
    await dialog().get('[aria-label="Start time"]').setValue("12:00");
    await dialog().get('[aria-label="End time"]').setValue("12:30");
    await clickButton("Save", dialog());

    expect(fake.called("add_entry")).toEqual([
      {
        project: 1,
        task: { kind: "existing", value: 1 },
        start: "2026-10-17T12:00:00",
        end: "2026-10-17T12:30:00",
        note: "",
      },
    ]);
    expect(dialogOpen()).toBe(false);
  });

  it("edits an entry and shows an overlap with the conflicting entry (R10, R12)", async () => {
    const fake = backend({
      edit_entry: ({ end }: { end: string }) => {
        if (end > "2026-10-17T10:22:00") throw uiError("Overlaps Beta / API 10:22–11:07");
        return null;
      },
    });
    const wrapper = await mountToday();
    await clickButton("Previous day");
    await clickButton("Edit", wrapper.get('[data-entry="1"]'));
    expect(dialog().text()).toContain("Edit the entry");

    await dialog().get('[aria-label="End time"]').setValue("10:30");
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain("Overlaps Beta / API 10:22–11:07");

    await dialog().get('[aria-label="End time"]').setValue("10:15");
    await clickButton("Save", dialog());
    expect(fake.called("edit_entry")).toHaveLength(2);
    expect(fake.called("edit_entry")[1]).toEqual({
      id: 1,
      project: 1,
      task: { kind: "none" },
      start: "2026-10-17T09:00:00",
      end: "2026-10-17T10:15:00",
      note: "Kickoff\nwith the team",
    });
    expect(dialogOpen()).toBe(false);
  });

  it("asks before it deletes an entry (R10)", async () => {
    const fake = backend({ delete_entry: () => null });
    const wrapper = await mountToday();
    await clickButton("Previous day");

    await clickButton("Delete", wrapper.get('[data-entry="2"]'));
    expect(confirmation()?.textContent).toContain("Beta / API, 10:22–11:07");
    await clickButton("Cancel", new DOMWrapper(confirmation()!));
    expect(confirmation()).toBeNull();
    expect(fake.called("delete_entry")).toEqual([]);

    await clickButton("Delete", wrapper.get('[data-entry="2"]'));
    await clickButton("Delete", new DOMWrapper(confirmation()!));
    expect(fake.called("delete_entry")).toEqual([{ id: 2 }]);
  });

  it("moves on to the new today at midnight while the window stays open (R10)", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "setInterval", "clearInterval"] });
    let today = TODAY;
    const fake = backend({ get_today: () => today });
    await mountToday();

    today = "2026-10-19";
    await vi.advanceTimersByTimeAsync(60_000);
    await flushPromises();
    expect(fake.called("get_day_entries").map((a) => a.day)).toEqual([TODAY, "2026-10-19"]);
  });

  it("shows no entries of the previous day when a day fails to load", async () => {
    backend({
      get_day_entries: ({ day }: { day: string }) => {
        if (day === "2026-10-17") throw uiError("the database is busy", "store");
        return [running];
      },
    });
    const wrapper = await mountToday();
    expect(rows(wrapper)).toHaveLength(1);

    await clickButton("Previous day");
    expect(wrapper.text()).toContain("the database is busy");
    expect(rows(wrapper)).toEqual([]);
  });

  it("reloads when the data changes", async () => {
    const fake = backend();
    await mountToday();
    await emit("data-changed");
    await flushPromises();
    expect(fake.called("get_day_entries")).toEqual([{ day: TODAY }, { day: TODAY }]);
  });
});
