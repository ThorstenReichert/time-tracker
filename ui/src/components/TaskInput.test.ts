import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import type { TaskMatchesDto } from "../bindings";
import type { TaskPick } from "../lib/combo";
import { fakeBackend } from "../test/backend";
import { catalog } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { fakeSearchDelay, pickSuggestion, pressEnter, suggestions, typeAhead, typeKeys } from "../test/page";
import TaskInput from "./TaskInput.vue";

/** Matches like the core: tasks of the project containing the text; Create unless one is exact. */
function taskMatches({ project, text }: { project: number; text: string }): TaskMatchesDto {
  const tasks = catalog.tasks.filter((t) => t.project === project && t.name.toLowerCase().includes(text.toLowerCase()));
  const exact = tasks.some((t) => t.name.toLowerCase() === text.toLowerCase());
  return { tasks, offerCreate: text === "" || exact ? null : text };
}

function taskInput(project: number | null) {
  const backend = fakeBackend({ task_matches: taskMatches });
  const wrapper = mountInApp(TaskInput, {
    props: {
      modelValue: null as TaskPick | null,
      project,
      "onUpdate:modelValue": (pick: TaskPick | null) => wrapper.setProps({ modelValue: pick }),
    },
  });
  return { backend, wrapper, input: wrapper.get("input"), pick: () => wrapper.props("modelValue") };
}

describe("TaskInput", () => {
  beforeEach(fakeSearchDelay);

  it("is disabled until a project is picked", () => {
    const { input } = taskInput(null);
    expect(input.attributes("disabled")).toBeDefined();
  });

  it("matches the project's tasks as you type and picks one (R3)", async () => {
    const { backend, input, pick } = taskInput(1);
    await typeAhead(input, "rev");
    expect(backend.called("task_matches").at(-1)).toEqual({ project: 1, text: "rev" });
    expect(suggestions()).toEqual(["Review", 'Create "rev"']);
    await pickSuggestion("Review");
    expect(pick()).toEqual({ kind: "existing", id: 1, name: "Review" });
  });

  it("shows a task's abbreviation muted next to its name (R24)", async () => {
    fakeBackend({
      task_matches: (): TaskMatchesDto => ({
        tasks: [{ id: 1, project: 1, name: "Review", abbreviation: "Rev", archived: false }],
        offerCreate: null,
      }),
    });
    const wrapper = mountInApp(TaskInput, { props: { modelValue: null, project: 1 } });
    await typeAhead(wrapper.get("input"), "rev");
    const option = document.querySelector(".p-autocomplete-option");
    expect(option?.querySelector(".hint")?.textContent).toBe("Rev");
  });

  it("offers Create for a new name and picks it as a new task (R3)", async () => {
    const { input, pick } = taskInput(1);
    await typeAhead(input, "Deploy");
    expect(suggestions()).toEqual(['Create "Deploy"']);
    await pickSuggestion('Create "Deploy"');
    expect(pick()).toEqual({ kind: "new", name: "Deploy" });
  });

  it("picks the first offer when Enter comes before the search answered", async () => {
    const { backend, input, pick } = taskInput(1);
    await typeKeys(input, "rev");
    await pressEnter(input);
    expect(backend.called("task_matches").at(-1)).toEqual({ project: 1, text: "rev" });
    expect(pick()).toEqual({ kind: "existing", id: 1, name: "Review" });
  });

  it("still picks when AutoComplete's own search fires while Enter waits", async () => {
    const { backend, input, pick } = taskInput(1);
    const answers: (() => void)[] = [];
    backend.handlers.task_matches = (args) => new Promise((done) => answers.push(() => done(taskMatches(args))));
    await typeKeys(input, "rev");
    await vi.advanceTimersByTimeAsync(290);
    await pressEnter(input);
    await vi.advanceTimersByTimeAsync(20);
    expect(backend.called("task_matches")).toHaveLength(2);
    answers.forEach((answer) => answer());
    await flushPromises();
    expect(pick()).toEqual({ kind: "existing", id: 1, name: "Review" });
  });

  it("leaves Enter on an answered list to the highlighted option", async () => {
    const { input, pick } = taskInput(1);
    await typeAhead(input, "rev");
    await input.trigger("keydown", { key: "ArrowDown", code: "ArrowDown" });
    await pressEnter(input);
    expect(pick()).toEqual({ kind: "new", name: "rev" });
  });

  it("offers no Create when a task has the exact name, in any case", async () => {
    const { input } = taskInput(1);
    await typeAhead(input, "docs");
    expect(suggestions()).toEqual(["Docs"]);
  });

  it("drops the pick when you type again: only a picked task counts", async () => {
    const { input, pick } = taskInput(1);
    await typeAhead(input, "rev");
    await pickSuggestion("Review");
    await typeAhead(input, "Revi");
    expect(pick()).toBeNull();
  });
});
