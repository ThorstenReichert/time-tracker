import { describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { emit } from "@tauri-apps/api/event";
import type { ComboDto, EntryDto, TaskMatchesDto } from "../bindings";
import { deferred, fakeBackend, uiError, type Handler } from "../test/backend";
import { acmeIntern, acmeWeb, acmeWebDocs, acmeWebReview, betaOps, combo, runningEntry } from "../test/fixtures";
import QuickSwitch from "./QuickSwitch.vue";

const allCombos = [acmeWebReview, betaOps, acmeWebDocs, acmeIntern, acmeWeb];

/** The popup with a running entry on Acme / Web / Review, unless `running` says otherwise. */
async function popup(options: { running?: EntryDto | null; handlers?: Record<string, Handler> } = {}) {
  const backend = fakeBackend(
    {
      get_running: () => (options.running === undefined ? runningEntry(acmeWebReview, 83) : options.running),
      quick_switch: ({ query }) => (query === "" ? allCombos : allCombos.filter((c) => c.label.toLowerCase().includes(query))),
      task_matches: (): TaskMatchesDto => ({ tasks: [{ id: 1, project: 1, name: "Review", abbreviation: null, archived: false }], offerCreate: null }),
      start: () => null,
      stop: () => null,
      "plugin:window|hide": () => null,
      ...options.handlers,
    },
    "quick-switch",
  );
  const wrapper = mount(QuickSwitch, { attachTo: document.body });
  await flushPromises();
  const input = wrapper.get("input");
  const press = async (key: string, init: KeyboardEventInit = {}) => {
    await input.trigger("keydown", { key, ...init });
    await flushPromises();
  };
  const type = async (text: string) => {
    await input.setValue(text);
    await flushPromises();
  };
  // Row labels; a row marked as running ends with " (running)".
  const rows = () =>
    wrapper.findAll("[role=option]").map((li) => li.get(".label").text() + (li.find(".tag").exists() ? " (running)" : ""));
  const highlighted = () => wrapper.get("[aria-selected=true] .label").text();
  const hidden = () => backend.called("plugin:window|hide").length > 0;
  return { backend, wrapper, input, press, type, rows, highlighted, hidden };
}

const started = (combo: ComboDto) => ({
  project: combo.project,
  task: combo.task === null ? { kind: "none" } : { kind: "existing", value: combo.task },
});

describe("QuickSwitch", () => {
  it("shows short labels with the full ones on hover, and task abbreviations in the task step (R24)", async () => {
    const ascp = combo(9, 9, 9, ["Northwind Instruments", "adaptive signal calibration pipeline", "Review"], ["NWI", "ASCP", "Rev"]);
    const { wrapper, rows, press } = await popup({
      running: runningEntry(ascp, 60),
      handlers: {
        quick_switch: () => [ascp, acmeWeb],
        task_matches: (): TaskMatchesDto => ({
          tasks: [{ id: 9, project: 9, name: "Review", abbreviation: "Rev", archived: false }],
          offerCreate: null,
        }),
      },
    });
    expect(wrapper.get("header .label").text()).toBe("NWI / ASCP / Rev");
    expect(rows()).toEqual(["Acme / Web"]);
    await wrapper.get("input").setValue("ascp");
    await flushPromises();
    const row = wrapper.get("[role=option]");
    expect(row.get(".label").text()).toBe("NWI / ASCP / Rev");
    expect(row.attributes("title")).toBe("Northwind Instruments / adaptive signal calibration pipeline / Review");
    await press("Tab");
    expect(wrapper.get(".chip").text()).toBe("NWI / ASCP");
    expect(wrapper.get(".chip").attributes("title")).toBe("Northwind Instruments / adaptive signal calibration pipeline");
    expect(wrapper.findAll("[role=option] .hint").map((h) => h.text())).toEqual(["Rev"]);
  });

  it("shows the running entry with its elapsed time on top", async () => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-10-18T10:00:00Z"));
    const { wrapper } = await popup();
    expect(wrapper.get("header").text()).toContain("Acme / Web / Review");
    expect(wrapper.get("header").text()).toContain("0:01:23");
  });

  it("lists the other combos with the previous one highlighted, so hotkey + Enter switches back", async () => {
    const { rows, highlighted, press, backend, hidden } = await popup();
    expect(rows()).toEqual(["Beta / Ops", "Acme / Web / Docs", "Acme / Intern", "Acme / Web"]);
    expect(highlighted()).toBe("Beta / Ops");
    await press("Enter");
    expect(backend.called("start")).toEqual([started(betaOps)]);
    expect(hidden()).toBe(true);
  });

  it("filters as you type and keeps a match on the running combo, marked", async () => {
    const { rows, type, backend } = await popup();
    await type("rev");
    expect(backend.called("quick_switch").at(-1)).toEqual({ query: "rev" });
    expect(rows()).toEqual(["Acme / Web / Review (running)"]);
  });

  it("moves the highlight with the arrow keys", async () => {
    const { press, highlighted, backend } = await popup();
    await press("ArrowDown");
    await press("ArrowDown");
    await press("ArrowUp");
    expect(highlighted()).toBe("Acme / Web / Docs");
    await press("Enter");
    expect(backend.called("start")).toEqual([started(acmeWebDocs)]);
  });

  it("starts a clicked row", async () => {
    const { wrapper, backend } = await popup();
    await wrapper.findAll("[role=option]")[2]!.trigger("click");
    await flushPromises();
    expect(backend.called("start")).toEqual([started(acmeIntern)]);
  });

  it("waits for the latest search when Enter comes before its result", async () => {
    const slow = deferred<ComboDto[]>();
    const { input, press, backend } = await popup({
      handlers: { quick_switch: ({ query }) => (query === "intern" ? slow.promise : allCombos) },
    });
    await input.setValue("intern");
    await press("Enter");
    expect(backend.called("start")).toEqual([]);
    slow.resolve([acmeIntern]);
    await flushPromises();
    expect(backend.called("start")).toEqual([started(acmeIntern)]);
  });

  it("picks a task of the highlighted project after Tab, with No task first (R3)", async () => {
    const { press, wrapper, rows, backend } = await popup();
    await press("ArrowDown"); // Acme / Web / Docs: Tab takes its project
    await press("Tab");
    expect(wrapper.get(".chip").text()).toBe("Acme / Web");
    expect(backend.called("task_matches").at(-1)).toEqual({ project: 1, text: "" });
    expect(rows()).toEqual(["No task", "Review (running)"]);
    await press("Enter");
    expect(backend.called("start")).toEqual([{ project: 1, task: { kind: "none" } }]);
  });

  it("creates a typed task in the task step (R3)", async () => {
    const { press, type, rows, backend } = await popup({
      handlers: { task_matches: ({ text }): TaskMatchesDto => ({ tasks: [], offerCreate: text === "" ? null : text }) },
    });
    await press("Tab");
    await type("Lit");
    expect(rows()).toEqual(['Create "Lit"']);
    await press("Enter");
    expect(backend.called("start")).toEqual([{ project: 4, task: { kind: "new", value: "Lit" } }]);
  });

  it("goes back to the combos with Shift+Tab or Backspace, keeping the query", async () => {
    const { press, type, input, wrapper } = await popup();
    await type("web");
    await press("Tab");
    expect(input.element.value).toBe("");
    await press("Tab", { shiftKey: true });
    expect(wrapper.find(".chip").exists()).toBe(false);
    expect(input.element.value).toBe("web");
    await press("Tab");
    await press("Backspace");
    expect(wrapper.find(".chip").exists()).toBe(false);
    expect(input.element.value).toBe("web");
  });

  it("stops the running timer with Ctrl+S and hides", async () => {
    const { press, backend, hidden } = await popup();
    await press("s", { ctrlKey: true });
    expect(backend.called("stop")).toEqual([{ at: null }]);
    expect(hidden()).toBe(true);
  });

  it("says so when Ctrl+S has nothing to stop", async () => {
    const { press, backend, wrapper, hidden } = await popup({ running: null });
    expect(wrapper.get("header").text()).toBe("No timer running");
    await press("s", { ctrlKey: true });
    expect(backend.called("stop")).toEqual([]);
    expect(wrapper.get(".error").text()).toBe("No timer is running.");
    expect(hidden()).toBe(false);
  });

  it("shows a rejected switch and stays open", async () => {
    const { press, wrapper, hidden } = await popup({
      handlers: {
        start: () => {
          throw uiError("Overlaps Acme / Web 09:00–10:22");
        },
      },
    });
    await press("Enter");
    expect(wrapper.get(".error").text()).toBe("Overlaps Acme / Web 09:00–10:22");
    expect(hidden()).toBe(false);
  });

  it("hides on Esc without changing anything, and starts fresh next time", async () => {
    const { press, type, input, backend, hidden } = await popup();
    await type("web");
    await press("Escape");
    expect(hidden()).toBe(true);
    expect(input.element.value).toBe("");
    expect(backend.called("start")).toEqual([]);
    expect(backend.called("stop")).toEqual([]);
  });

  it("highlights the first row again next time, even before the reload answers", async () => {
    const { press, backend } = await popup();
    await press("ArrowDown");
    await press("ArrowDown");
    await press("Escape");

    const slow = deferred<EntryDto>();
    backend.handlers.get_running = () => slow.promise;
    await emit("quick-switch-shown");
    await press("Enter");
    expect(backend.called("start")).toEqual([started(betaOps)]);
    slow.resolve(runningEntry(acmeWebReview, 90));
  });

  it("starts fresh after losing focus", async () => {
    const { type, press, input, wrapper } = await popup();
    await type("web");
    await press("Tab");
    window.dispatchEvent(new Event("blur"));
    await flushPromises();
    expect(wrapper.find(".chip").exists()).toBe(false);
    expect(input.element.value).toBe("");
  });

  it("reloads when the data changes", async () => {
    const { backend, wrapper } = await popup();
    backend.handlers.get_running = () => runningEntry(betaOps, 5);
    await emit("data-changed");
    await flushPromises();
    expect(wrapper.get("header").text()).toContain("Beta / Ops");
  });
});
