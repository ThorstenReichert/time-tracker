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
    rename_project: () => null,
    create_task: () => null,
    rename_task: () => null,
  });
  const wrapper = mountInApp(ClientsView);
  useCatalogStore().catalog = catalog;
  await flushPromises();
  /** The section of one client. */
  const client = (name: string) => wrapper.findAll("section.client").find((s) => s.get(".client-name").text() === name)!;
  /** Saves the open dialog with this name. */
  const saveAs = async (name: string) => {
    await dialog().get("input").setValue(name);
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
    await saveAs("Gamma");
    expect(backend.called("create_client")).toEqual([
      { name: "Gamma", rule: { unit: 15, direction: "nearest", level: "project" } },
    ]);
    expect(dialogOpen()).toBe(false);
  });

  it("edits a client", async () => {
    const { backend, client, saveAs } = await view();
    await clickButton("Edit", client("Beta"));
    expect(dialog().text()).toContain("Edit client Beta");
    await saveAs("Beta GmbH");
    expect(backend.called("update_client")).toEqual([
      { id: 2, name: "Beta GmbH", rule: { unit: 30, direction: "up", level: "client" } },
    ]);
  });

  it("creates and renames projects and tasks of the right parent", async () => {
    const { backend, client, saveAs } = await view();
    await clickButton("New project", client("Beta"));
    expect(dialog().text()).toContain("New project for Beta");
    await saveAs("Support");
    await clickButton("Rename", client("Acme"));
    expect(dialog().text()).toContain("Rename project Intern");
    await saveAs("Internal");
    await clickButton("New task", client("Acme"));
    await saveAs("Deploy");
    await clickButton("Review", client("Acme"));
    expect(dialog().text()).toContain("Rename task Review");
    await saveAs("Code review");
    expect(backend.called("create_project")).toEqual([{ client: 2, name: "Support" }]);
    expect(backend.called("rename_project")).toEqual([{ id: 2, name: "Internal" }]);
    expect(backend.called("create_task")).toEqual([{ project: 2, name: "Deploy" }]);
    expect(backend.called("rename_task")).toEqual([{ id: 1, name: "Code review" }]);
  });
});
