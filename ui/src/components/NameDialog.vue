<script setup lang="ts">
// Asks for a name and an optional abbreviation (create or edit a project or task, R1, R24).
// The core validates both; a rejection is shown here and the dialog stays open.
import { ref, useId, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import { errorMessage } from "../lib/api";
import { blankToNull } from "../lib/format";

export interface Naming {
  name: string;
  abbreviation: string | null;
}

const visible = defineModel<boolean>("visible", { required: true });
const props = defineProps<{
  header: string;
  label: string;
  initial?: Naming;
  /** Gets the abbreviation as `null` when its field is blank. */
  save: (name: string, abbreviation: string | null) => Promise<void>;
}>();

const ids = { name: useId(), abbreviation: useId() };
const name = ref("");
const abbreviation = ref("");
const error = ref<string | null>(null);
const busy = ref(false);

watch(visible, (open) => {
  if (!open) return;
  name.value = props.initial?.name ?? "";
  abbreviation.value = props.initial?.abbreviation ?? "";
  error.value = null;
});

async function submit() {
  busy.value = true;
  try {
    await props.save(name.value, blankToNull(abbreviation.value));
    visible.value = false;
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal :header="header" :style="{ width: '28rem' }">
    <form class="dialog-form" @submit.prevent="submit">
      <label :for="ids.name">{{ label }}</label>
      <InputText :id="ids.name" v-model="name" autofocus fluid />
      <label :for="ids.abbreviation">Abbreviation (optional)</label>
      <InputText :id="ids.abbreviation" v-model="abbreviation" fluid />
      <Message v-if="error" severity="error">{{ error }}</Message>
      <div class="dialog-actions">
        <Button type="button" label="Cancel" severity="secondary" text @click="visible = false" />
        <Button type="submit" label="Save" :loading="busy" />
      </div>
    </form>
  </Dialog>
</template>
