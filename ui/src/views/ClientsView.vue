<script setup lang="ts">
// Clients with their rounding rule, their projects and tasks: create, rename, abbreviate
// and archive clients and projects (R1, R2a, R2g, R17, R24). Archived clients and projects
// are hidden here and listed in the Archive view (R2e, R2j). New client and New project
// offer to unarchive a matching archived client or project (R2c, R2i).
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import { commands, type NameCheckDto, type RuleDto, type TaskDto } from "../bindings";
import ClientDialog from "../components/ClientDialog.vue";
import NameDialog, { type Naming } from "../components/NameDialog.vue";
import type { NameChecker } from "../lib/nameCheck";
import { call } from "../lib/api";
import { useAttempt } from "../lib/attempt";
import { formatRule } from "../lib/format";
import { useCatalogStore, type ClientNode, type ProjectNode } from "../stores/catalog";

const { tree } = storeToRefs(useCatalogStore());
const attempt = useAttempt();

/** The tree without archived clients and projects (the store keeps them for the other views). */
const clients = computed(() =>
  tree.value
    .filter((client) => !client.archived)
    .map((client) => ({
      ...client,
      projects: client.projects.filter((p) => !p.archived),
      hasArchived: client.projects.some((p) => p.archived),
    })),
);
const hasArchivedClients = computed(() => tree.value.some((client) => client.archived));

// One client dialog and one name dialog, configured by whatever opened them.
const clientDialog = ref({
  visible: false,
  header: "",
  initial: undefined as (Naming & { rule: RuleDto }) | undefined,
  save: async (_name: string, _abbreviation: string | null, _rule: RuleDto) => {},
  check: undefined as NameChecker | undefined,
  unarchive: undefined as ((client: number) => Promise<void>) | undefined,
});
const nameDialog = ref({
  visible: false,
  header: "",
  label: "Name",
  initial: undefined as Naming | undefined,
  save: async (_name: string, _abbreviation: string | null) => {},
  check: undefined as NameChecker | undefined,
  unarchive: undefined as ((project: number) => Promise<void>) | undefined,
});

function newClient() {
  clientDialog.value = {
    visible: true,
    header: "New client",
    initial: undefined,
    save: async (name, abbreviation, rule) => {
      await call(commands.createClient(name, abbreviation, rule));
    },
    check: (name, abbreviation) => call(commands.checkClientName(name, abbreviation)),
    unarchive: async (client) => {
      await call(commands.unarchiveClient(client));
    },
  };
}

function editClient(client: ClientNode) {
  clientDialog.value = {
    visible: true,
    header: `Edit client ${client.name}`,
    initial: { name: client.name, abbreviation: client.abbreviation, rule: client.rule },
    save: async (name, abbreviation, rule) => {
      await call(commands.updateClient(client.id, name, abbreviation, rule));
    },
    check: undefined,
    unarchive: undefined,
  };
}

/**
 * Archives at once, without a confirmation: the Archive view unarchives with one click. Its
 * projects keep their own archive state (R2g). A rejection (the timer runs on one of its
 * projects) is a toast.
 */
function archiveClient(client: ClientNode) {
  void attempt(() => call(commands.archiveClient(client.id)));
}

function askName(
  header: string,
  initial: Naming | undefined,
  save: (name: string, abbreviation: string | null) => Promise<unknown>,
  check?: NameChecker,
) {
  nameDialog.value = {
    visible: true,
    header,
    label: "Name",
    initial,
    save: async (name, abbreviation) => {
      await save(name, abbreviation);
    },
    check,
    unarchive:
      check === undefined
        ? undefined
        : async (project) => {
            await call(commands.unarchiveProject(project));
          },
  };
}

const newProject = (client: ClientNode) =>
  askName(
    `New project for ${client.name}`,
    undefined,
    (name, abbreviation) => call(commands.createProject(client.id, name, abbreviation)),
    (name, abbreviation) => call(commands.checkProjectName(client.id, name, abbreviation)),
  );

/**
 * Archives at once, without a confirmation: the Archive view unarchives with one click
 * (R2a as amended by R2f). A rejection (e.g. the timer runs on it) is a toast.
 */
function archiveProject(project: ProjectNode) {
  void attempt(() => call(commands.archiveProject(project.id)));
}
const editProject = (project: ProjectNode) =>
  askName(`Edit project ${project.name}`, project, (name, abbreviation) =>
    call(commands.updateProject(project.id, name, abbreviation)),
  );
const newTask = (project: ProjectNode) =>
  askName(`New task for ${project.name}`, undefined, (name, abbreviation) =>
    call(commands.createTask(project.id, name, abbreviation)),
  );
const editTask = (task: TaskDto) =>
  askName(`Edit task ${task.name}`, task, (name, abbreviation) =>
    call(commands.updateTask(task.id, name, abbreviation)),
  );

/** "Name (ABBR)" for a task button, or just the name. */
const withAbbreviation = (item: Naming) =>
  item.abbreviation === null ? item.name : `${item.name} (${item.abbreviation})`;
</script>

<template>
  <div class="view">
    <div class="heading">
      <h2>Clients &amp; projects</h2>
      <Button label="New client" @click="newClient" />
    </div>
    <p v-if="clients.length === 0" class="muted">
      {{ hasArchivedClients ? "No active clients." : "No clients yet. Create one to start tracking." }}
    </p>

    <section v-for="client in clients" :key="client.id" class="client">
      <div class="row client-row">
        <strong class="client-name">{{ client.name }}</strong>
        <span v-if="client.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
          client.abbreviation
        }}</span>
        <span class="muted">{{ formatRule(client.rule) }}</span>
        <span class="spacer"></span>
        <Button label="Edit" text size="small" @click="editClient(client)" />
        <Button label="New project" text size="small" @click="newProject(client)" />
        <Button label="Archive" text size="small" severity="secondary" @click="archiveClient(client)" />
      </div>
      <p v-if="client.projects.length === 0" class="muted indent">
        {{ client.hasArchived ? "No active projects." : "No projects yet." }}
      </p>
      <div v-for="project in client.projects" :key="project.id" class="project indent">
        <div class="row">
          <span class="project-name">{{ project.name }}</span>
          <span v-if="project.abbreviation !== null" class="abbreviation" title="Abbreviation">{{
            project.abbreviation
          }}</span>
          <span class="spacer"></span>
          <Button label="Edit" text size="small" @click="editProject(project)" />
          <Button label="New task" text size="small" @click="newTask(project)" />
          <Button label="Archive" text size="small" severity="secondary" @click="archiveProject(project)" />
        </div>
        <div v-if="project.tasks.length" class="tasks">
          <Button
            v-for="task in project.tasks"
            :key="task.id"
            :label="withAbbreviation(task)"
            title="Edit"
            severity="secondary"
            outlined
            size="small"
            @click="editTask(task)"
          />
        </div>
      </div>
    </section>

    <ClientDialog
      v-model:visible="clientDialog.visible"
      :header="clientDialog.header"
      :initial="clientDialog.initial"
      :save="clientDialog.save"
      :check="clientDialog.check"
      :unarchive="clientDialog.unarchive"
    />
    <NameDialog
      v-model:visible="nameDialog.visible"
      :header="nameDialog.header"
      :label="nameDialog.label"
      :initial="nameDialog.initial"
      :save="nameDialog.save"
      :check="nameDialog.check"
      :unarchive="nameDialog.unarchive"
    />
  </div>
</template>

<style scoped>
.heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
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
.tasks {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  padding: 0.25rem 0 0.5rem;
}
</style>
