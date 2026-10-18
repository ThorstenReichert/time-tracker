import { beforeEach, describe, expect, it } from "vitest";
import type { ComboChoice } from "../lib/combo";
import { fakeBackend } from "../test/backend";
import { acmeIntern, acmeWeb, acmeWebReview, betaOps } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { fakeSearchDelay, pickSuggestion, suggestions, typeAhead } from "../test/page";
import ComboPicker from "./ComboPicker.vue";

const combos = [acmeWebReview, betaOps, acmeIntern, acmeWeb];

function comboPicker() {
  const backend = fakeBackend({
    quick_switch: ({ query }) => combos.filter((c) => c.label.toLowerCase().includes(query.toLowerCase())),
    task_matches: () => ({ tasks: [], offerCreate: null }),
  });
  const wrapper = mountInApp(ComboPicker, {
    props: {
      modelValue: null as ComboChoice | null,
      "onUpdate:modelValue": (choice: ComboChoice | null) => wrapper.setProps({ modelValue: choice }),
    },
  });
  const [project, task] = wrapper.findAll("input");
  const value = (input: typeof project) => (input!.element as HTMLInputElement).value;
  return { backend, wrapper, project: project!, task: task!, value, choice: () => wrapper.props("modelValue") };
}

describe("ComboPicker", () => {
  beforeEach(fakeSearchDelay);

  it("searches the combos with the quick-switch read model (R6)", async () => {
    const { backend, project } = comboPicker();
    await typeAhead(project, "acme");
    expect(backend.called("quick_switch").at(-1)).toEqual({ query: "acme" });
    expect(suggestions()).toEqual(["Acme / Web / Review", "Acme / Intern", "Acme / Web"]);
  });

  it("fills project and task from a picked task combo", async () => {
    const { project, task, value, choice } = comboPicker();
    await typeAhead(project, "rev");
    await pickSuggestion("Acme / Web / Review");
    expect(choice()).toEqual({
      project: { ...acmeWeb, label: "Acme / Web" },
      task: { kind: "existing", id: 1, name: "Review" },
    });
    expect(value(project)).toBe("Acme / Web");
    expect(value(task)).toBe("Review");
  });

  it("picks a project without a task, which enables the task input", async () => {
    const { project, task, choice } = comboPicker();
    expect(task.attributes("disabled")).toBeDefined();
    await typeAhead(project, "ops");
    await pickSuggestion("Beta / Ops");
    expect(choice()).toEqual({ project: betaOps, task: null });
    expect(task.attributes("disabled")).toBeUndefined();
  });

  it("drops the pick when you type again: only a picked combo counts", async () => {
    const { project, choice } = comboPicker();
    await typeAhead(project, "ops");
    await pickSuggestion("Beta / Ops");
    await typeAhead(project, "Beta / Op");
    expect(choice()).toBeNull();
  });
});
