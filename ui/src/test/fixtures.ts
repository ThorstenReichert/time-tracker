import type { CatalogDto, ComboDto, EntryDto, OverviewDto, StatusDto } from "../bindings";

function combo(project: number, task: number | null, client: number, names: [string, string, string?]): ComboDto {
  const [clientName, projectName, taskName = null] = names;
  const label = [clientName, projectName, taskName].filter((n) => n !== null).join(" / ");
  return { project, task, client, clientName, projectName, taskName, label };
}

// The demo catalog: Acme (Web with Review and Docs, Intern) and Beta (API, Ops).
export const acmeWebReview = combo(1, 1, 1, ["Acme", "Web", "Review"]);
export const acmeWebDocs = combo(1, 2, 1, ["Acme", "Web", "Docs"]);
export const acmeWeb = combo(1, null, 1, ["Acme", "Web"]);
export const acmeIntern = combo(2, null, 1, ["Acme", "Intern"]);
export const betaApi = combo(3, null, 2, ["Beta", "API"]);
export const betaOps = combo(4, null, 2, ["Beta", "Ops"]);

/** The running entry on `combo`, started `seconds` ago. */
export function runningEntry(onCombo: ComboDto, seconds: number): EntryDto {
  const start = Math.floor(Date.now() / 1000) - seconds;
  return {
    id: 1,
    combo: onCombo,
    start,
    end: null,
    startLocal: "2026-10-18T09:00:00",
    endLocal: null,
    duration: seconds,
    running: true,
    note: null,
  };
}

/** A finished entry on `combo`, with its local times as the backend sends them. */
export function finishedEntry(
  id: number,
  onCombo: ComboDto,
  startLocal: string,
  endLocal: string,
  note: string | null = null,
): EntryDto {
  // The Unix seconds read the local times as UTC; the views only use their difference.
  const start = Date.parse(`${startLocal}Z`) / 1000;
  const end = Date.parse(`${endLocal}Z`) / 1000;
  return { id, combo: onCombo, start, end, startLocal, endLocal, duration: end - start, running: false, note };
}

export const catalog: CatalogDto = {
  clients: [
    { id: 1, name: "Acme", rule: { unit: 15, direction: "nearest", level: "project" }, archived: false },
    { id: 2, name: "Beta", rule: { unit: 30, direction: "up", level: "client" }, archived: false },
  ],
  projects: [
    { id: 1, client: 1, name: "Web", archived: false },
    { id: 2, client: 1, name: "Intern", archived: false },
    { id: 3, client: 2, name: "API", archived: false },
    { id: 4, client: 2, name: "Ops", archived: false },
  ],
  tasks: [
    { id: 1, project: 1, name: "Review", archived: false },
    { id: 2, project: 1, name: "Docs", archived: false },
  ],
  settings: { hotkey: null, decimalSeparator: "auto", recentsN: 8, backupDir: null, backupKeep: 14 },
  defaultRule: { unit: 15, direction: "nearest", level: "project" },
  roundingUnits: [6, 12, 15, 30, 60],
};

export const healthy: StatusDto = { broken: null, warnings: [], dbPath: "C:\\demo\\timetracker.db" };

/**
 * The plan's worked day (step 4) on `date`: Acme (15 nearest, per project) Web 09:00–10:22,
 * Beta (30 up, per client) API 10:22–11:07 and Ops 11:07–11:20.
 */
export function workedDay(date: string, running = false): OverviewDto {
  return {
    date,
    clients: [
      {
        client: 1,
        name: "Acme",
        rule: { unit: 15, direction: "nearest", level: "project" },
        total: { raw: 4920, rounded: 4500, diff: -420 },
        projects: [{ project: 1, name: "Web", raw: 4920, booked: { raw: 4920, rounded: 4500, diff: -420 }, running: false }],
        running: false,
      },
      {
        client: 2,
        name: "Beta",
        rule: { unit: 30, direction: "up", level: "client" },
        total: { raw: 3480, rounded: 3600, diff: 120 },
        projects: [
          { project: 3, name: "API", raw: 2700, booked: null, running: false },
          { project: 4, name: "Ops", raw: 780, booked: null, running },
        ],
        running,
      },
    ],
  };
}
