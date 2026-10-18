<script setup lang="ts">
// Start or switch from the recents or any combo (R5, R7, R3). The day's entries come in step 12.
import { ref } from "vue";
import Button from "primevue/button";
import ComboPicker from "../components/ComboPicker.vue";
import RecentsRow from "../components/RecentsRow.vue";
import { useTimerStore } from "../stores/timer";
import { useAttempt } from "../lib/attempt";
import { taskChoice, type ComboChoice } from "../lib/combo";

const timer = useTimerStore();
const attempt = useAttempt();
const choice = ref<ComboChoice | null>(null);
// Remounts the picker after a start, which clears its inputs.
const pickerKey = ref(0);

async function start() {
  const picked = choice.value;
  if (picked === null) return;
  if (await attempt(() => timer.start(picked.project.project, taskChoice(picked.task)))) {
    choice.value = null;
    pickerKey.value++;
  }
}
</script>

<template>
  <div class="view">
    <h2>Today</h2>
    <RecentsRow />
    <section>
      <h3>Start a timer</h3>
      <form class="start" @submit.prevent="start">
        <ComboPicker :key="pickerKey" v-model="choice" />
        <Button type="submit" label="Start" :disabled="choice === null" />
      </form>
    </section>
  </div>
</template>

<style scoped>
.start {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 0.5rem;
  align-items: start;
}
</style>
