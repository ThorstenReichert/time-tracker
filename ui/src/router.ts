import { createRouter, createWebHashHistory } from "vue-router";
import TodayView from "./views/TodayView.vue";
import OverviewView from "./views/OverviewView.vue";
import ClientsView from "./views/ClientsView.vue";
import SettingsView from "./views/SettingsView.vue";

// Hash history: the page is served as a file by Tauri, so there is no server to rewrite paths.
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "today", component: TodayView, meta: { title: "Today" } },
    { path: "/overview", name: "overview", component: OverviewView, meta: { title: "Daily overview" } },
    { path: "/clients", name: "clients", component: ClientsView, meta: { title: "Clients & projects" } },
    { path: "/settings", name: "settings", component: SettingsView, meta: { title: "Settings" } },
  ],
});
