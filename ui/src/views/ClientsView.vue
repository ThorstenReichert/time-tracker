<script setup lang="ts">
// Clients with their rounding rule, their projects and tasks: create and rename (R1, R17).
// Archiving comes in slice 2 (R2).
import { ref } from "vue";
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import { commands, type RuleDto } from "../bindings";
import ClientDialog from "../components/ClientDialog.vue";
import NameDialog from "../components/NameDialog.vue";
import { call } from "../lib/api";
import { useCatalogStore, type ClientNode, type ProjectNode } from "../stores/catalog";

const { tree } = storeToRefs(useCatalogStore());

function ruleText(rule: RuleDto): string {
  return `${rule.unit} min · ${rule.direction} · per ${rule.level}`;
}

// One client dialog and one name dialog, configured by whatever opened them.
const clientDialog = ref({
  visible: false,
  header: "",
  initial: undefined as { name: string; rule: RuleDto } | undefined,
  save: async (_name: string, _rule: RuleDto) => {},
});
const nameDialog = ref({
  visible: false,
  header: "",
  label: "Name",
  initial: undefined as string | undefined,
  save: async (_name: string) => {},
});

function newClient() {
  clientDialog.value = {
    visible: true,
    header: "New client",
    initial: undefined,
    save: async (name, rule) => {
      await call(commands.createClient(name, rule));
    },
  };
}

function editClient(client: ClientNode) {
  clientDialog.value = {
    visible: true,
    header: `Edit client ${client.name}`,
    initial: { name: client.name, rule: client.rule },
    save: async (name, rule) => {
      await call(commands.updateClient(client.id, name, rule));
    },
  };
}

function askName(header: string, initial: string | undefined, save: (name: string) => Promise<unknown>) {
  nameDialog.value = {
    visible: true,
    header,
    label: "Name",
    initial,
    save: async (name) => {
      await save(name);
    },
  };
}

const newProject = (client: ClientNode) =>
  askName(`New project for ${client.name}`, undefined, (name) => call(commands.createProject(client.id, name)));
const renameProject = (project: ProjectNode) =>
  askName(`Rename project ${project.name}`, project.name, (name) => call(commands.renameProject(project.id, name)));
const newTask = (project: ProjectNode) =>
  askName(`New task for ${project.name}`, undefined, (name) => call(commands.createTask(project.id, name)));
const renameTask = (task: { id: number; name: string }) =>
  askName(`Rename task ${task.name}`, task.name, (name) => call(commands.renameTask(task.id, name)));
</script>

<template>
  <div class="view">
    <div class="heading">
      <h2>Clients &amp; projects</h2>
      <Button label="New client" @click="newClient" />
    </div>
    <p v-if="tree.length === 0" class="muted">No clients yet. Create one to start tracking.</p>

    <section v-for="client in tree" :key="client.id" class="client">
      <div class="row">
        <strong class="client-name">{{ client.name }}</strong>
        <span class="muted">{{ ruleText(client.rule) }}</span>
        <span class="spacer"></span>
        <Button label="Edit" text size="small" @click="editClient(client)" />
        <Button label="New project" text size="small" @click="newProject(client)" />
      </div>
      <p v-if="client.projects.length === 0" class="muted indent">No projects yet.</p>
      <div v-for="project in client.projects" :key="project.id" class="project indent">
        <div class="row">
          <span class="project-name">{{ project.name }}</span>
          <span class="spacer"></span>
          <Button label="Rename" text size="small" @click="renameProject(project)" />
          <Button label="New task" text size="small" @click="newTask(project)" />
        </div>
        <div v-if="project.tasks.length" class="tasks">
          <Button
            v-for="task in project.tasks"
            :key="task.id"
            :label="task.name"
            title="Rename"
            severity="secondary"
            outlined
            size="small"
            @click="renameTask(task)"
          />
        </div>
      </div>
    </section>

    <ClientDialog
      v-model:visible="clientDialog.visible"
      :header="clientDialog.header"
      :initial="clientDialog.initial"
      :save="clientDialog.save"
    />
    <NameDialog
      v-model:visible="nameDialog.visible"
      :header="nameDialog.header"
      :label="nameDialog.label"
      :initial="nameDialog.initial"
      :save="nameDialog.save"
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
