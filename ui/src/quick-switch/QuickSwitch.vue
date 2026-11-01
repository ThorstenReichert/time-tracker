<script setup lang="ts">
// The quick-switch popup (R6): the running entry on top, then the combos (recents first, then
// the other active combos A–Z; typing filters them). Enter starts or switches and hides, Tab
// picks a task of the highlighted project (R3), Ctrl+S stops, Esc hides. The matching is a
// tt-core read model; this page only renders it and handles the keys. The header carries the
// ThorSoft logo; the popup is light only (R25).
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { commands, type EntryDto } from "../bindings";
import BrandLogo from "../components/BrandLogo.vue";
import { call, errorMessage } from "../lib/api";
import { taskChoice } from "../lib/combo";
import { formatElapsed } from "../lib/format";
import { comboTargets, moveHighlight, taskTargets, type Target, type TaskStep } from "../lib/quickSwitch";

const input = ref<HTMLInputElement | null>(null);
const list = ref<HTMLElement | null>(null);
const query = ref("");
/** Set while picking a task of a project (after Tab). */
const step = ref<TaskStep | null>(null);
/** The combo query to return to from the task step. */
let comboQuery = "";
const targets = ref<Target[]>([]);
const highlight = ref(0);
const running = ref<EntryDto | null>(null);
const error = ref<string | null>(null);

// The elapsed time is computed here from the start time (spec §2 "Elapsed time display").
const unixNow = () => Math.floor(Date.now() / 1000);
const now = ref(unixNow());
setInterval(() => (now.value = unixNow()), 1000);
const elapsed = computed(() => (running.value === null ? 0 : now.value - running.value.start));

// A search runs on every keystroke. Enter and Tab wait for the latest one, so typing fast
// and pressing Enter never picks a row of an older query.
let latest = 0;
let pending: Promise<void> = Promise.resolve();

function search() {
  const request = ++latest;
  const text = query.value;
  const taskStep = step.value;
  const runningCombo = running.value?.combo ?? null;
  error.value = null;
  pending = (async () => {
    try {
      const rows =
        taskStep === null
          ? comboTargets(await call(commands.quickSwitch(text)), runningCombo, text)
          : taskTargets(taskStep, await call(commands.taskMatches(taskStep.project, text)), text, runningCombo);
      if (request !== latest) return;
      targets.value = rows;
      highlight.value = 0;
    } catch (e) {
      if (request === latest) error.value = errorMessage(e);
    }
  })();
}
watch([query, step], search, { flush: "sync" });

async function reload() {
  try {
    running.value = await call(commands.getRunning());
    now.value = unixNow();
  } catch (e) {
    error.value = errorMessage(e);
  }
  search();
}

/** Back to the start state, ready for the next hotkey press. */
function reset() {
  query.value = "";
  step.value = null;
  comboQuery = "";
  highlight.value = 0;
  error.value = null;
}

async function hide() {
  reset();
  await getCurrentWindow().hide();
}

/** Runs a start or stop; hides on success, shows the error otherwise. */
async function act(action: () => Promise<unknown>) {
  try {
    await action();
    await hide();
  } catch (e) {
    error.value = errorMessage(e);
  }
}

async function choose(index?: number) {
  await pending;
  const target = targets.value[index ?? highlight.value];
  if (target === undefined) return;
  await act(() => call(commands.start(target.project, taskChoice(target.task))));
}

async function stop() {
  if (running.value === null) {
    error.value = "No timer is running.";
    return;
  }
  await act(() => call(commands.stop(null)));
}

async function pickTask() {
  if (step.value !== null) return;
  await pending;
  const target = targets.value[highlight.value];
  if (target === undefined) return;
  comboQuery = query.value;
  step.value = { project: target.project, projectLabel: target.projectLabel, projectTitle: target.projectTitle };
  query.value = "";
}

function backToCombos() {
  if (step.value === null) return;
  step.value = null;
  query.value = comboQuery;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    highlight.value = moveHighlight(highlight.value, e.key === "ArrowDown" ? 1 : -1, targets.value.length);
  } else if (e.key === "Enter") {
    void choose();
  } else if (e.key === "Tab") {
    if (e.shiftKey) backToCombos();
    else void pickTask();
  } else if (e.key === "Escape") {
    void hide();
  } else if (e.key === "Backspace" && query.value === "" && step.value !== null) {
    backToCombos();
  } else if (e.ctrlKey && e.key.toLowerCase() === "s") {
    void stop();
  } else {
    return;
  }
  e.preventDefault();
}

watch(highlight, async (index) => {
  await nextTick();
  list.value?.children[index]?.scrollIntoView({ block: "nearest" });
});

const emptyText = computed(() => {
  if (step.value !== null) return "No matching task.";
  if (query.value.trim() !== "") return "No matching combo.";
  return "Nothing else to switch to yet. Add clients and projects in the main window.";
});

// The window gets focus from Rust; the input only gets it when the page asks (plan step 10).
onMounted(async () => {
  await listen("quick-switch-shown", () => {
    input.value?.focus();
    void reload();
  });
  await listen("data-changed", () => void reload());
  // Rust hides the popup when it loses focus; it opens fresh next time.
  window.addEventListener("blur", reset);
  input.value?.focus();
  void reload();
});
</script>

<template>
  <div class="popup">
    <header class="running" :class="{ active: running }">
      <BrandLogo :size="18" />
      <template v-if="running">
        <span class="dot" aria-hidden="true"></span>
        <span class="label" :title="running.combo.label">{{ running.combo.shortLabel }}</span>
        <span class="elapsed">{{ formatElapsed(elapsed) }}</span>
        <button type="button" tabindex="-1" @mousedown.prevent @click="stop">Stop <kbd>Ctrl+S</kbd></button>
      </template>
      <span v-else class="muted">No timer running</span>
    </header>
    <div class="search">
      <span v-if="step" class="chip" :title="step.projectTitle">{{ step.projectLabel }}</span>
      <input
        ref="input"
        v-model="query"
        :placeholder="step ? 'Task: type to find or create' : 'Switch to… (type to search)'"
        spellcheck="false"
        autocomplete="off"
        @keydown="onKeydown"
      />
    </div>
    <ul v-if="targets.length" ref="list" class="targets" role="listbox">
      <li
        v-for="(target, i) in targets"
        :key="`${target.project}:${target.label}`"
        role="option"
        :aria-selected="i === highlight"
        :class="{ highlighted: i === highlight, create: target.task?.kind === 'new' }"
        :title="target.title"
        @mousemove="highlight = i"
        @mousedown.prevent
        @click="choose(i)"
      >
        <span class="label">{{ target.label }}</span>
        <span v-if="target.hint" class="hint">{{ target.hint }}</span>
        <span v-if="target.running" class="tag">running</span>
      </li>
    </ul>
    <p v-else class="targets empty muted">{{ emptyText }}</p>
    <p v-if="error" class="error">{{ error }}</p>
    <footer class="muted">
      <template v-if="step">Enter start · Shift+Tab back · Esc close</template>
      <template v-else>↑↓ choose · Enter switch · Tab pick a task · Ctrl+S stop · Esc close</template>
    </footer>
  </div>
</template>

<style>
/* Light only (R25). The popup doesn't use PrimeVue; --primary mirrors brand[500] in lib/brand.ts. */
:root {
  --primary: #436088;
  --text: #334155;
  --muted: #64748b;
  --border: #cbd5e1;
  --surface: #ffffff;
  --danger: #dc2626;
  color-scheme: light;
}
html,
body,
#app {
  margin: 0;
  height: 100%;
}
body {
  font-family: "Segoe UI", system-ui, sans-serif;
  font-size: 14px;
  color: var(--text);
  background: var(--surface);
}
.popup {
  display: flex;
  flex-direction: column;
  height: 100%;
  box-sizing: border-box;
  border: 1px solid var(--border);
}
.muted {
  color: var(--muted);
}
.running {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  height: 2.5rem;
  flex: none;
  padding: 0 0.9rem;
  border-bottom: 1px solid var(--border);
}
.running.active {
  background: color-mix(in srgb, var(--primary) 12%, transparent);
}
.running .label {
  flex: 1;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.dot {
  width: 0.55rem;
  height: 0.55rem;
  border-radius: 50%;
  background: var(--primary);
}
.elapsed {
  font-variant-numeric: tabular-nums;
}
.running button {
  font: inherit;
  color: var(--danger);
  background: none;
  border: 1px solid currentColor;
  border-radius: 4px;
  padding: 0.1rem 0.5rem;
  cursor: pointer;
}
kbd {
  font: inherit;
  font-size: 0.8em;
  opacity: 0.75;
}
.search {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: none;
  padding: 0.6rem 0.9rem;
}
.chip {
  white-space: nowrap;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  background: color-mix(in srgb, var(--primary) 18%, transparent);
}
.search input {
  flex: 1;
  min-width: 0;
  font: inherit;
  font-size: 1.15rem;
  color: inherit;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.4rem 0.6rem;
  outline: none;
}
.search input:focus {
  border-color: var(--primary);
}
.targets {
  flex: 1;
  overflow-y: auto;
  margin: 0;
  padding: 0 0.5rem;
  list-style: none;
}
.targets.empty {
  padding: 0.5rem 1rem;
}
.targets li {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.35rem 0.5rem;
  border-radius: 4px;
  cursor: pointer;
}
.targets li.highlighted {
  background: color-mix(in srgb, var(--primary) 20%, transparent);
}
.targets li.create .label {
  font-style: italic;
}
.targets .label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tag {
  font-size: 0.8em;
  color: var(--primary);
}
.hint {
  font-size: 0.85em;
  color: var(--muted);
}
.error {
  flex: none;
  margin: 0;
  padding: 0.4rem 0.9rem;
  color: var(--danger);
}
footer {
  flex: none;
  padding: 0.4rem 0.9rem;
  font-size: 0.85em;
  border-top: 1px solid var(--border);
}
</style>
