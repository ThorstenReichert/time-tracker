import type { ComboDto, TaskMatchesDto } from "../bindings";
import { projectOnly, taskPicks, type TaskPick } from "./combo";

/** A row of the quick-switch popup: what Enter starts or switches to (R6). */
export interface Target {
  project: number;
  /** "Client / Project" with abbreviations, for the task step (R24). */
  projectLabel: string;
  /** The full "Client / Project", shown on hover. */
  projectTitle: string;
  task: TaskPick | null;
  /** The short label (R24). */
  label: string;
  /** The full label, shown on hover. */
  title: string;
  /** Shown muted after the label: a task's abbreviation in the task step. */
  hint: string | null;
  running: boolean;
}

/** The project the popup's task step picks a task for (Tab, R3). */
export interface TaskStep {
  project: number;
  projectLabel: string;
  projectTitle: string;
}

const isRunning = (running: ComboDto | null, project: number, task: number | null) =>
  running !== null && running.project === project && running.task === task;

/**
 * The combo rows. With nothing typed, the running combo is left out: it's shown above the
 * list, and leaving it out puts the previous combo first, so hotkey + Enter switches back.
 */
export function comboTargets(rows: ComboDto[], running: ComboDto | null, query: string): Target[] {
  const blank = query.trim() === "";
  return rows
    .map((combo) => {
      const project = projectOnly(combo);
      return {
        project: combo.project,
        projectLabel: project.shortLabel,
        projectTitle: project.label,
        task: combo.task === null ? null : { kind: "existing" as const, id: combo.task, name: combo.taskName ?? "" },
        label: combo.shortLabel,
        title: combo.label,
        hint: null,
        running: isRunning(running, combo.project, combo.task),
      };
    })
    .filter((t) => !(blank && t.running));
}

/** The task rows of the task step: "No task" first while nothing is typed, then the matches. */
export function taskTargets(step: TaskStep, matches: TaskMatchesDto, text: string, running: ComboDto | null): Target[] {
  const picks: (TaskPick | null)[] = taskPicks(matches);
  if (text.trim() === "") picks.unshift(null);
  const abbreviation = (id: number) => matches.tasks.find((t) => t.id === id)?.abbreviation ?? null;
  return picks.map((task) => {
    const label = task === null ? "No task" : task.kind === "new" ? `Create "${task.name}"` : task.name;
    return {
      ...step,
      task,
      label,
      title: label,
      hint: task?.kind === "existing" ? abbreviation(task.id) : null,
      running: isRunning(running, step.project, task?.kind === "existing" ? task.id : null),
    };
  });
}

/** Moves the highlight by `delta`, within the list. */
export function moveHighlight(index: number, delta: number, length: number): number {
  return length === 0 ? 0 : Math.min(length - 1, Math.max(0, index + delta));
}
