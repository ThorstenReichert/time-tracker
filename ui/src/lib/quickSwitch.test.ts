import { describe, expect, it } from "vitest";
import type { ComboDto } from "../bindings";
import { projectOnly } from "./combo";
import { comboTargets, moveHighlight, taskTargets } from "./quickSwitch";

const review: ComboDto = {
  project: 2,
  task: 3,
  client: 1,
  clientName: "Acme",
  projectName: "Web",
  taskName: "Review",
  label: "Acme / Web / Review",
  clientAbbreviation: null,
  projectAbbreviation: null,
  taskAbbreviation: null,
  shortLabel: "Acme / Web / Review",
};
const web = projectOnly(review);
const api: ComboDto = { ...web, project: 4, projectName: "API", label: "Acme / API", shortLabel: "Acme / API" };
const step = { project: 2, projectLabel: "Acme / Web", projectTitle: "Acme / Web" };

describe("comboTargets", () => {
  it("starts the combo's project and task", () => {
    expect(comboTargets([review, api], null, "")).toEqual([
      {
        project: 2,
        projectLabel: "Acme / Web",
        projectTitle: "Acme / Web",
        task: { kind: "existing", id: 3, name: "Review" },
        label: "Acme / Web / Review",
        title: "Acme / Web / Review",
        hint: null,
        running: false,
      },
      {
        project: 4,
        projectLabel: "Acme / API",
        projectTitle: "Acme / API",
        task: null,
        label: "Acme / API",
        title: "Acme / API",
        hint: null,
        running: false,
      },
    ]);
  });

  it("labels rows with the short label and keeps the full one as the title (R24)", () => {
    const short: ComboDto = {
      ...review,
      clientAbbreviation: "AC",
      taskAbbreviation: "Rev",
      shortLabel: "AC / Web / Rev",
    };
    const [target] = comboTargets([short], null, "rev");
    expect(target).toMatchObject({
      label: "AC / Web / Rev",
      title: "Acme / Web / Review",
      projectLabel: "AC / Web",
      projectTitle: "Acme / Web",
    });
  });

  it("leaves the running combo out while nothing is typed, so the previous one comes first", () => {
    expect(comboTargets([review, api], review, " ").map((t) => t.label)).toEqual(["Acme / API"]);
  });

  it("keeps a typed match on the running combo, marked", () => {
    const targets = comboTargets([review, web], review, "rev");
    expect(targets.map((t) => [t.label, t.running])).toEqual([
      ["Acme / Web / Review", true],
      ["Acme / Web", false],
    ]);
  });
});

describe("taskTargets", () => {
  const matches = {
    tasks: [{ id: 3, project: 2, name: "Review", abbreviation: "Rev", archived: false }],
    offerCreate: null,
  };

  it("offers No task first while nothing is typed", () => {
    expect(taskTargets(step, matches, "", null).map((t) => [t.label, t.task])).toEqual([
      ["No task", null],
      ["Review", { kind: "existing", id: 3, name: "Review" }],
    ]);
  });

  it("offers Create after the matches once something is typed (R3)", () => {
    const targets = taskTargets(step, { tasks: [], offerCreate: "Docs" }, "Docs", null);
    expect(targets).toEqual([
      {
        ...step,
        task: { kind: "new", name: "Docs" },
        label: 'Create "Docs"',
        title: 'Create "Docs"',
        hint: null,
        running: false,
      },
    ]);
  });

  it("shows a task's abbreviation as the hint (R24)", () => {
    expect(taskTargets(step, matches, "", null).map((t) => t.hint)).toEqual([null, "Rev"]);
  });

  it("marks the running task, and No task when the project runs without one", () => {
    expect(taskTargets(step, matches, "", review).map((t) => t.running)).toEqual([false, true]);
    expect(taskTargets(step, matches, "", web).map((t) => t.running)).toEqual([true, false]);
  });
});

describe("moveHighlight", () => {
  it("stays within the list", () => {
    expect(moveHighlight(0, -1, 3)).toBe(0);
    expect(moveHighlight(1, 1, 3)).toBe(2);
    expect(moveHighlight(2, 1, 3)).toBe(2);
    expect(moveHighlight(0, 1, 0)).toBe(0);
  });
});
