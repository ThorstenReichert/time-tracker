<script setup lang="ts">
// The Archive view (R2e, R2f): archived projects grouped by client, each with its active
// period, in the order the backend gives (tt-core read::archived_projects). Each project can
// be unarchived with one click (no confirmation: archiving it again is as easy).
import { onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import Button from "primevue/button";
import { commands, type ArchiveClientDto, type ArchivedProjectDto } from "../bindings";
import { call, errorMessage } from "../lib/api";
import { useAttempt } from "../lib/attempt";
import { formatShortDay } from "../lib/day";

const attempt = useAttempt();

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

/** Unarchives at once, without a confirmation; a rejection is a toast (R2f). */
function unarchive(project: ArchivedProjectDto) {
  void attempt(() => call(commands.unarchiveProject(project.id)));
}
</script>

<template>
  <div class="view">
    <h2>Archive</h2>
    <p v-if="loadError !== null" class="error">{{ loadError }}</p>
    <p v-if="clients !== null && clients.length === 0" class="muted">No archived projects.</p>

    <!-- One grid for all clients (subgrids below), so the periods line up in columns: the
         first days start at one edge and the dashes share a centre. -->
    <div v-if="clients !== null && clients.length > 0" class="archive">
      <section v-for="client in clients" :key="client.id" class="client">
        <div class="row client-row">
          <strong class="client-name">{{ client.name }}</strong>
          <span v-if="client.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
            client.abbreviation
          }}</span>
        </div>
        <div v-for="project in client.projects" :key="project.id" class="project">
          <div class="row">
            <span class="project-name">{{ project.name }}</span>
            <span v-if="project.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
              project.abbreviation
            }}</span>
          </div>
          <span class="period muted">
            <template v-if="project.period === null">
              <span class="none">No entries</span>
            </template>
            <template v-else>
              <span class="first">{{ formatShortDay(project.period.first) }}</span>
              <!-- A single day shows once. -->
              <template v-if="project.period.last !== project.period.first">
                <span class="dash">–</span>
                <span class="last">{{ formatShortDay(project.period.last) }}</span>
              </template>
            </template>
          </span>
          <Button class="action" label="Unarchive" text size="small" @click="unarchive(project)" />
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.error {
  color: var(--error-text);
}
.archive {
  display: grid;
  grid-template-columns:
    [name] minmax(0, 1fr) [first] max-content [dash] 1.5rem [last] max-content
    [action] max-content [end];
  column-gap: 0.75rem;
  row-gap: 0.75rem;
}
.client,
.project,
.period {
  display: grid;
  grid-template-columns: subgrid;
}
.client {
  grid-column: name / end;
  row-gap: 0;
  border: 1px solid var(--p-content-border-color);
  border-radius: var(--p-content-border-radius, 6px);
  padding: 0.5rem 0.75rem;
}
.client-row {
  grid-column: name / end;
}
.project {
  grid-column: name / end;
  align-items: center;
  margin-left: 1.25rem;
  border-top: 1px solid var(--p-content-border-color);
  padding: 0.25rem 0;
}
.period {
  grid-column: first / action;
}
.first {
  grid-column: first;
}
.none {
  grid-column: first / action;
}
.dash {
  grid-column: dash;
  text-align: center;
}
.last {
  grid-column: last;
}
.action {
  grid-column: action;
}
.row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
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
</style>
