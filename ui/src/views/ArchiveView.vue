<script setup lang="ts">
// The Archive view (R2e, R2f, R2j): archived clients and archived projects grouped by
// client, each project with its active period, in the order the backend gives (tt-core
// read::archived_projects). An archived client lists all its projects and is unarchived as a
// whole; its projects archived on their own are marked and stay archived. Otherwise each
// project is unarchived with one click (no confirmation: archiving it again is as easy).
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

/** Unarchives the client at once, its projects as they were (R2g, R2j). */
function unarchiveClient(client: ArchiveClientDto) {
  void attempt(() => call(commands.unarchiveClient(client.id)));
}
</script>

<template>
  <div class="view">
    <h2>Archive</h2>
    <p v-if="loadError !== null" class="error">{{ loadError }}</p>
    <p v-if="clients !== null && clients.length === 0" class="muted">Nothing archived.</p>

    <!-- One grid for all clients (subgrids below), so the periods line up in columns: the
         first days start at one edge and the dashes share a centre. -->
    <div v-if="clients !== null && clients.length > 0" class="archive">
      <section v-for="client in clients" :key="client.id" class="client">
        <div class="client-row">
          <div class="row">
            <strong class="client-name">{{ client.name }}</strong>
            <span v-if="client.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
              client.abbreviation
            }}</span>
            <span v-if="client.archived" class="tag">archived</span>
          </div>
          <Button
            v-if="client.archived"
            class="action"
            label="Unarchive client"
            :aria-label="`Unarchive client ${client.name}`"
            text
            size="small"
            @click="unarchiveClient(client)"
          />
        </div>
        <p v-if="client.projects.length === 0" class="no-projects muted">No projects</p>
        <div v-for="project in client.projects" :key="project.id" class="project">
          <div class="row">
            <span class="project-name">{{ project.name }}</span>
            <span v-if="project.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
              project.abbreviation
            }}</span>
            <!-- Under an archived client: archived on its own, so it stays archived (R2j). -->
            <span v-if="client.archived && project.archived" class="tag">archived separately</span>
          </div>
          <span class="period muted">
            <template v-if="project.period === null">
              <span class="none">No entries</span>
            </template>
            <template v-else>
              <!-- Always a full range, also for a single day (owner). -->
              <span class="first">{{ formatShortDay(project.period.first) }}</span>
              <span class="dash">–</span>
              <span class="last">{{ formatShortDay(project.period.last) }}</span>
            </template>
          </span>
          <Button
            v-if="!client.archived"
            class="action"
            label="Unarchive"
            :aria-label="`Unarchive project ${project.name}`"
            text
            size="small"
            @click="unarchive(project)"
          />
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
  display: grid;
  grid-template-columns: subgrid;
  align-items: center;
}
.client-row .row {
  grid-column: name / action;
}
.no-projects {
  grid-column: name / end;
  margin: 0.25rem 0 0 1.25rem;
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
.tag {
  font-size: 0.75rem;
  padding: 0 0.4rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
  color: var(--p-text-muted-color);
  white-space: nowrap;
}
.abbreviation {
  font-size: 0.85rem;
  padding: 0 0.4rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 4px;
  color: var(--p-text-muted-color);
}
</style>
