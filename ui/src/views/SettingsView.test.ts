import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { flushPromises, type VueWrapper } from "@vue/test-utils";
import type { AutostartDto, SettingsDto } from "../bindings";
import { useCatalogStore } from "../stores/catalog";
import { fakeBackend, uiError, type Handler } from "../test/backend";
import { catalog, healthy } from "../test/fixtures";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton } from "../test/page";
import SettingsView from "./SettingsView.vue";

const DEFAULTS = { hotkey: "Ctrl+Alt+Space", backupDir: "C:\\demo\\backups" };

function backend(autostart: AutostartDto = { available: true, enabled: true }, handlers: Record<string, Handler> = {}) {
  return fakeBackend({
    get_settings_defaults: () => DEFAULTS,
    get_autostart: () => autostart,
    update_settings: () => null,
    set_autostart: ({ enabled }: { enabled: boolean }) => {
      autostart = { ...autostart, enabled };
      return null;
    },
    ...handlers,
  });
}

async function mountSettings(settings: Partial<SettingsDto> = {}) {
  const wrapper = mountInApp(withOverlays(SettingsView));
  const store = useCatalogStore();
  store.catalog = { ...catalog, settings: { ...catalog.settings, ...settings } };
  store.status = healthy;
  await flushPromises();
  return wrapper;
}

/** The input labelled `label`. */
function field(wrapper: VueWrapper, label: string) {
  const forId = wrapper.findAll("label").find((l) => l.text() === label)?.attributes("for");
  if (forId === undefined) throw new Error(`no label "${label}"`);
  return wrapper.get(`[id="${forId}"]`);
}

const value = (wrapper: VueWrapper, label: string) => (field(wrapper, label).element as HTMLInputElement).value;
const pressed = (wrapper: VueWrapper) => wrapper.findAll("[aria-pressed=true]").map((b) => b.text());

describe("SettingsView", () => {
  it("shows the ThorSoft logo, the product name and the version from package.json in About (R25)", async () => {
    backend();
    const wrapper = await mountSettings();
    const { version } = JSON.parse(readFileSync("package.json", "utf8")) as { version: string };
    expect(version).toMatch(/^\d+\.\d+\.\d+/);

    const about = wrapper.get(".about");
    expect(about.get('img[alt="ThorSoft"]').attributes("width")).toBe("64");
    expect(about.text()).toContain("ThorSoft TimeTracker");
    expect(about.text()).toContain(`Version ${version}`);
  });

  it("shows the saved settings, with the defaults for what is unset", async () => {
    backend();
    const wrapper = await mountSettings({ recentsN: 5, backupKeep: 30, decimalSeparator: "comma" });

    expect(value(wrapper, "Hotkey")).toBe("");
    expect(field(wrapper, "Hotkey").attributes("placeholder")).toBe("Ctrl+Alt+Space");
    expect(value(wrapper, "Recent combos")).toBe("5");
    expect(pressed(wrapper)).toEqual(["Comma (1,25)"]);
    expect(value(wrapper, "Folder")).toBe("");
    expect(field(wrapper, "Folder").attributes("placeholder")).toBe("C:\\demo\\backups");
    expect(value(wrapper, "Backups to keep")).toBe("30");
    // The database itself stays where it is (AC-5).
    expect(wrapper.text()).toContain("stays at C:\\demo\\timetracker.db");
  });

  it("saves every setting; blank means the default (R6, R7, R15, R22)", async () => {
    const fake = backend();
    const wrapper = await mountSettings({ hotkey: "Ctrl+Alt+K", backupDir: "D:\\old" });

    await field(wrapper, "Hotkey").setValue("  Ctrl+Shift+K ");
    await field(wrapper, "Recent combos").setValue("12");
    await clickButton("Dot (1.25)");
    await field(wrapper, "Folder").setValue("   ");
    await field(wrapper, "Backups to keep").setValue("7");
    await clickButton("Save");

    expect(fake.called("update_settings")).toEqual([
      {
        settings: { hotkey: "Ctrl+Shift+K", decimalSeparator: "dot", recentsN: 12, backupDir: null, backupKeep: 7 },
      },
    ]);
    expect(document.body.textContent).toContain("Settings saved");
  });

  it("shows a rejected save and keeps what was typed", async () => {
    backend(undefined, {
      update_settings: () => {
        throw uiError('"Ctrl+Foo" isn\'t a valid hotkey; write it like Ctrl+Alt+Space');
      },
    });
    const wrapper = await mountSettings();

    await field(wrapper, "Hotkey").setValue("Ctrl+Foo");
    await clickButton("Save");

    expect(wrapper.text()).toContain('"Ctrl+Foo" isn\'t a valid hotkey');
    expect(value(wrapper, "Hotkey")).toBe("Ctrl+Foo");
    expect(document.body.textContent).not.toContain("Settings saved");
  });

  it("asks for whole numbers instead of sending something else", async () => {
    const fake = backend();
    const wrapper = await mountSettings();

    await field(wrapper, "Backups to keep").setValue("");
    await clickButton("Save");

    expect(wrapper.text()).toContain("Enter whole numbers");
    expect(fake.called("update_settings")).toEqual([]);
  });

  it("keeps unsaved input when the data reloads, and follows a saved change", async () => {
    backend();
    const wrapper = await mountSettings();
    await field(wrapper, "Hotkey").setValue("Ctrl+Alt+J");

    // A reload for anything else, e.g. a timer switched from the popup.
    const store = useCatalogStore();
    store.catalog = { ...catalog, settings: { ...catalog.settings } };
    await flushPromises();
    expect(value(wrapper, "Hotkey")).toBe("Ctrl+Alt+J");

    store.catalog = { ...catalog, settings: { ...catalog.settings, hotkey: "Ctrl+Alt+H" } };
    await flushPromises();
    expect(value(wrapper, "Hotkey")).toBe("Ctrl+Alt+H");
  });

  it("toggles autostart and then shows what Windows has (R9)", async () => {
    const fake = backend({ available: true, enabled: true });
    const wrapper = await mountSettings();
    const toggle = field(wrapper, "Start with Windows");
    expect((toggle.element as HTMLInputElement).checked).toBe(true);

    await toggle.trigger("change");
    await flushPromises();

    expect(fake.called("set_autostart")).toEqual([{ enabled: false }]);
    expect(fake.called("get_autostart")).toHaveLength(2);
    expect((field(wrapper, "Start with Windows").element as HTMLInputElement).checked).toBe(false);
  });

  it("shows a refused autostart change as a toast", async () => {
    backend(undefined, {
      set_autostart: () => {
        throw uiError("Windows refused the change: access denied", "system");
      },
    });
    const wrapper = await mountSettings();

    await field(wrapper, "Start with Windows").trigger("change");
    await flushPromises();

    expect(document.body.textContent).toContain("Windows refused the change: access denied");
    expect((field(wrapper, "Start with Windows").element as HTMLInputElement).checked).toBe(true);
  });

  it("leaves autostart alone in a development build", async () => {
    backend({ available: false, enabled: false });
    const wrapper = await mountSettings();

    expect((field(wrapper, "Start with Windows").element as HTMLInputElement).disabled).toBe(true);
    expect(wrapper.text()).toContain("Only the installed app changes this");
  });
});
