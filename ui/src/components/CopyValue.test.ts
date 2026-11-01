import { describe, expect, it, vi } from "vitest";
import { defineComponent, h } from "vue";
import { fakeBackend } from "../test/backend";
import { mountInApp, withOverlays } from "../test/mount";
import { clickButton } from "../test/page";
import CopyValue from "./CopyValue.vue";

const COPY = "plugin:clipboard-manager|write_text";

describe("CopyValue", () => {
  it("is a button showing the value, which copies just the number and confirms it for a moment (R15)", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    const backend = fakeBackend({ [COPY]: () => null });
    const wrapper = mountInApp(CopyValue, { props: { value: "1,25" } });
    const button = () => wrapper.get('button[aria-label="Copy 1,25"]');
    // The value is the button's whole text: no separate "Copy" label.
    expect(button().text()).toBe("1,25");
    expect(button().attributes("title")).toBe("Copy");
    expect(wrapper.findAll("button")).toHaveLength(1);

    await clickButton("Copy 1,25");
    expect(backend.called(COPY)).toEqual([{ text: "1,25" }]);
    expect(button().find(".copied").exists()).toBe(true);
    expect(button().get(".value").text()).toBe("1,25");
    expect(button().attributes("title")).toBe("Copied");

    await vi.advanceTimersByTimeAsync(1500);
    expect(button().find(".copied").exists()).toBe(false);
    expect(button().text()).toBe("1,25");
    expect(button().attributes("title")).toBe("Copy");
  });

  it("shows a failure as a toast", async () => {
    fakeBackend({
      [COPY]: () => {
        throw "clipboard busy";
      },
    });
    const wrapper = mountInApp(withOverlays(defineComponent({ render: () => h(CopyValue, { value: "1.25" }) })));

    await clickButton("Copy 1.25");
    expect(document.body.textContent).toContain("clipboard busy");
    expect(wrapper.get('[aria-label="Copy 1.25"]').find(".copied").exists()).toBe(false);
  });
});
