import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { CommandError } from "../lib/api";
import { uiError } from "../test/backend";
import { mountInApp } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import NameDialog, { type Naming } from "./NameDialog.vue";

type Save = (name: string, abbreviation: string | null) => Promise<void>;

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
    expect(save).toHaveBeenCalledWith("Website", null);
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
});
