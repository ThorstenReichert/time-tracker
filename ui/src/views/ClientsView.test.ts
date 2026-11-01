import { describe, expect, it } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { useCatalogStore } from "../stores/catalog";
import { fakeBackend } from "../test/backend";
import { catalog } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { clickButton, dialog, dialogOpen } from "../test/page";
import ClientsView from "./ClientsView.vue";

async function view() {
  const backend = fakeBackend({
    create_client: () => null,
    update_client: () => null,
    create_project: () => null,
    update_project: () => null,
    create_task: () => null,
    update_task: () => null,
  });
  const wrapper = mountInApp(ClientsView);
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
      { id: 2, name: "Beta GmbH", abbreviation: null, rule: { unit: 30, direction: "up", level: "client" } },
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
    expect(backend.called("create_task")).toEqual([{ project: 2, name: "Deploy", abbreviation: null }]);
    expect(backend.called("update_task")).toEqual([{ id: 1, name: "Code review", abbreviation: null }]);
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
});
