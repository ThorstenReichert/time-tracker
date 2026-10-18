import { describe, expect, it, vi } from "vitest";
import { defineComponent, h } from "vue";
import { fakeBackend } from "../test/backend";
import { mountInApp, withToast } from "../test/mount";
import { clickButton } from "../test/page";
import CopyValue from "./CopyValue.vue";

const COPY = "plugin:clipboard-manager|write_text";

describe("CopyValue", () => {
  it("copies just the number and confirms it for a moment (R15)", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    const backend = fakeBackend({ [COPY]: () => null });
    const wrapper = mountInApp(CopyValue, { props: { value: "1,25" } });
    const button = () => wrapper.get('[aria-label="Copy 1,25"]');
    expect(wrapper.get(".value").text()).toBe("1,25");
    expect(button().text()).toBe("Copy");

    await clickButton("Copy 1,25");
    expect(backend.called(COPY)).toEqual([{ text: "1,25" }]);
    expect(button().text()).toBe("Copied");

    await vi.advanceTimersByTimeAsync(1500);
    expect(button().text()).toBe("Copy");
  });

  it("shows a failure as a toast", async () => {
    fakeBackend({
      [COPY]: () => {
        throw "clipboard busy";
      },
    });
    const wrapper = mountInApp(withToast(defineComponent({ render: () => h(CopyValue, { value: "1.25" }) })));

    await clickButton("Copy 1.25");
    expect(document.body.textContent).toContain("clipboard busy");
    expect(wrapper.get('[aria-label="Copy 1.25"]').text()).toBe("Copy");
  });
});
