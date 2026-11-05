import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import type { CheckedItemDto, NameCheckDto, RuleDto } from "../bindings";
import { useCatalogStore } from "../stores/catalog";
import { CommandError } from "../lib/api";
import { uiError } from "../test/backend";
import { catalog } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import ClientDialog from "./ClientDialog.vue";

type Save = (name: string, abbreviation: string | null, rule: RuleDto) => Promise<void>;
type Check = (name: string, abbreviation: string | null) => Promise<NameCheckDto>;
type Unarchive = (client: number) => Promise<void>;

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
    expect(save).toHaveBeenCalledWith("Gamma", "", { unit: 15, direction: "up", level: "client" });
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

  it("sends a cleared abbreviation as typed, which the core reads as none (R24)", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save, { name: "Beta", abbreviation: "B", rule: { unit: 30, direction: "up", level: "client" } });
    await dialog().findAll("input")[1]!.setValue(" ");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("Beta", " ", { unit: 30, direction: "up", level: "client" });
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

  describe("name check for a new client (R2i)", () => {
    const oldCorp: CheckedItemDto = { id: 3, label: 'archived client "Old Corp" (OC)' };
    const acme: CheckedItemDto = { id: 1, label: 'client "Acme"' };

    /** A check that answers by the typed name: "old corp" → unarchive, "acme" → taken. */
    const check = vi.fn<Check>(async (name) => {
      const key = name.trim().toLowerCase();
      if (key === "old corp") return { kind: "unarchive", items: [oldCorp] };
      if (key === "acme") return { kind: "taken", items: [acme] };
      return { kind: "free", items: [] };
    });

    async function openNew(save: Save, unarchive: Unarchive, checker: Check = check) {
      const wrapper = mountInApp(ClientDialog, {
        props: { visible: false, header: "New client", save, check: checker, unarchive },
      });
      useCatalogStore().catalog = catalog;
      await wrapper.setProps({ visible: true });
      await flushPromises();
      return wrapper;
    }

    const submitButton = () => dialog().findAll("button").find((b) => b.attributes("type") === "submit")!;

    it("offers to unarchive a matching archived client, ignoring the typed rule", async () => {
      const save = vi.fn<Save>(async () => {});
      const unarchive = vi.fn<Unarchive>(async () => {});
      const wrapper = await openNew(save, unarchive);
      await dialog().get("input").setValue("Old Corp");
      await flushPromises();
      expect(check).toHaveBeenLastCalledWith("Old Corp", "");
      expect(dialog().text()).toContain('The archived client "Old Corp" (OC) exists.');
      expect(submitButton().text()).toBe("Unarchive client");
      await clickButton("Up", dialog());
      await clickButton("Unarchive client", dialog());
      expect(unarchive).toHaveBeenCalledWith(3);
      expect(save).not.toHaveBeenCalled();
      expect(wrapper.emitted("update:visible")).toEqual([[false]]);
    });

    it("blocks a clash and names the client", async () => {
      const save = vi.fn<Save>(async () => {});
      await openNew(save, vi.fn<Unarchive>(async () => {}));
      await dialog().get("input").setValue("acme");
      await flushPromises();
      expect(dialog().text()).toContain('Already used by client "Acme"');
      expect(submitButton().attributes("disabled")).toBeDefined();
      await submitButton().trigger("submit");
      await flushPromises();
      expect(save).not.toHaveBeenCalled();
    });

    it("saves, not unarchives, when the answer for the new text is still pending", async () => {
      const pending = vi.fn<Check>((name) =>
        name === "Old Corp" ? Promise.resolve({ kind: "unarchive", items: [oldCorp] }) : new Promise(() => {}),
      );
      const save = vi.fn<Save>(async () => {});
      const unarchive = vi.fn<Unarchive>(async () => {});
      await openNew(save, unarchive, pending);
      const input = dialog().get("input");
      await input.setValue("Old Corp");
      await flushPromises();
      expect(submitButton().text()).toBe("Unarchive client");
      await input.setValue("Old Corp 2");
      expect(submitButton().text()).toBe("Save");
      await submitButton().trigger("submit");
      await flushPromises();
      expect(unarchive).not.toHaveBeenCalled();
      expect(save).toHaveBeenCalledWith("Old Corp 2", "", { unit: 15, direction: "nearest", level: "project" });
    });

    it("drops an answer that arrives after the dialog is reused for an edit", async () => {
      let answer!: (r: NameCheckDto) => void;
      const slow = vi.fn<Check>(() => new Promise((resolve) => (answer = resolve)));
      const save = vi.fn<Save>(async () => {});
      const wrapper = await openNew(save, async () => {}, slow);
      await dialog().get("input").setValue("acme");
      await wrapper.setProps({ visible: false });
      await wrapper.setProps({
        visible: true,
        header: "Edit client Acme",
        check: undefined,
        unarchive: undefined,
        initial: { name: "acme", abbreviation: null, rule: { unit: 15, direction: "nearest", level: "project" } },
      });
      await flushPromises();
      answer({ kind: "taken", items: [acme] });
      await flushPromises();
      expect(dialog().text()).not.toContain("Already used");
      await clickButton("Save", dialog());
      expect(save).toHaveBeenCalledTimes(1);
    });
  });
});
