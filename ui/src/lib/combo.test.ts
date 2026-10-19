import { describe, expect, it } from "vitest";
import type { ComboDto } from "../bindings";
import { fromCombo, projectOnly, sameCombo, taskChoice } from "./combo";

const review: ComboDto = {
  project: 2,
  task: 3,
  client: 1,
  clientName: "Acme",
  projectName: "Web",
  taskName: "Review",
  label: "Acme / Web / Review",
};

describe("combo choices", () => {
  it("drops the task from a combo", () => {
    expect(projectOnly(review)).toEqual({ ...review, task: null, taskName: null, label: "Acme / Web" });
  });

  it("picks a combo with its task", () => {
    expect(fromCombo(review)).toEqual({
      project: projectOnly(review),
      task: { kind: "existing", id: 3, name: "Review" },
    });
  });

  it("picks a project-only combo without a task", () => {
    expect(fromCombo(projectOnly(review)).task).toBeNull();
  });

  it("turns picks into the command's task choice", () => {
    expect(taskChoice(null)).toEqual({ kind: "none" });
    expect(taskChoice({ kind: "existing", id: 3, name: "Review" })).toEqual({ kind: "existing", value: 3 });
    expect(taskChoice({ kind: "new", name: "Docs" })).toEqual({ kind: "new", value: "Docs" });
  });

  it("compares combos by project and task", () => {
    expect(sameCombo(review, { ...review, label: "renamed" })).toBe(true);
    expect(sameCombo(review, projectOnly(review))).toBe(false);
  });
});
