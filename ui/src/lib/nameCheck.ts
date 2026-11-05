// The live name check of the New project (R2c) and New client (R2i) dialogs. The typed name
// and abbreviation are checked by tt-core while typing: a match with one archived item turns
// the dialog's button into "Unarchive …"; a clash blocks it.
// An answer only counts for exactly the text it was asked for. While the answer to the
// current text is pending, the dialog saves, and the core still checks the name. Opening or
// closing the dialog drops every answer still on its way (the dialogs are reused).
import { computed, ref, watch, type Ref } from "vue";
import type { CheckedItemDto, NameCheckDto } from "../bindings";

export type NameChecker = (name: string, abbreviation: string | null) => Promise<NameCheckDto>;

/**
 * @param check the dialog's `check` prop, read on every change (none: no check, e.g. Edit).
 * @param canUnarchive whether the dialog can unarchive (its `unarchive` prop is set).
 */
export function useNameCheck(
  visible: Ref<boolean>,
  name: Ref<string>,
  abbreviation: Ref<string>,
  check: () => NameChecker | undefined,
  canUnarchive: () => boolean,
) {
  /** The newest answer and the text it was for. */
  const answer = ref<{ name: string; abbreviation: string; result: NameCheckDto } | null>(null);
  let latest = 0;

  watch(visible, () => {
    latest++;
    answer.value = null;
  });

  watch([name, abbreviation], async ([typedName, typedAbbreviation]) => {
    const checker = check();
    if (checker === undefined || !visible.value) return;
    const request = ++latest;
    let result: NameCheckDto | null;
    try {
      result = await checker(typedName, typedAbbreviation);
    } catch {
      result = null; // the save shows the real error
    }
    if (request !== latest) return;
    answer.value = result === null ? null : { name: typedName, abbreviation: typedAbbreviation, result };
  });

  /** The check of exactly the text in the fields, or null while it is pending. */
  const current = computed(() => {
    const a = answer.value;
    if (check() === undefined || a === null) return null;
    return a.name === name.value && a.abbreviation === abbreviation.value ? a.result : null;
  });
  /** The archived item to unarchive instead of creating, if any. */
  const toUnarchive = computed<CheckedItemDto | null>(() =>
    current.value?.kind === "unarchive" && canUnarchive() ? (current.value.items[0] ?? null) : null,
  );
  /** The items the typed text clashes with; creating is blocked while there are any. */
  const taken = computed<CheckedItemDto[]>(() => (current.value?.kind === "taken" ? current.value.items : []));

  return { toUnarchive, taken };
}
