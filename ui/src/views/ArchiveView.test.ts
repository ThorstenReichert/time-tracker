import { describe, expect, it } from "vitest";
import { DOMWrapper, flushPromises } from "@vue/test-utils";
import { emit } from "@tauri-apps/api/event";
import type { ArchiveClientDto } from "../bindings";
import { fakeBackend, uiError } from "../test/backend";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton } from "../test/page";
import ArchiveView from "./ArchiveView.vue";

/** The backend's answer, already in the R2e order (tt-core sorts it). */
const archive: ArchiveClientDto[] = [
  {
    id: 1,
    name: "Acme",
    abbreviation: "AC",
    projects: [
      { id: 6, name: "Unused", abbreviation: null, first: null, last: null },
      { id: 4, name: "Portal", abbreviation: "PT", first: "2026-06-01", last: "2026-07-01" },
      { id: 2, name: "Legacy", abbreviation: null, first: "2026-03-02", last: "2026-03-02" },
    ],
  },
  {
    id: 2,
    name: "Beta",
    abbreviation: null,
    projects: [{ id: 8, name: "Ops", abbreviation: null, first: "2026-09-01", last: "2026-09-14" }],
  },
];
const confirmation = () => document.querySelector("[role=alertdialog]");

async function view(answer: ArchiveClientDto[] = archive) {
  const backend = fakeBackend({ get_archive: () => answer, unarchive_project: () => null });
  const wrapper = mountInApp(withOverlays(ArchiveView));
  await flushPromises();
  const projects = () => wrapper.findAll(".project");
  return { backend, wrapper, projects };
}

describe("ArchiveView", () => {
  it("lists archived projects by client in the backend's order (R2e)", async () => {
    const { wrapper, projects } = await view();
    expect(wrapper.findAll(".client-name").map((c) => c.text())).toEqual(["Acme", "Beta"]);
    expect(projects().map((p) => p.get(".project-name").text())).toEqual(["Unused", "Portal", "Legacy", "Ops"]);
    expect(projects()[1]!.text()).toContain("PT");
  });

  it("shows the active period, or No entries (R2e)", async () => {
    const { projects } = await view();
    const period = (i: number) => projects()[i]!.get(".period").text();
    expect(period(0)).toBe("No entries");
    expect(period(1)).toMatch(/2026.*–.*2026/);
    // A single day shows once.
    expect(period(2)).not.toContain("–");
  });

  it("says so when nothing is archived", async () => {
    const { wrapper } = await view([]);
    expect(wrapper.text()).toContain("No archived projects.");
  });

  it("unarchives a project after a confirmation (R2f)", async () => {
    const { backend, projects } = await view();
    const portal = projects()[1]!;
    await clickButton("Unarchive", portal);
    expect(confirmation()?.textContent).toContain("Acme / Portal can be booked again");
    await clickButton("Cancel", new DOMWrapper(confirmation()!));
    expect(backend.called("unarchive_project")).toEqual([]);

    await clickButton("Unarchive", portal);
    await clickButton("Unarchive", new DOMWrapper(confirmation()!));
    expect(backend.called("unarchive_project")).toEqual([{ id: 4 }]);
  });

  it("shows a rejected unarchive as a toast (R2f)", async () => {
    const { backend, projects } = await view();
    backend.handlers.unarchive_project = () => {
      throw uiError("Something went wrong");
    };
    await clickButton("Unarchive", projects()[0]!);
    await clickButton("Unarchive", new DOMWrapper(confirmation()!));
    await flushPromises();
    expect(document.body.textContent).toContain("Something went wrong");
  });

  it("reloads on data-changed", async () => {
    const { backend } = await view();
    await emit("data-changed");
    await flushPromises();
    expect(backend.called("get_archive")).toHaveLength(2);
  });
});
