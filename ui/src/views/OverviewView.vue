<script setup lang="ts">
// The daily billing overview (R14): per client, the values to book with raw and difference,
// in decimal hours (R15). The rounding happens in tt-core (R16, R18); this only displays it.
import { computed, onUnmounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { commands, type OverviewDto } from "../bindings";
import CopyValue from "../components/CopyValue.vue";
import DayBar from "../components/DayBar.vue";
import { call, errorMessage } from "../lib/api";
import { decimalMark, formatDiff, formatHours, formatRule } from "../lib/format";
import { useCatalogStore } from "../stores/catalog";

const catalog = useCatalogStore();
const mark = computed(() => decimalMark(catalog.catalog?.settings.decimalSeparator ?? "auto"));
const hours = (seconds: number) => formatHours(seconds, mark.value);
const diff = (seconds: number) => formatDiff(seconds, mark.value);

const day = ref<string | null>(null);
const overview = ref<OverviewDto | null>(null);
const loadError = ref<string | null>(null);

async function load() {
  const asked = day.value;
  if (asked === null) return;
  try {
    const result = await call(commands.getOverview(asked));
    // A slower answer for a day that's no longer shown is dropped.
    if (asked !== day.value) return;
    overview.value = result;
    loadError.value = null;
  } catch (e) {
    if (asked !== day.value) return;
    loadError.value = errorMessage(e);
  }
}

// Another day's figures are never shown under this day's date, not even when it fails.
watch(day, () => {
  overview.value = null;
  loadError.value = null;
  void load();
});

// A running entry counts up to now (R14), so its values are reloaded once a minute.
const running = computed(() => overview.value?.clients.some((c) => c.running) ?? false);
const timer = setInterval(() => {
  if (running.value) void load();
}, 60_000);

const unlisten = listen("data-changed", () => void load());
onUnmounted(() => {
  clearInterval(timer);
  void unlisten.then((stop) => stop());
});
</script>

<template>
  <div class="view">
    <h2>Daily overview</h2>
    <DayBar v-model="day" />

    <p v-if="loadError !== null" class="error">{{ loadError }}</p>
    <p v-if="overview !== null && overview.clients.length === 0" class="muted">No time tracked on this day.</p>

    <section v-for="block in overview?.clients ?? []" :key="block.client" class="client" :data-client="block.name">
      <header class="client-head">
        <h3>{{ block.name }}</h3>
        <span class="muted">{{ formatRule(block.rule) }}</span>
        <span v-if="block.running" class="running">running</span>
      </header>
      <table>
        <thead>
          <tr>
            <th class="name">Project</th>
            <th class="num">To book</th>
            <th class="num">Raw</th>
            <th class="num">Diff</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in block.projects" :key="row.project" :data-project="row.name">
            <td class="name">
              {{ row.name }}
              <span v-if="row.running" class="running">running</span>
            </td>
            <td class="num booked">
              <CopyValue v-if="row.booked !== null" :value="hours(row.booked.rounded)" />
            </td>
            <td class="num raw">{{ hours(row.raw) }}</td>
            <td class="num diff">{{ row.booked !== null ? diff(row.booked.diff) : "" }}</td>
          </tr>
        </tbody>
        <tfoot>
          <tr class="total">
            <th class="name">{{ block.rule.level === "client" ? "Client total" : "Subtotal" }}</th>
            <td class="num booked">
              <!-- At client level the total is the value to book; at project level, only a sum. -->
              <CopyValue v-if="block.rule.level === 'client'" :value="hours(block.total.rounded)" />
              <span v-else class="sum">{{ hours(block.total.rounded) }}</span>
            </td>
            <td class="num raw">{{ hours(block.total.raw) }}</td>
            <td class="num diff">{{ diff(block.total.diff) }}</td>
          </tr>
        </tfoot>
      </table>
    </section>
  </div>
</template>

<style scoped>
.error {
  color: var(--p-red-500);
}
.client {
  margin-bottom: 1.5rem;
}
.client-head {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
  margin-bottom: 0.25rem;
}
.client-head h3 {
  margin: 0;
}
.running {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--p-primary-color);
}
.running::before {
  content: "● ";
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  padding: 0.3rem 0.5rem;
  border-bottom: 1px solid var(--p-content-border-color);
}
thead th {
  font-weight: 600;
  color: var(--p-text-muted-color);
}
.name {
  text-align: left;
}
.num {
  text-align: right;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.booked {
  width: 9rem;
}
.raw,
.diff {
  width: 5rem;
}
.diff {
  color: var(--p-text-muted-color);
}
tfoot th,
tfoot td {
  border-bottom: none;
}
.sum {
  font-weight: 600;
  /* Lines up with the CopyValue numbers above it, which have the button to their right. */
  margin-right: 4.75rem;
}
</style>
