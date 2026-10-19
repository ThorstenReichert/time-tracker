<script setup lang="ts">
// Asks for one name (create or rename a project or task). The core validates it; a rejection
// is shown here and the dialog stays open.
import { ref, useId, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import { errorMessage } from "../lib/api";

const visible = defineModel<boolean>("visible", { required: true });
const props = defineProps<{
  header: string;
  label: string;
  initial?: string;
  save: (name: string) => Promise<void>;
}>();

const id = useId();
const name = ref("");
const error = ref<string | null>(null);
const busy = ref(false);

watch(visible, (open) => {
  if (!open) return;
  name.value = props.initial ?? "";
  error.value = null;
});

async function submit() {
  busy.value = true;
  try {
    await props.save(name.value);
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
      <label :for="id">{{ label }}</label>
      <InputText :id="id" v-model="name" autofocus fluid />
      <Message v-if="error" severity="error">{{ error }}</Message>
      <div class="dialog-actions">
        <Button type="button" label="Cancel" severity="secondary" text @click="visible = false" />
        <Button type="submit" label="Save" :loading="busy" />
      </div>
    </form>
  </Dialog>
</template>
