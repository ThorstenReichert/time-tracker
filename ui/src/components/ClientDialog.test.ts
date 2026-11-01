import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import type { RuleDto } from "../bindings";
import { useCatalogStore } from "../stores/catalog";
import { CommandError } from "../lib/api";
import { uiError } from "../test/backend";
import { catalog } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import ClientDialog from "./ClientDialog.vue";

type Save = (name: string, abbreviation: string | null, rule: RuleDto) => Promise<void>;

async function open(save: Save, initial?: { name: string; abbreviation: string | null; rule: RuleDto }) {
  const wrapper = mountInApp(ClientDialog, { props: { visible: false, header: "Client", initial, save } });
  useCatalogStore().catalog = catalog;
  await wrapper.setProps({ visible: true });
  await flushPromises();
  return wrapper;
}

/** The pressed option of each SelectButton: direction, then level. */
const pressed = () =>
  dialog()
    .findAll("[aria-pressed=true]")
    .map((b) => b.text());

describe("ClientDialog", () => {
  it("starts a new client with the core's default rule (R17)", async () => {
    await open(vi.fn<Save>());
    expect(dialog().text()).toContain("15 min");
    expect(pressed()).toEqual(["Nearest", "Per project"]);
  });

  it("saves the name with the chosen direction and level", async () => {
    const save = vi.fn<Save>(async () => {});
    const wrapper = await open(save);
    await dialog().get("input").setValue("Gamma");
    await clickButton("Up", dialog());
    await clickButton("Per client", dialog());
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("Gamma", null, { unit: 15, direction: "up", level: "client" });
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });

  it("starts an edit with the client's name and rule", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save, { name: "Beta", abbreviation: "B", rule: { unit: 30, direction: "up", level: "client" } });
    const [name, abbreviation] = dialog().findAll("input");
    expect((name!.element as HTMLInputElement).value).toBe("Beta");
    expect((abbreviation!.element as HTMLInputElement).value).toBe("B");
    expect(dialog().text()).toContain("30 min");
    expect(pressed()).toEqual(["Up", "Per client"]);
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("Beta", "B", { unit: 30, direction: "up", level: "client" });
  });

  it("sends a cleared abbreviation as none (R24)", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save, { name: "Beta", abbreviation: "B", rule: { unit: 30, direction: "up", level: "client" } });
    await dialog().findAll("input")[1]!.setValue(" ");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("Beta", null, { unit: 30, direction: "up", level: "client" });
  });

  it("shows a rejected client and stays open", async () => {
    const wrapper = await open(async () => {
      throw new CommandError(uiError('The name is already used by client "Acme"'));
    });
    await dialog().get("input").setValue("acme");
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain('The name is already used by client "Acme"');
    expect(wrapper.emitted("update:visible")).toBeUndefined();
  });
});
