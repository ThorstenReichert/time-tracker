import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { useCatalogStore } from "../stores/catalog";
import { fakeBackend, uiError } from "../test/backend";
import { catalog } from "../test/fixtures";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton, dialog, dialogOpen } from "../test/page";
import ClientsView from "./ClientsView.vue";

/** An archived project of Acme. */
const legacy = { id: 5, client: 1, name: "Legacy", abbreviation: "LG", archived: true };
const confirmation = () => document.querySelector("[role=alertdialog]");

async function view() {
  const backend = fakeBackend({
    create_client: () => null,
    update_client: () => null,
    create_project: () => null,
    update_project: () => null,
    create_task: () => null,
    update_task: () => null,
    archive_project: () => null,
    unarchive_project: () => null,
    check_project_name: ({ name }: { name: string }) =>
      name.trim().toLowerCase() === "legacy"
        ? { kind: "unarchive", items: [{ id: 5, label: 'archived project "Legacy" (LG)' }] }
        : { kind: "free", items: [] },
    archive_client: () => null,
    unarchive_client: () => null,
    check_client_name: ({ name }: { name: string }) =>
      name.trim().toLowerCase() === "old corp"
        ? { kind: "unarchive", items: [{ id: 3, label: 'archived client "Old Corp" (OC)' }] }
        : { kind: "free", items: [] },
  });
  const wrapper = mountInApp(withOverlays(ClientsView));
  useCatalogStore().catalog = catalog;
  await flushPromises();
  /** The section of one client. */
  const client = (name: string) => wrapper.findAll("section.client").find((s) => s.get(".client-name").text() === name)!;
  /** Saves the open dialog with this name, and this abbreviation if given. */
  const saveAs = async (name: string, abbreviation?: string) => {
    const [nameInput, abbreviationInput] = dialog().findAll("input");
    await nameInput!.setValue(name);
    if (abbreviation !== undefined) await abbreviationInput!.setValue(abbreviation);
    await clickButton("Save", dialog());
  };
  return { backend, wrapper, client, saveAs };
}

describe("ClientsView", () => {
  it("asks for a first client while there are none", async () => {
    fakeBackend({});
    const wrapper = mountInApp(ClientsView);
    await flushPromises();
    expect(wrapper.text()).toContain("No clients yet.");
  });

  it("shows clients with their rule, projects and tasks, sorted by name (R1, R17)", async () => {
    const { wrapper, client } = await view();
    expect(wrapper.findAll(".client-name").map((c) => c.text())).toEqual(["Acme", "Beta"]);
    expect(client("Beta").text()).toContain("30 min · up · per client");
    expect(client("Acme").findAll(".project-name").map((p) => p.text())).toEqual(["Intern", "Web"]);
    expect(client("Acme").findAll(".tasks button").map((t) => t.text())).toEqual(["Docs", "Review"]);
  });

  it("creates a client", async () => {
    const { backend, saveAs } = await view();
    await clickButton("New client");
    expect(dialog().text()).toContain("New client");
    await saveAs("Gamma", "G");
    expect(backend.called("create_client")).toEqual([
      { name: "Gamma", abbreviation: "G", rule: { unit: 15, direction: "nearest", level: "project" } },
    ]);
    expect(dialogOpen()).toBe(false);
  });

  it("edits a client", async () => {
    const { backend, client, saveAs } = await view();
    await clickButton("Edit", client("Beta"));
    expect(dialog().text()).toContain("Edit client Beta");
    await saveAs("Beta GmbH");
    expect(backend.called("update_client")).toEqual([
      { id: 2, name: "Beta GmbH", abbreviation: "", rule: { unit: 30, direction: "up", level: "client" } },
    ]);
  });

  it("creates and edits projects and tasks of the right parent (R24)", async () => {
    const { backend, client, saveAs } = await view();
    await clickButton("New project", client("Beta"));
    expect(dialog().text()).toContain("New project for Beta");
    await saveAs("Support", "Sup");
    const intern = client("Acme").findAll(".project")[0]!;
    await clickButton("Edit", intern);
    expect(dialog().text()).toContain("Edit project Intern");
    await saveAs("Internal", "Int");
    await clickButton("New task", client("Acme"));
    await saveAs("Deploy");
    await clickButton("Review", client("Acme"));
    expect(dialog().text()).toContain("Edit task Review");
    await saveAs("Code review", "  ");
    expect(backend.called("create_project")).toEqual([{ client: 2, name: "Support", abbreviation: "Sup" }]);
    expect(backend.called("update_project")).toEqual([{ id: 2, name: "Internal", abbreviation: "Int" }]);
    expect(backend.called("create_task")).toEqual([{ project: 2, name: "Deploy", abbreviation: "" }]);
    expect(backend.called("update_task")).toEqual([{ id: 1, name: "Code review", abbreviation: "  " }]);
  });

  it("shows abbreviations next to the names and starts an edit with them (R24)", async () => {
    const { backend, client, saveAs } = await view();
    useCatalogStore().catalog = {
      ...catalog,
      clients: catalog.clients.map((c) => (c.id === 1 ? { ...c, abbreviation: "AC" } : c)),
      projects: catalog.projects.map((p) => (p.id === 1 ? { ...p, abbreviation: "W" } : p)),
      tasks: catalog.tasks.map((t) => (t.id === 1 ? { ...t, abbreviation: "Rev" } : t)),
    };
    await flushPromises();
    expect(client("Acme").findAll(".abbreviation").map((a) => a.text())).toEqual(["AC", "W"]);
    expect(client("Acme").findAll(".tasks button").map((t) => t.text())).toEqual(["Docs", "Review (Rev)"]);
    const web = client("Acme").findAll(".project")[1]!;
    await clickButton("Edit", web);
    const [, abbreviation] = dialog().findAll("input");
    expect((abbreviation!.element as HTMLInputElement).value).toBe("W");
    await saveAs("Web");
    expect(backend.called("update_project")).toEqual([{ id: 1, name: "Web", abbreviation: "W" }]);
  });

  it("hides archived projects (R2a)", async () => {
    const { client } = await view();
    useCatalogStore().catalog = { ...catalog, projects: [...catalog.projects, legacy] };
    await flushPromises();
    expect(client("Acme").findAll(".project-name").map((p) => p.text())).toEqual(["Intern", "Web"]);
    // A client with only archived projects says so.
    useCatalogStore().catalog = {
      ...catalog,
      projects: catalog.projects.map((p) => (p.client === 2 ? { ...p, archived: true } : p)),
    };
    await flushPromises();
    expect(client("Beta").text()).toContain("No active projects.");
  });

  it("archives a project with one click, without a confirmation (R2a)", async () => {
    const { backend, client } = await view();
    await clickButton("Archive", client("Acme").findAll(".project")[1]!);
    await flushPromises();
    expect(confirmation()).toBeNull();
    expect(backend.called("archive_project")).toEqual([{ id: 1 }]);
  });

  it("shows a rejected archive as a toast (R2d)", async () => {
    const { backend, client, wrapper } = await view();
    backend.handlers.archive_project = () => {
      throw uiError("The timer is running on this project; stop it first");
    };
    await clickButton("Archive", client("Acme").findAll(".project")[1]!);
    await flushPromises();
    expect(document.body.textContent).toContain("The timer is running on this project; stop it first");
    expect(wrapper.exists()).toBe(true);
  });

  it("offers to unarchive when a new project matches an archived one (R2c)", async () => {
    const { backend, client } = await view();
    await clickButton("New project", client("Acme"));
    await dialog().get("input").setValue("legacy");
    await flushPromises();
    expect(backend.called("check_project_name").at(-1)).toEqual({ client: 1, name: "legacy", abbreviation: "" });
    expect(dialog().text()).toContain('The archived project "Legacy" (LG) exists.');
    await clickButton("Unarchive project", dialog());
    expect(backend.called("unarchive_project")).toEqual([{ id: 5 }]);
    expect(backend.called("create_project")).toEqual([]);
    expect(dialogOpen()).toBe(false);
  });

  it("checks names only for new projects", async () => {
    const { backend, client, saveAs } = await view();
    await clickButton("Edit", client("Acme").findAll(".project")[1]!);
    await saveAs("Legacy");
    expect(backend.called("check_project_name")).toEqual([]);
    expect(backend.called("update_project")).toEqual([{ id: 1, name: "Legacy", abbreviation: "" }]);
  });

  it("hides archived clients and says so when no active client is left (R2g)", async () => {
    const { wrapper } = await view();
    useCatalogStore().catalog = {
      ...catalog,
      clients: catalog.clients.map((c) => (c.id === 2 ? { ...c, archived: true } : c)),
    };
    await flushPromises();
    expect(wrapper.findAll(".client-name").map((c) => c.text())).toEqual(["Acme"]);
    useCatalogStore().catalog = { ...catalog, clients: catalog.clients.map((c) => ({ ...c, archived: true })) };
    await flushPromises();
    expect(wrapper.findAll(".client-name")).toEqual([]);
    expect(wrapper.text()).toContain("No active clients.");
  });

  it("archives a client with one click, without a confirmation (R2g)", async () => {
    const { backend, client } = await view();
    await clickButton("Archive", client("Beta").get(".client-row"));
    await flushPromises();
    expect(confirmation()).toBeNull();
    expect(backend.called("archive_client")).toEqual([{ id: 2 }]);
    expect(backend.called("archive_project")).toEqual([]);
  });

  it("shows a rejected client archive as a toast (R2k)", async () => {
    const { backend, client } = await view();
    backend.handlers.archive_client = () => {
      throw uiError("The timer is running on a project of this client; stop it first");
    };
    await clickButton("Archive", client("Acme").get(".client-row"));
    await flushPromises();
    expect(document.body.textContent).toContain("The timer is running on a project of this client; stop it first");
  });

  it("offers to unarchive when a new client matches an archived one (R2i)", async () => {
    const { backend } = await view();
    await clickButton("New client");
    await dialog().get("input").setValue("Old Corp");
    await flushPromises();
    expect(backend.called("check_client_name").at(-1)).toEqual({ name: "Old Corp", abbreviation: "" });
    expect(dialog().text()).toContain('The archived client "Old Corp" (OC) exists.');
    await clickButton("Unarchive client", dialog());
    expect(backend.called("unarchive_client")).toEqual([{ id: 3 }]);
    expect(backend.called("create_client")).toEqual([]);
    expect(dialogOpen()).toBe(false);
  });

  it("checks names only for new clients", async () => {
    const { backend, client, saveAs } = await view();
    await clickButton("Edit", client("Beta"));
    await saveAs("Old Corp");
    expect(backend.called("check_client_name")).toEqual([]);
    expect(backend.called("update_client")).toHaveLength(1);
  });

  it("names the item in each row button's accessible name, after its visible text (WCAG 2.5.3)", async () => {
    const { backend, client } = await view();
    const labels = client("Acme")
      .findAll("button")
      .map((b) => [b.text(), b.attributes("aria-label")]);
    expect(labels).toEqual([
      ["Edit", "Edit client Acme"],
      ["New project", "New project for Acme"],
      ["Archive", "Archive client Acme"],
      ["Edit", "Edit project Intern"],
      ["New task", "New task for Intern"],
      ["Archive", "Archive project Intern"],
      ["Edit", "Edit project Web"],
      ["New task", "New task for Web"],
      ["Archive", "Archive project Web"],
      ["Docs", "Edit task Docs"],
      ["Review", "Edit task Review"],
    ]);
    // A screen reader user can tell the buttons apart.
    await clickButton("Archive project Web");
    expect(backend.called("archive_project")).toEqual([{ id: 1 }]);
  });
});
