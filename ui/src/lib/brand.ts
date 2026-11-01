// The ThorSoft brand colours and the PrimeVue theme built from them (R25).
// Steps 500, 800 and 950 are the logo's own colours (src/assets/thorsoft-logo.svg); the other
// steps are interpolated around them. Light only: main.ts turns PrimeVue's dark mode off.
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

/** Aura with the brand scale as `primary`. Aura's light surfaces (white, slate grey) stay. */
export const ThorSoftPreset = definePreset(Aura, { semantic: { primary: brand } });
