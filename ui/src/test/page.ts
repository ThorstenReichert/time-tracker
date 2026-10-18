import { vi } from "vitest";
import { DOMWrapper, flushPromises } from "@vue/test-utils";

/** The open dialog, which PrimeVue renders into the body rather than inside the component. */
export function dialog(): DOMWrapper<Element> {
  const el = document.querySelector("[role=dialog]");
  if (el === null) throw new Error("no dialog is open");
  return new DOMWrapper(el);
}

/** Whether a dialog is open. */
export function dialogOpen(): boolean {
  return document.querySelector("[role=dialog]") !== null;
}

/** Clicks the button with this label inside `within` (the page by default). */
export async function clickButton(
  label: string,
  within: Pick<DOMWrapper<Element>, "findAll"> = new DOMWrapper(document.body),
) {
  const button = within.findAll("button").find((b) => b.text() === label || b.attributes("aria-label") === label);
  if (button === undefined) throw new Error(`no button "${label}"`);
  await button.trigger("click");
  await flushPromises();
}

/**
 * Fakes `setTimeout`, so `typeAhead` can skip AutoComplete's search delay. `setImmediate`, which
 * flushPromises uses, and the clock stay real.
 */
export function fakeSearchDelay() {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
}

/** Types into a PrimeVue AutoComplete and lets its 300 ms search delay pass. */
export async function typeAhead(input: Pick<DOMWrapper<Element>, "setValue">, text: string) {
  await input.setValue(text);
  await vi.advanceTimersByTimeAsync(300);
  await flushPromises();
}

/** The labels of the AutoComplete suggestions on screen. */
export function suggestions(): string[] {
  return [...document.querySelectorAll(".p-autocomplete-option")].map((o) => o.textContent?.trim() ?? "");
}

/** Clicks the suggestion with this label. */
export async function pickSuggestion(label: string) {
  const option = [...document.querySelectorAll(".p-autocomplete-option")].find((o) => o.textContent?.trim() === label);
  if (option === undefined) throw new Error(`no suggestion "${label}" in ${JSON.stringify(suggestions())}`);
  await new DOMWrapper(option).trigger("click");
  await flushPromises();
}
