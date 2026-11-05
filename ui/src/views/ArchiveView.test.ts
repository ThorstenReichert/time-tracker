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
    archived: false,
    projects: [
      { id: 6, name: "Unused", abbreviation: null, archived: true, period: null },
      { id: 4, name: "Portal", abbreviation: "PT", archived: true, period: { first: "2026-06-01", last: "2026-07-01" } },
      { id: 2, name: "Legacy", abbreviation: null, archived: true, period: { first: "2026-03-02", last: "2026-03-02" } },
    ],
  },
  {
    id: 2,
    name: "Beta",
    abbreviation: null,
    archived: false,
    projects: [
      { id: 8, name: "Ops", abbreviation: null, archived: true, period: { first: "2026-09-01", last: "2026-09-14" } },
    ],
  },
];

/** Northwind (archived) with an active and a separately archived project, and Empty Ltd. */
const withClients: ArchiveClientDto[] = [
  ...archive,
  {
    id: 3,
    name: "Northwind Instruments",
    abbreviation: "NWI",
    archived: true,
    projects: [
      { id: 10, name: "Old portal", abbreviation: null, archived: true, period: null },
      { id: 11, name: "ASCP", abbreviation: null, archived: false, period: { first: "2026-03-02", last: "2026-11-05" } },
    ],
  },
  { id: 4, name: "Empty Ltd", abbreviation: null, archived: true, projects: [] },
];
const confirmation = () => document.querySelector("[role=alertdialog]");

async function view(answer: ArchiveClientDto[] = archive) {
  const backend = fakeBackend({ get_archive: () => answer, unarchive_project: () => null, unarchive_client: () => null });
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
    expect(wrapper.text()).toContain("Nothing archived.");
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
    expect(wrapper.text()).not.toContain("Nothing archived.");
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

  describe("archived clients (R2j)", () => {
    const section = (wrapper: Awaited<ReturnType<typeof view>>["wrapper"], name: string) =>
      wrapper.findAll("section.client").find((c) => c.get(".client-name").text() === name)!;

    it("marks an archived client and lists all its projects without their own buttons", async () => {
      const { wrapper } = await view(withClients);
      const northwind = section(wrapper, "Northwind Instruments");
      expect(northwind.get(".client-row").text()).toContain("archived");
      expect(northwind.findAll(".project-name").map((p) => p.text())).toEqual(["Old portal", "ASCP"]);
      const [oldPortal, ascp] = northwind.findAll(".project");
      expect(oldPortal!.text()).toContain("archived separately");
      expect(ascp!.text()).not.toContain("archived");
      expect(northwind.findAll(".project button")).toEqual([]);
      // An active client keeps its project buttons and has no client button.
      expect(section(wrapper, "Acme").text()).not.toContain("Unarchive client");
      expect(section(wrapper, "Acme").findAll(".project button")).toHaveLength(3);
    });

    it("says No projects for an archived client without projects", async () => {
      const { wrapper } = await view(withClients);
      expect(section(wrapper, "Empty Ltd").text()).toContain("No projects");
    });

    it("unarchives a client with one click, without a confirmation", async () => {
      const { backend, wrapper } = await view(withClients);
      await clickButton("Unarchive client", section(wrapper, "Northwind Instruments"));
      await flushPromises();
      expect(confirmation()).toBeNull();
      expect(backend.called("unarchive_client")).toEqual([{ id: 3 }]);
      expect(backend.called("unarchive_project")).toEqual([]);
    });

    it("shows a rejected client unarchive as a toast", async () => {
      const { backend, wrapper } = await view(withClients);
      backend.handlers.unarchive_client = () => {
        throw uiError("Something went wrong");
      };
      await clickButton("Unarchive client", section(wrapper, "Empty Ltd"));
      await flushPromises();
      expect(document.body.textContent).toContain("Something went wrong");
    });
  });
});
