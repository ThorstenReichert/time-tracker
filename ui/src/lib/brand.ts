// The ThorSoft brand colours and the PrimeVue theme built from them (R25).
// Steps 500, 800 and 950 of `brand` are the logo's own colours (src/assets/thorsoft-logo.svg);
// the other steps are interpolated around them. The app follows Windows' light or dark mode:
// light is white with the brand blue as accent, dark is navy (`night`) with a light brand accent.
import { definePreset } from "@primeuix/themes";
import Aura from "@primeuix/themes/aura";

export const brand = {
  50: "#f2f5f9",
  100: "#e1e8f1",
  200: "#c6d2e3",
  300: "#9fb2cd",
  400: "#718cb0",
  500: "#436088",
  600: "#375178",
  700: "#2c4265",
  800: "#1f3453",
  900: "#172843",
  950: "#0d1d37",
} as const;

/** Dark mode's surfaces: a muted navy grey from the logo's hue (Aura's dark mode uses zinc). */
export const night = {
  0: "#ffffff",
  50: "#f4f6fa",
  100: "#e6eaf1",
  200: "#cbd3e0",
  300: "#a9b5c8",
  400: "#8291aa",
  500: "#61718c",
  600: "#48566f",
  700: "#344259",
  800: "#233149",
  900: "#17233a",
  950: "#0e182b",
} as const;

/** The logo's colours and the ones its dark-mode variant (thorsoft-logo-dark.svg) uses instead.
 *  The shading is kept: the top faces are the lightest, then the sides, then the front. */
export const darkLogoColours = {
  [brand[500]]: brand[50], // top faces
  [brand[800]]: brand[200], // sides
  [brand[950]]: brand[300], // front
} as const;

/** Aura with the brand scale as `primary`. Light: Aura's white and slate surfaces, accent brand
 *  500. Dark: `night` surfaces, accent brand 300 (Aura's 400 is too faint on navy). */
export const ThorSoftPreset = definePreset(Aura, {
  semantic: {
    primary: brand,
    colorScheme: {
      dark: {
        surface: night,
        primary: {
          color: "{primary.300}",
          contrastColor: "{surface.950}",
          hoverColor: "{primary.200}",
          activeColor: "{primary.100}",
        },
        highlight: {
          background: "color-mix(in srgb, {primary.300}, transparent 84%)",
          focusBackground: "color-mix(in srgb, {primary.300}, transparent 76%)",
          color: "rgba(255,255,255,.87)",
          focusColor: "rgba(255,255,255,.87)",
        },
      },
    },
  },
});
