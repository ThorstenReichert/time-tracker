<script setup lang="ts">
// Creates or edits a client: its name and rounding rule (R1, R17). The units and the default
// rule come from the backend, so the UI keeps no copy of the core's rules.
import { computed, ref, useId, watch } from "vue";
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import Select from "primevue/select";
import SelectButton from "primevue/selectbutton";
import type { Direction, Level, RuleDto } from "../bindings";
import { errorMessage } from "../lib/api";
import { useCatalogStore } from "../stores/catalog";

const visible = defineModel<boolean>("visible", { required: true });
const props = defineProps<{
  header: string;
  /** The client to edit; a new client starts with the default rule. */
  initial?: { name: string; rule: RuleDto };
  save: (name: string, rule: RuleDto) => Promise<void>;
}>();

const { catalog } = storeToRefs(useCatalogStore());
const units = computed(() => (catalog.value?.roundingUnits ?? []).map((u) => ({ label: `${u} min`, value: u })));
const directions: { label: string; value: Direction }[] = [
  { label: "Up", value: "up" },
  { label: "Nearest", value: "nearest" },
  { label: "Down", value: "down" },
];
const levels: { label: string; value: Level }[] = [
  { label: "Per project", value: "project" },
  { label: "Per client", value: "client" },
];

const ids = { name: useId(), unit: useId() };
// The fields are filled when the dialog opens.
const name = ref("");
const unit = ref(0);
const direction = ref<Direction>("nearest");
const level = ref<Level>("project");
const error = ref<string | null>(null);
const busy = ref(false);

watch(visible, (open) => {
  if (!open) return;
  const rule = props.initial?.rule ?? catalog.value?.defaultRule;
  name.value = props.initial?.name ?? "";
  if (rule) {
    unit.value = rule.unit;
    direction.value = rule.direction;
    level.value = rule.level;
  }
  error.value = null;
});

async function submit() {
  busy.value = true;
  try {
    await props.save(name.value, { unit: unit.value, direction: direction.value, level: level.value });
    visible.value = false;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal :header="header" :style="{ width: '30rem' }">
    <form class="dialog-form" @submit.prevent="submit">
      <label :for="ids.name">Name</label>
      <InputText :id="ids.name" v-model="name" autofocus fluid />
      <label :for="ids.unit">Round to</label>
      <Select :input-id="ids.unit" v-model="unit" :options="units" option-label="label" option-value="value" fluid />
      <span>Direction</span>
      <SelectButton v-model="direction" :options="directions" option-label="label" option-value="value" :allow-empty="false" />
      <span>Round</span>
      <SelectButton v-model="level" :options="levels" option-label="label" option-value="value" :allow-empty="false" />
      <Message v-if="error" severity="error">{{ error }}</Message>
      <div class="dialog-actions">
        <Button type="button" label="Cancel" severity="secondary" text @click="visible = false" />
        <Button type="submit" label="Save" :loading="busy" />
      </div>
    </form>
  </Dialog>
</template>
