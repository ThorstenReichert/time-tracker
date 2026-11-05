import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import type { NameCheckDto, ProjectDto } from "../bindings";
import { CommandError } from "../lib/api";
import { uiError } from "../test/backend";
import { mountInApp } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import NameDialog, { type Naming } from "./NameDialog.vue";

type Save = (name: string, abbreviation: string | null) => Promise<void>;
type Check = (name: string, abbreviation: string | null) => Promise<NameCheckDto>;
type Unarchive = (project: number) => Promise<void>;

/** The dialog, opened the way ClientsView opens it: mounted closed, then made visible. */
async function open(save: Save, initial?: Naming) {
  const wrapper = mountInApp(NameDialog, {
    props: { visible: false, header: "Edit project Web", label: "Name", initial, save },
  });
  await wrapper.setProps({ visible: true });
  await flushPromises();
  return wrapper;
}

describe("NameDialog", () => {
  it("starts with the current name and saves the typed one, then closes", async () => {
    const save = vi.fn<Save>(async () => {});
    const wrapper = await open(save, { name: "Web", abbreviation: null });
    const [name, abbreviation] = dialog().findAll("input");
    expect((name!.element as HTMLInputElement).value).toBe("Web");
    expect((abbreviation!.element as HTMLInputElement).value).toBe("");
    await name!.setValue("Website");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("Website", "");
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });

  it("starts with the current abbreviation and saves a changed one (R24)", async () => {
    const save = vi.fn<Save>(async () => {});
    await open(save, { name: "adaptive signal calibration pipeline", abbreviation: "ASCP" });
    const abbreviation = dialog().findAll("input")[1]!;
    expect((abbreviation.element as HTMLInputElement).value).toBe("ASCP");
    await abbreviation.setValue("AS");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("adaptive signal calibration pipeline", "AS");
  });

  it("shows a rejected name and stays open (R1)", async () => {
    const save = vi.fn<Save>(async () => {
      throw new CommandError(uiError('The name is already used by project "Web"'));
    });
    const wrapper = await open(save);
    await dialog().get("input").setValue("web");
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain('The name is already used by project "Web"');
    expect(wrapper.emitted("update:visible")).toBeUndefined();
  });

  it("closes on Cancel without saving", async () => {
    const save = vi.fn<Save>(async () => {});
    const wrapper = await open(save, { name: "Web", abbreviation: null });
    await clickButton("Cancel", dialog());
    expect(save).not.toHaveBeenCalled();
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });

  describe("name check for a new project (R2c)", () => {
    const oldWeb: ProjectDto = { id: 2, client: 1, name: "Old web", abbreviation: "OW", archived: true };
    const web: ProjectDto = { id: 1, client: 1, name: "Web", abbreviation: null, archived: false };

    /** A check that answers by the typed name: "old web" → unarchive, "web" → taken. */
    const check = vi.fn<Check>(async (name) => {
      const key = name.trim().toLowerCase();
      if (key === "old web") return { kind: "unarchive", projects: [oldWeb] };
      if (key === "web") return { kind: "taken", projects: [web, oldWeb] };
      return { kind: "free", projects: [] };
    });

    async function openNew(save: Save, unarchive: Unarchive) {
      const wrapper = mountInApp(NameDialog, {
        props: { visible: false, header: "New project for Acme", label: "Name", save, check, unarchive },
      });
      await wrapper.setProps({ visible: true });
      await flushPromises();
      return wrapper;
    }

    const submitButton = () => dialog().findAll("button").find((b) => b.attributes("type") === "submit")!;

    it("offers to unarchive a matching archived project instead of creating it", async () => {
      const save = vi.fn<Save>(async () => {});
      const unarchive = vi.fn<Unarchive>(async () => {});
      const wrapper = await openNew(save, unarchive);
      await dialog().get("input").setValue("Old Web");
      await flushPromises();
      expect(check).toHaveBeenLastCalledWith("Old Web", "");
      expect(dialog().text()).toContain('The archived project "Old web" (OW) exists.');
      expect(submitButton().text()).toBe("Unarchive project");

      await clickButton("Unarchive project", dialog());
      expect(unarchive).toHaveBeenCalledWith(2);
      expect(save).not.toHaveBeenCalled();
      expect(wrapper.emitted("update:visible")).toEqual([[false]]);
    });

    it("goes back to creating once the name is changed", async () => {
      const save = vi.fn<Save>(async () => {});
      const unarchive = vi.fn<Unarchive>(async () => {});
      await openNew(save, unarchive);
      const input = dialog().get("input");
      await input.setValue("Old web");
      await flushPromises();
      await input.setValue("Old webshop");
      await flushPromises();
      expect(dialog().text()).not.toContain("archived");
      await clickButton("Save", dialog());
      expect(save).toHaveBeenCalledWith("Old webshop", "");
      expect(unarchive).not.toHaveBeenCalled();
    });

    it("blocks a clash and names the projects", async () => {
      const save = vi.fn<Save>(async () => {});
      await openNew(save, vi.fn<Unarchive>(async () => {}));
      await dialog().get("input").setValue("web");
      await flushPromises();
      expect(dialog().text()).toContain('Already used by project "Web", archived project "Old web" (OW)');
      expect(submitButton().attributes("disabled")).toBeDefined();
      await submitButton().trigger("submit");
      await flushPromises();
      expect(save).not.toHaveBeenCalled();
    });

    it("keeps only the answer to the newest text", async () => {
      let answerFirst!: (r: NameCheckDto) => void;
      const slow = vi.fn<Check>((name) =>
        name === "Old web"
          ? new Promise((resolve) => (answerFirst = resolve))
          : Promise.resolve({ kind: "free", projects: [] }),
      );
      const wrapper = mountInApp(NameDialog, {
        props: { visible: false, header: "New project", label: "Name", save: async () => {}, check: slow, unarchive: async () => {} },
      });
      await wrapper.setProps({ visible: true });
      await flushPromises();
      const input = dialog().get("input");
      await input.setValue("Old web");
      await input.setValue("Mobile");
      await flushPromises();
      answerFirst({ kind: "unarchive", projects: [oldWeb] });
      await flushPromises();
      expect(dialog().text()).not.toContain("archived");
      expect(submitButton().text()).toBe("Save");
    });
  });
});
