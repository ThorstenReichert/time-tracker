import { describe, expect, it, vi } from "vitest";
import { flushPromises, type VueWrapper } from "@vue/test-utils";
import { emit } from "@tauri-apps/api/event";
import type { Separator } from "../bindings";
import { useCatalogStore } from "../stores/catalog";
import { fakeBackend, uiError } from "../test/backend";
import { catalog, workedDay } from "../test/fixtures";
import { mountInApp, withToast } from "../test/mount";
import { clickButton } from "../test/page";
import OverviewView from "./OverviewView.vue";

const TODAY = "2026-10-18";
const COPY = "plugin:clipboard-manager|write_text";

/** A backend whose every day is the worked day, except the days in `empty`. */
function backend(empty: string[] = [], running = false) {
  return fakeBackend({
    get_today: () => TODAY,
    get_overview: ({ day }: { day: string }) => (empty.includes(day) ? { date: day, clients: [] } : workedDay(day, running)),
    [COPY]: () => null,
  });
}

async function mountOverview(separator: Separator = "dot") {
  const wrapper = mountInApp(withToast(OverviewView));
  useCatalogStore().catalog = { ...catalog, settings: { ...catalog.settings, decimalSeparator: separator } };
  await flushPromises();
  return wrapper;
}

/** The cells of the row for this project, or of the client's total row. */
function cells(wrapper: VueWrapper, client: string, project?: string): string[] {
  const block = wrapper.get(`[data-client="${client}"]`);
  const row = project === undefined ? block.get("tr.total") : block.get(`[data-project="${project}"]`);
  return row.findAll("th, td").map((c) => c.text());
}

describe("OverviewView", () => {
  it("shows today's values to book with raw and diff (R14, R15)", async () => {
    const fake = backend();
    const wrapper = await mountOverview();

    expect(fake.called("get_overview")).toEqual([{ day: TODAY }]);
    // Project level: one value to book per project, and a subtotal.
    expect(cells(wrapper, "Acme", "Web")).toEqual(["Web", "1.25Copy", "1.37", "-0.12"]);
    expect(cells(wrapper, "Acme")).toEqual(["Subtotal", "1.25", "1.37", "-0.12"]);
    // Client level: the projects are raw only; the client total is the value to book.
    expect(cells(wrapper, "Beta", "API")).toEqual(["API", "", "0.75", ""]);
    expect(cells(wrapper, "Beta", "Ops")).toEqual(["Ops", "", "0.22", ""]);
    expect(cells(wrapper, "Beta")).toEqual(["Client total", "1.00Copy", "0.97", "+0.03"]);
    expect(wrapper.get('[data-client="Beta"]').text()).toContain("30 min · up · per client");
    expect(wrapper.find(".running").exists()).toBe(false);
  });

  it("copies just the number, with the separator from the settings (R15)", async () => {
    const fake = backend();
    const wrapper = await mountOverview("comma");
    expect(cells(wrapper, "Acme", "Web")).toEqual(["Web", "1,25Copy", "1,37", "-0,12"]);

    await clickButton("Copy 1,25");
    expect(fake.called(COPY)).toEqual([{ text: "1,25" }]);
    expect(wrapper.get('[aria-label="Copy 1,25"]').text()).toBe("Copied");

    await clickButton("Copy 1,00");
    expect(fake.called(COPY)).toEqual([{ text: "1,25" }, { text: "1,00" }]);
  });

  it("shows a failed copy", async () => {
    const fake = backend();
    fake.handlers[COPY] = () => {
      throw "clipboard busy";
    };
    await mountOverview();

    await clickButton("Copy 1.25");
    expect(document.body.textContent).toContain("clipboard busy");
  });

  it("steps through the days and back to today (R14)", async () => {
    const fake = backend(["2026-10-17"]);
    const wrapper = await mountOverview();
    const todayButton = () => wrapper.findAll("button").find((b) => b.text() === "Today")!;
    expect(todayButton().attributes("disabled")).toBeDefined();

    await clickButton("Previous day");
    expect(wrapper.text()).toContain("No time tracked on this day.");
    expect(wrapper.find("[data-client]").exists()).toBe(false);
    expect(todayButton().attributes("disabled")).toBeUndefined();

    await clickButton("Previous day");
    await clickButton("Next day");
    await clickButton("Next day");
    await clickButton("Next day");
    await clickButton("Today");
    expect(fake.called("get_overview").map((a) => a.day)).toEqual([
      TODAY,
      "2026-10-17",
      "2026-10-16",
      "2026-10-17",
      TODAY,
      "2026-10-19",
      TODAY,
    ]);
    expect(wrapper.find('[data-client="Acme"]').exists()).toBe(true);
  });

  it("reloads when the data changes", async () => {
    const fake = backend();
    await mountOverview();

    await emit("data-changed");
    await flushPromises();
    expect(fake.called("get_overview")).toEqual([{ day: TODAY }, { day: TODAY }]);
  });

  it("marks the running entry and reloads it each minute (R14)", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    const fake = backend([], true);
    const wrapper = await mountOverview();

    expect(wrapper.get('[data-client="Beta"] .client-head').text()).toContain("running");
    expect(wrapper.get('[data-project="Ops"]').text()).toContain("running");
    expect(wrapper.get('[data-project="API"]').text()).not.toContain("running");
    expect(wrapper.get('[data-client="Acme"]').text()).not.toContain("running");

    await vi.advanceTimersByTimeAsync(60_000);
    await flushPromises();
    expect(fake.called("get_overview")).toHaveLength(2);
  });

  it("doesn't reload each minute with nothing running", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    const fake = backend();
    await mountOverview();

    await vi.advanceTimersByTimeAsync(180_000);
    await flushPromises();
    expect(fake.called("get_overview")).toHaveLength(1);
  });

  it("shows a failed load", async () => {
    fakeBackend({
      get_today: () => TODAY,
      get_overview: () => {
        throw uiError("the database is broken", "broken");
      },
    });
    const wrapper = await mountOverview();
    expect(wrapper.get(".error").text()).toBe("the database is broken");
  });

  it("ignores an answer for a day that's no longer shown", async () => {
    let answerLate: (() => void) | undefined;
    fakeBackend({
      get_today: () => TODAY,
      get_overview: ({ day }: { day: string }) =>
        day === TODAY
          ? workedDay(day)
          : new Promise((resolve) => (answerLate = () => resolve({ date: day, clients: [] }))),
    });
    const wrapper = await mountOverview();

    await clickButton("Previous day");
    await clickButton("Today");
    answerLate!();
    await flushPromises();
    // Still today's overview: the stale, empty answer for the 17th didn't replace it.
    expect(wrapper.find('[data-client="Acme"]').exists()).toBe(true);
    expect(wrapper.text()).not.toContain("No time tracked");
  });
});
