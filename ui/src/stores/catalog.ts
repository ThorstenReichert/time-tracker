import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { commands, type CatalogDto, type ClientDto, type ProjectDto, type StatusDto, type TaskDto } from "../bindings";
import { call, errorMessage } from "../lib/api";

export interface ProjectNode extends ProjectDto {
  tasks: TaskDto[];
}

export interface ClientNode extends ClientDto {
  projects: ProjectNode[];
}

const byName = (a: { name: string }, b: { name: string }) =>
  a.name.localeCompare(b.name, undefined, { sensitivity: "base" });

/** The catalog and the app status, reloaded on `data-changed`. */
export const useCatalogStore = defineStore("catalog", () => {
  const catalog = ref<CatalogDto | null>(null);
  const status = ref<StatusDto | null>(null);
  /** Why the last reload failed, if it did. */
  const loadError = ref<string | null>(null);

  async function refresh() {
    try {
      status.value = await call(commands.getStatus());
      catalog.value = await call(commands.getCatalog());
      loadError.value = null;
    } catch (e) {
      loadError.value = errorMessage(e);
    }
  }

  /** Clients → projects → tasks, each level sorted by name. */
  const tree = computed<ClientNode[]>(() => {
    const c = catalog.value;
    if (c === null) return [];
    return [...c.clients].sort(byName).map((client) => ({
      ...client,
      projects: c.projects
        .filter((p) => p.client === client.id)
        .sort(byName)
        .map((project) => ({
          ...project,
          tasks: c.tasks.filter((t) => t.project === project.id).sort(byName),
        })),
    }));
  });

  return { catalog, status, loadError, refresh, tree };
});
