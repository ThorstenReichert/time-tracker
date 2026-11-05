import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { emit } from "@tauri-apps/api/event";
import type { ArchiveClientDto } from "../bindings";
import { fakeBackend, uiError } from "../test/backend";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton } from "../test/page";
import { formatShortDay } from "../lib/day";
import ArchiveView from "./ArchiveView.vue";

/** The backend's answer, already in the R2e order (tt-core sorts it). */
const archive: ArchiveClientDto[] = [
  {
    id: 1,
    name: "Acme",
    abbreviation: "AC",
    projects: [
      { id: 6, name: "Unused", abbreviation: null, period: null },
      { id: 4, name: "Portal", abbreviation: "PT", period: { first: "2026-06-01", last: "2026-07-01" } },
      { id: 2, name: "Legacy", abbreviation: null, period: { first: "2026-03-02", last: "2026-03-02" } },
    ],
  },
  {
    id: 2,
    name: "Beta",
    abbreviation: null,
    projects: [{ id: 8, name: "Ops", abbreviation: null, period: { first: "2026-09-01", last: "2026-09-14" } }],
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
    // A single day is a full range too.
    const day = formatShortDay("2026-03-02");
    expect(period(2).split("–").map((s) => s.trim())).toEqual([day, day]);
  });

  it("says so when nothing is archived", async () => {
    const { wrapper } = await view([]);
    expect(wrapper.text()).toContain("No archived projects.");
  });

  it("unarchives a project with one click, without a confirmation (R2f)", async () => {
    const { backend, projects } = await view();
    await clickButton("Unarchive", projects()[1]!);
    await flushPromises();
    expect(confirmation()).toBeNull();
    expect(backend.called("unarchive_project")).toEqual([{ id: 4 }]);
  });

  it("shows a rejected unarchive as a toast (R2f)", async () => {
    const { backend, projects } = await view();
    backend.handlers.unarchive_project = () => {
      throw uiError("Something went wrong");
    };
    await clickButton("Unarchive", projects()[0]!);
    await flushPromises();
    expect(document.body.textContent).toContain("Something went wrong");
  });

  it("says why the archive can't be loaded", async () => {
    fakeBackend({
      get_archive: () => {
        throw uiError("The database can't be read");
      },
    });
    const wrapper = mountInApp(ArchiveView);
    await flushPromises();
    expect(wrapper.get(".error").text()).toBe("The database can't be read");
    expect(wrapper.text()).not.toContain("No archived projects.");
  });

  it("reloads on data-changed, so an unarchived project leaves the view (R2f)", async () => {
    const { backend, projects } = await view();
    const [acme, beta] = archive;
    backend.handlers.get_archive = () => [{ ...acme!, projects: acme!.projects.filter((p) => p.id !== 4) }, beta!];
    await emit("data-changed");
    await flushPromises();
    expect(backend.called("get_archive")).toHaveLength(2);
    expect(projects().map((p) => p.get(".project-name").text())).toEqual(["Unused", "Legacy", "Ops"]);
  });
});
