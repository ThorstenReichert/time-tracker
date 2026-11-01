<script setup lang="ts">
// The last N distinct combos; one click starts or switches (R7).
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import type { ComboDto } from "../bindings";
import { useTimerStore } from "../stores/timer";
import { useAttempt } from "../lib/attempt";
import { fromCombo, sameCombo, taskChoice } from "../lib/combo";

const timer = useTimerStore();
const { recents, running } = storeToRefs(timer);
const attempt = useAttempt();

const isRunning = (combo: ComboDto) => running.value !== null && sameCombo(running.value.combo, combo);

function startCombo(combo: ComboDto) {
  void attempt(() => timer.start(combo.project, taskChoice(fromCombo(combo).task)));
}
</script>

<template>
  <section>
    <h3>Recent</h3>
    <div v-if="recents.length" class="recents">
      <Button
        v-for="combo in recents"
        :key="`${combo.project}-${combo.task}`"
        :label="combo.shortLabel"
        :severity="isRunning(combo) ? undefined : 'secondary'"
        :outlined="!isRunning(combo)"
        :title="(isRunning(combo) ? 'Running: ' : 'Start ') + combo.label"
        size="small"
        @click="startCombo(combo)"
      />
    </div>
    <p v-else class="muted">Combos you track appear here for one-click switching.</p>
  </section>
</template>

<style scoped>
.recents {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}
</style>
