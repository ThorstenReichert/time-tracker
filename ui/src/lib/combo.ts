import type { ComboDto, TaskChoiceDto, TaskMatchesDto } from "../bindings";

/** A task as picked in a task input: an existing task, or a new one to create (R3). */
export type TaskPick = { kind: "existing"; id: number; name: string } | { kind: "new"; name: string };

/** A combo as picked in the UI: the project (as a project-only combo) and an optional task. */
export interface ComboChoice {
  project: ComboDto;
  task: TaskPick | null;
}

/** The combo without its task, labelled "Client / Project". */
export function projectOnly(combo: ComboDto): ComboDto {
  return {
    ...combo,
    task: null,
    taskName: null,
    label: `${combo.clientName} / ${combo.projectName}`,
  };
}

/** The choice for a combo picked from a list (recents, quick-switch matches). */
export function fromCombo(combo: ComboDto): ComboChoice {
  const task: TaskPick | null =
    combo.task === null ? null : { kind: "existing", id: combo.task, name: combo.taskName ?? "" };
  return { project: projectOnly(combo), task };
}

/** The tasks a task input offers: the matches, then Create "<text>" if offered (R3). */
export function taskPicks(matches: TaskMatchesDto): TaskPick[] {
  const picks: TaskPick[] = matches.tasks.map((t) => ({ kind: "existing", id: t.id, name: t.name }));
  if (matches.offerCreate !== null) picks.push({ kind: "new", name: matches.offerCreate });
  return picks;
}

/** The task part of a command. */
export function taskChoice(pick: TaskPick | null): TaskChoiceDto {
  if (pick === null) return { kind: "none" };
  return pick.kind === "existing" ? { kind: "existing", value: pick.id } : { kind: "new", value: pick.name };
}

/** Whether two combos name the same project and task. */
export function sameCombo(a: ComboDto, b: ComboDto): boolean {
  return a.project === b.project && a.task === b.task;
}
