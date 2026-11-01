import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import BrandLogo from "./BrandLogo.vue";

describe("BrandLogo", () => {
  it("shows the ThorSoft logo from the original SVG, at the given size (R25)", () => {
    const img = mount(BrandLogo, { props: { size: 64 } }).get("img");
    expect(img.attributes("alt")).toBe("ThorSoft");
    expect(img.attributes("src")).toMatch(/thorsoft-logo\.svg$/);
    expect(img.attributes("width")).toBe("64");
    expect(img.attributes("height")).toBe("64");
  });

  it("switches to the light variant in dark mode, by the theme's media query", () => {
    const source = mount(BrandLogo).get("picture source");
    expect(source.attributes("media")).toBe("(prefers-color-scheme: dark)");
    expect(source.attributes("srcset")).toMatch(/thorsoft-logo-dark\.svg$/);
  });

  it("is 24 px by default", () => {
    expect(mount(BrandLogo).get("img").attributes("width")).toBe("24");
  });
});
