import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { commands, type ComboDto, type EntryDto, type TaskChoiceDto } from "../bindings";
import { call } from "../lib/api";

const unixNow = () => Math.floor(Date.now() / 1000);

/** The running entry and the recents, reloaded on `data-changed`. */
export const useTimerStore = defineStore("timer", () => {
  const running = ref<EntryDto | null>(null);
  const recents = ref<ComboDto[]>([]);

  // The elapsed time is computed here from the start time; the backend isn't polled for it
  // (spec §2 "Elapsed time display").
  const now = ref(unixNow());
  setInterval(() => (now.value = unixNow()), 1000);
  const elapsed = computed(() => (running.value === null ? 0 : now.value - running.value.start));

  /** Errors are left to the caller: the catalog store reports a failed reload. */
  async function refresh() {
    running.value = await call(commands.getRunning());
    recents.value = await call(commands.getRecents());
    now.value = unixNow();
  }

  /** Starts the combo, or switches to it (R5). */
  async function start(project: number, task: TaskChoiceDto) {
    await call(commands.start(project, task));
  }

  /** Stops now, or at a typed local time ("YYYY-MM-DDTHH:MM:SS", the forgotten stop, R11). */
  async function stop(at: string | null = null) {
    await call(commands.stop(at));
  }

  return { running, recents, elapsed, refresh, start, stop };
});
