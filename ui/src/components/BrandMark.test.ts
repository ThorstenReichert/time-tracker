import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import BrandMark from "./BrandMark.vue";

describe("BrandMark", () => {
  it("shows the logo and the product name (R25)", () => {
    const wrapper = mount(BrandMark);
    expect(wrapper.get('img[alt="ThorSoft"]').attributes("width")).toBe("24");
    expect(wrapper.text()).toBe("TimeTracker");
    expect(wrapper.attributes("title")).toBe("ThorSoft TimeTracker");
  });
});
