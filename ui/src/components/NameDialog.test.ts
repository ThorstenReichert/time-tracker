import { describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { CommandError } from "../lib/api";
import { uiError } from "../test/backend";
import { mountInApp } from "../test/mount";
import { clickButton, dialog } from "../test/page";
import NameDialog from "./NameDialog.vue";

/** The dialog, opened the way ClientsView opens it: mounted closed, then made visible. */
async function open(save: (name: string) => Promise<void>, initial?: string) {
  const wrapper = mountInApp(NameDialog, {
    props: { visible: false, header: "Rename project Web", label: "Name", initial, save },
  });
  await wrapper.setProps({ visible: true });
  await flushPromises();
  return wrapper;
}

describe("NameDialog", () => {
  it("starts with the current name and saves the typed one, then closes", async () => {
    const save = vi.fn(async () => {});
    const wrapper = await open(save, "Web");
    const input = dialog().get("input");
    expect((input.element as HTMLInputElement).value).toBe("Web");
    await input.setValue("Website");
    await clickButton("Save", dialog());
    expect(save).toHaveBeenCalledWith("Website");
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });

  it("shows a rejected name and stays open (R1)", async () => {
    const save = vi.fn(async () => {
      throw new CommandError(uiError('The name is already used by project "Web"'));
    });
    const wrapper = await open(save);
    await dialog().get("input").setValue("web");
    await clickButton("Save", dialog());
    expect(dialog().text()).toContain('The name is already used by project "Web"');
    expect(wrapper.emitted("update:visible")).toBeUndefined();
  });

  it("closes on Cancel without saving", async () => {
    const save = vi.fn(async () => {});
    const wrapper = await open(save, "Web");
    await clickButton("Cancel", dialog());
    expect(save).not.toHaveBeenCalled();
    expect(wrapper.emitted("update:visible")).toEqual([[false]]);
  });
});
