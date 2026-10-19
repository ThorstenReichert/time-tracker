<script setup lang="ts">
// Picks the day a view shows: previous/next day, any date, and back to today (R10, R14).
// It starts on today, as the backend's OS time zone sees it.
import { computed, onMounted, onUnmounted, ref } from "vue";
import Button from "primevue/button";
import DatePicker from "primevue/datepicker";
import { commands } from "../bindings";
import { call, errorMessage } from "../lib/api";
import { addDays, formatDay, fromPickerDate, toPickerDate } from "../lib/day";

/** YYYY-MM-DD; `null` until today is known. */
const day = defineModel<string | null>({ required: true });

const today = ref<string | null>(null);
const error = ref<string | null>(null);

async function goToday() {
  try {
    today.value = await call(commands.getToday());
    day.value = today.value;
    error.value = null;
  } catch (e) {
    error.value = errorMessage(e);
  }
}

const step = (days: number) => {
  if (day.value !== null) day.value = addDays(day.value, days);
};

const pickerDate = computed({
  get: () => (day.value === null ? null : toPickerDate(day.value)),
  set: (date: Date | null) => {
    if (date !== null) day.value = fromPickerDate(date);
  },
});

// Closing the window only hides it, so a view can outlive the day. When the window comes
// back, today is asked again; a view that showed the old today moves on to the new one.
async function refreshToday() {
  try {
    const now = await call(commands.getToday());
    if (day.value === null || day.value === today.value) day.value = now;
    today.value = now;
    error.value = null;
  } catch (e) {
    error.value = errorMessage(e);
  }
}
const onVisible = () => {
  if (document.visibilityState === "visible") void refreshToday();
};
const onFocus = () => void refreshToday();

onMounted(() => {
  void goToday();
  window.addEventListener("focus", onFocus);
  document.addEventListener("visibilitychange", onVisible);
});
onUnmounted(() => {
  window.removeEventListener("focus", onFocus);
  document.removeEventListener("visibilitychange", onVisible);
});
</script>

<template>
  <div class="day-bar">
    <Button label="Previous day" severity="secondary" size="small" :disabled="day === null" @click="step(-1)" />
    <DatePicker v-model="pickerDate" date-format="yy-mm-dd" :manual-input="false" input-class="day-input" aria-label="Date" />
    <Button label="Next day" severity="secondary" size="small" :disabled="day === null" @click="step(1)" />
    <Button label="Today" severity="secondary" size="small" :disabled="day !== null && day === today" @click="goToday" />
    <span v-if="day !== null" class="day-name">{{ formatDay(day) }}</span>
  </div>
  <p v-if="error !== null" class="error">{{ error }}</p>
</template>

<style scoped>
.day-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
}
.day-bar :deep(.day-input) {
  width: 7.5rem;
}
.day-name {
  margin-left: 0.5rem;
  font-weight: 600;
}
.error {
  color: var(--p-red-500);
}
</style>
