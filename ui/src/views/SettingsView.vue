<script setup lang="ts">
// Settings: the hotkey (R6), recents (R7), the decimal separator (R15) and the backups (R22) are
// saved together; the bridge registers a new hotkey at once. Autostart (R9) is applied at once,
// because Windows' entry is its only record. The limits are checked in the core. About shows the
// brand and the version (R25).
import { computed, onMounted, ref, useId, watch } from "vue";
import { storeToRefs } from "pinia";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Message from "primevue/message";
import SelectButton from "primevue/selectbutton";
import ToggleSwitch from "primevue/toggleswitch";
import { useToast } from "primevue/usetoast";
import BrandLogo from "../components/BrandLogo.vue";
import { commands, type AutostartDto, type Separator, type SettingsDefaultsDto, type SettingsDto } from "../bindings";
import { call, errorMessage } from "../lib/api";
import { useAttempt } from "../lib/attempt";
import { decimalMark, formatHours } from "../lib/format";
import { useCatalogStore } from "../stores/catalog";

const { catalog, status } = storeToRefs(useCatalogStore());
const toast = useToast();
const version = __APP_VERSION__;
const attempt = useAttempt();

const sample = (setting: Separator) => formatHours(4500, decimalMark(setting));
const separators = computed(() => [
  { label: `Automatic (${sample("auto")})`, value: "auto" },
  { label: "Dot (1.25)", value: "dot" },
  { label: "Comma (1,25)", value: "comma" },
]);

const ids = { hotkey: useId(), recents: useId(), backupDir: useId(), backupKeep: useId(), autostart: useId() };
const hotkey = ref("");
const separator = ref<Separator>("auto");
const recentsN = ref("");
const backupDir = ref("");
const backupKeep = ref("");
const error = ref<string | null>(null);
const busy = ref(false);

// The form follows the saved settings, but only when they change, so a reload for anything
// else (e.g. a timer switched from the popup) keeps what is being typed.
const saved = computed(() => catalog.value?.settings ?? null);
watch(
  () => JSON.stringify(saved.value),
  () => {
    const s = saved.value;
    if (s === null) return;
    hotkey.value = s.hotkey ?? "";
    separator.value = s.decimalSeparator;
    recentsN.value = String(s.recentsN);
    backupDir.value = s.backupDir ?? "";
    backupKeep.value = String(s.backupKeep);
  },
  { immediate: true },
);

const defaults = ref<SettingsDefaultsDto | null>(null);
const autostart = ref<AutostartDto | null>(null);
// Bumped on every load: after a refused change the value is the same as before, which alone
// wouldn't reset the switch the user flipped.
const autostartLoads = ref(0);
const loadError = ref<string | null>(null);

async function loadAutostart() {
  autostart.value = await call(commands.getAutostart());
  autostartLoads.value++;
}

onMounted(async () => {
  try {
    defaults.value = await call(commands.getSettingsDefaults());
    await loadAutostart();
  } catch (e) {
    loadError.value = errorMessage(e);
  }
});

/** A count as typed; `null` unless it's a whole number. */
function count(text: string): number | null {
  const n = Number(text.trim());
  return text.trim() !== "" && Number.isInteger(n) && n >= 0 ? n : null;
}

/** Blank means the default. */
const orNull = (text: string) => (text.trim() === "" ? null : text.trim());

async function save() {
  const recents = count(recentsN.value);
  const keep = count(backupKeep.value);
  if (recents === null || keep === null) {
    error.value = "Enter whole numbers for the recents and the backups to keep.";
    return;
  }
  const settings: SettingsDto = {
    hotkey: orNull(hotkey.value),
    decimalSeparator: separator.value,
    recentsN: recents,
    backupDir: orNull(backupDir.value),
    backupKeep: keep,
  };
  busy.value = true;
  try {
    await call(commands.updateSettings(settings));
    error.value = null;
    toast.add({ severity: "success", summary: "Settings saved", life: 3000 });
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}

async function setAutostart(enabled: boolean) {
  await attempt(() => call(commands.setAutostart(enabled)));
  // Show what Windows has now, whether or not the change went through.
  await loadAutostart().catch((e) => (loadError.value = errorMessage(e)));
}
</script>

<template>
  <div class="view">
    <h2>Settings</h2>
    <Message v-if="loadError !== null" severity="error">{{ loadError }}</Message>

    <form class="settings" @submit.prevent="save">
      <h3>Quick switch</h3>
      <label :for="ids.hotkey">Hotkey</label>
      <div>
        <InputText :id="ids.hotkey" v-model="hotkey" :placeholder="defaults?.hotkey" fluid />
        <small class="muted">For example Ctrl+Alt+Space or Ctrl+Shift+K. Empty: {{ defaults?.hotkey }}.</small>
      </div>
      <label :for="ids.recents">Recent combos</label>
      <InputText :id="ids.recents" v-model="recentsN" type="number" min="1" class="count" />

      <h3>Hours</h3>
      <span>Decimal separator</span>
      <SelectButton
        v-model="separator"
        :options="separators"
        option-label="label"
        option-value="value"
        :allow-empty="false"
      />

      <h3>Backups</h3>
      <label :for="ids.backupDir">Folder</label>
      <div>
        <InputText :id="ids.backupDir" v-model="backupDir" :placeholder="defaults?.backupDir" fluid />
        <small class="muted">
          May be a OneDrive folder. Empty: {{ defaults?.backupDir }}. Used from the next start. The database
          itself stays at {{ status?.dbPath }}.
        </small>
      </div>
      <label :for="ids.backupKeep">Backups to keep</label>
      <InputText :id="ids.backupKeep" v-model="backupKeep" type="number" min="1" class="count" />

      <Message v-if="error" severity="error" class="span">{{ error }}</Message>
      <div class="span actions">
        <Button type="submit" label="Save" :loading="busy" />
      </div>
    </form>

    <h3>Startup</h3>
    <div class="autostart">
      <ToggleSwitch
        :key="autostartLoads"
        :input-id="ids.autostart"
        :model-value="autostart?.enabled ?? false"
        :disabled="!autostart?.available"
        @update:model-value="setAutostart"
      />
      <label :for="ids.autostart">Start with Windows</label>
    </div>
    <small v-if="autostart !== null && !autostart.available" class="muted">
      Only the installed app changes this, so a development build never adds itself to Windows' startup.
    </small>

    <h3>About</h3>
    <div class="about">
      <BrandLogo :size="64" />
      <div>
        <div class="product">ThorSoft TimeTracker</div>
        <div class="muted">Version {{ version }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings {
  display: grid;
  grid-template-columns: 11rem minmax(0, 32rem);
  gap: 0.75rem 1rem;
  align-items: baseline;
}
.settings h3,
.span {
  grid-column: 1 / -1;
}
.settings small {
  display: block;
  margin-top: 0.25rem;
}
.count {
  width: 6rem;
}
.actions {
  display: flex;
  justify-content: flex-end;
}
.about {
  display: flex;
  align-items: center;
  gap: 1rem;
}
.product {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--p-primary-800);
}
.autostart {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.25rem;
}
</style>
