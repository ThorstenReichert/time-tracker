import { beforeEach, describe, expect, it } from "vitest";
import { useTimerStore } from "../stores/timer";
import { fakeBackend } from "../test/backend";
import { acmeWebReview, betaOps } from "../test/fixtures";
import { mountInApp } from "../test/mount";
import { clickButton, fakeSearchDelay, pickSuggestion, typeAhead } from "../test/page";
import TodayView from "./TodayView.vue";

describe("TodayView", () => {
  beforeEach(fakeSearchDelay);

  it("starts the picked combo and clears the picker (R5)", async () => {
    const backend = fakeBackend({
      quick_switch: () => [acmeWebReview, betaOps],
      task_matches: () => ({ tasks: [], offerCreate: null }),
      start: () => null,
    });
    const wrapper = mountInApp(TodayView);
    useTimerStore().recents = [betaOps];
    const startButton = () => wrapper.findAll("button").find((b) => b.text() === "Start")!;
    expect(startButton().attributes("disabled")).toBeDefined();

    await typeAhead(wrapper.get(".start input"), "rev");
    await pickSuggestion("Acme / Web / Review");
    expect(startButton().attributes("disabled")).toBeUndefined();
    await clickButton("Start");

    expect(backend.called("start")).toEqual([{ project: 1, task: { kind: "existing", value: 1 } }]);
    expect((wrapper.get(".start input").element as HTMLInputElement).value).toBe("");
    expect(startButton().attributes("disabled")).toBeDefined();
  });
});
