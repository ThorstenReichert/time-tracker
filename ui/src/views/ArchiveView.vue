<script setup lang="ts">
// The Archive view (R2e, R2f): archived projects grouped by client, each with its active
// period, in the order the backend gives (tt-core read::archived_projects). Each project can
// be unarchived after a confirmation.
import { onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import Button from "primevue/button";
import { useConfirm } from "primevue/useconfirm";
import { commands, type ArchiveClientDto, type ArchivedProjectDto } from "../bindings";
import { call, errorMessage } from "../lib/api";
import { useAttempt } from "../lib/attempt";
import { formatPeriod } from "../lib/day";

const attempt = useAttempt();
const confirm = useConfirm();

const clients = ref<ArchiveClientDto[] | null>(null);
const loadError = ref<string | null>(null);

async function load() {
  try {
    clients.value = await call(commands.getArchive());
    loadError.value = null;
  } catch (e) {
    loadError.value = errorMessage(e);
  }
}
void load();

const unlisten = listen("data-changed", () => void load());
onUnmounted(() => void unlisten.then((stop) => stop()));

/** The active period, or "No entries" for a project without any. */
function period(project: ArchivedProjectDto): string {
  return project.first === null || project.last === null ? "No entries" : formatPeriod(project.first, project.last);
}

/** Unarchives after a confirmation; a rejection is a toast (R2f). */
function unarchive(client: ArchiveClientDto, project: ArchivedProjectDto) {
  confirm.require({
    header: "Unarchive the project?",
    message: `${client.name} / ${project.name} can be booked again and is offered again.`,
    rejectProps: { label: "Cancel", severity: "secondary", text: true },
    acceptProps: { label: "Unarchive" },
    accept: () => void attempt(() => call(commands.unarchiveProject(project.id))),
  });
}
</script>

<template>
  <div class="view">
    <h2>Archive</h2>
    <p v-if="loadError !== null" class="error">{{ loadError }}</p>
    <p v-if="clients !== null && clients.length === 0" class="muted">No archived projects.</p>

    <section v-for="client in clients ?? []" :key="client.id" class="client">
      <div class="row">
        <strong class="client-name">{{ client.name }}</strong>
        <span v-if="client.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
          client.abbreviation
        }}</span>
      </div>
      <div v-for="project in client.projects" :key="project.id" class="project indent">
        <div class="row">
          <span class="project-name">{{ project.name }}</span>
          <span v-if="project.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
            project.abbreviation
          }}</span>
          <span class="period muted">{{ period(project) }}</span>
          <span class="spacer"></span>
          <Button label="Unarchive" text size="small" @click="unarchive(client, project)" />
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.client {
  border: 1px solid var(--p-content-border-color);
  border-radius: var(--p-content-border-radius, 6px);
  padding: 0.5rem 0.75rem;
  margin-bottom: 0.75rem;
}
.row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.spacer {
  flex: 1;
}
.client-name {
  font-size: 1.1rem;
}
.abbreviation {
  font-size: 0.85rem;
  padding: 0 0.4rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 4px;
  color: var(--p-text-muted-color);
}
.indent {
  margin-left: 1.25rem;
}
.project {
  border-top: 1px solid var(--p-content-border-color);
  padding: 0.25rem 0;
}
</style>
