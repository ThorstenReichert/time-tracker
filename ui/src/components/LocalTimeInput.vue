<script setup lang="ts">
// A local time typed as a date and a time of day (R10, R11). The time stays text: a JS Date
// would silently move a time in the DST gap, which the core has to reject instead (OQ-6).
import { computed } from "vue";
import DatePicker from "primevue/datepicker";
import InputText from "primevue/inputtext";
import { fromPickerDate, toPickerDate } from "../lib/day";

/** YYYY-MM-DD */
const date = defineModel<string>("date", { required: true });
/** As typed, e.g. "9:05" (see `parseTime`). */
const time = defineModel<string>("time", { required: true });
defineProps<{ label: string; timeId?: string; placeholder?: string; autofocus?: boolean }>();

const pickerDate = computed({
  get: () => toPickerDate(date.value),
  set: (picked: Date | null) => {
    if (picked !== null) date.value = fromPickerDate(picked);
  },
});
</script>

<template>
  <div class="local-time">
    <DatePicker
      v-model="pickerDate"
      date-format="yy-mm-dd"
      :manual-input="false"
      input-class="date-input"
      :aria-label="`${label} date`"
    />
    <InputText
      :id="timeId"
      v-model="time"
      class="time-input"
      :autofocus="autofocus"
      :placeholder="placeholder ?? 'HH:MM'"
      :aria-label="`${label} time`"
    />
  </div>
</template>

<style scoped>
.local-time {
  display: flex;
  gap: 0.5rem;
}
.local-time :deep(.date-input) {
  width: 7.5rem;
}
.time-input {
  width: 6rem;
}
</style>
