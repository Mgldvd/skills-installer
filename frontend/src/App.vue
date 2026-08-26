<template>
  <div class="app-shell">
    <main class="app-shell__main">
      <AppHeader
        :project-path="state.projectRoot"
        :scope="state.preferences.lastScope"
        @update:project-path="handleProjectPathUpdate"
        @update:scope="handleScopeUpdate"
      />

      <SkillToolbar
        :tags="state.tags"
        :skills="state.skills"
        :selected-ids="[...state.selectedSkillIds]"
        :needs-agents-count="skillsNeedingAgentsCount"
        @toggle-tag="toggleTagSelection"
        @reorder-tags="handleReorderTags"
        @clear-selection="clearSelection"
        @select-missing="handleSelectMissingAgents"
        @open-packs="isTagsOpen = !isTagsOpen"
      />

      <SkillFilterBar
        :query="skillQuery"
        :sort-by="skillSort"
        :view="displayMode"
        :source-filter="skillSourceFilter"
        :pack-filter="skillPackFilter"
        :tags="state.tags"
        :needs-agents-only="skillNeedsAgentsFilter"
        :needs-agents-count="skillsNeedingAgentsCount"
        @update:query="skillQuery = $event"
        @update:sort-by="skillSort = $event"
        @update:view="handleDisplayModeChange"
        @update:source-filter="skillSourceFilter = $event"
        @update:pack-filter="skillPackFilter = $event"
        @update:needs-agents-only="skillNeedsAgentsFilter = $event"
      />

      <SkillGrid
        :skills="displayedSkills"
        :tags="state.tags"
        :selected-ids="state.selectedSkillIds"
        :compact="state.preferences.compactCards"
        :installing-skill-id="installingSkillId"
        :skills-with-updates="state.skillsWithUpdates"
        :target-agents="state.preferences.defaultAgents"
        :agent-order="agentOrder"
        :view="skillView"
        :delete-mode="isDeleteMode"
        :delete-selected-ids="deleteSelectedIds"
        @toggle="toggleSelected"
        @edit="openEditDialog"
        @update="handleUpdateSkill"
        @toggle-delete="toggleDeleteSelected"
      />

      <InstallProgressPanel
        v-if="state.installation.isInstalling || state.installation.result || state.installation.error"
        :installation="state.installation"
        :skills="state.skills"
        @cancel="handleCancelInstall"
        @dismiss="dismissInstallPanel"
      />
    </main>

    <footer class="app-shell__footer">
      <div class="app-shell__footer-row">
        <div class="app-shell__footer-left">
          <button type="button" class="app-shell__footer-btn app-shell__footer-btn--add" @click="openAddDialog">
            Add Skill
          </button>
          <button type="button" class="app-shell__footer-btn" @click="isPresetsOpen = !isPresetsOpen">
            Presets
          </button>
          <button type="button" class="app-shell__footer-btn" @click="isPreferencesOpen = !isPreferencesOpen">
            Preferences
          </button>
          <button
            type="button"
            class="app-shell__footer-btn app-shell__footer-btn--icon"
            :class="{ 'is-loading': isCheckingForUpdates }"
            :disabled="isCheckingForUpdates"
            :aria-label="isCheckingForUpdates ? 'Checking for Skills updates…' : 'Check for Skills updates'"
            title="Check for Skills updates"
            @click="handleCheckForUpdates"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <polyline points="23 4 23 10 17 10" />
              <polyline points="1 20 1 14 7 14" />
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
            </svg>
          </button>
        </div>
        <div class="app-shell__footer-actions">
          <template v-if="isDeleteMode">
            <span class="app-shell__selected-count">{{ deleteSelectedIds.size }} selected to uninstall</span>
            <button type="button" class="app-shell__footer-btn" @click="deselectAllForDelete">Deselect All</button>
            <button type="button" class="app-shell__footer-btn" @click="selectAllForDelete">Select All</button>
            <button
              type="button"
              class="app-shell__footer-btn app-shell__footer-btn--danger"
              :disabled="deleteSelectedIds.size === 0 || isUninstalling"
              @click="confirmUninstall"
            >
              {{ isUninstalling ? "Uninstalling…" : "Confirm" }}
            </button>
          </template>
          <template v-else>
            <label
              class="app-shell__toggle"
              :title="
                state.preferences.confirmBeforeInstall
                  ? 'Install Selected will ask for confirmation first'
                  : 'Install Selected will run immediately, no confirmation'
              "
            >
              <input
                type="checkbox"
                class="app-shell__toggle-input"
                :checked="!state.preferences.confirmBeforeInstall"
                @change="toggleSkipConfirm"
              />
              <span class="app-shell__toggle-track" aria-hidden="true">
                <span class="app-shell__toggle-thumb"></span>
              </span>
              <span class="app-shell__toggle-label">Skip confirmation</span>
            </label>
            <span class="app-shell__selected-count">{{ selectedSkills.length }} selected</span>
            <button
              type="button"
              class="app-shell__footer-btn app-shell__footer-btn--primary"
              :disabled="selectedSkills.length === 0 || state.installation.isInstalling"
              @click="handleInstallClick"
            >
              Install Selected
            </button>
          </template>
          <button
            type="button"
            class="app-shell__footer-btn app-shell__footer-btn--icon app-shell__footer-btn--danger"
            :class="{ 'is-active': isDeleteMode }"
            :disabled="!isDeleteMode && !hasInstalledSkills"
            :aria-label="isDeleteMode ? 'Cancel bulk uninstall' : 'Uninstall Skills in bulk'"
            :title="isDeleteMode ? 'Cancel bulk uninstall' : 'Uninstall Skills in bulk'"
            @click="isDeleteMode ? exitDeleteMode() : enterDeleteMode()"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M4 7h16M9 7V4.5A1.5 1.5 0 0 1 10.5 3h3A1.5 1.5 0 0 1 15 4.5V7m2 0-.9 13a2 2 0 0 1-2 1.9H9.9a2 2 0 0 1-2-1.9L7 7h10ZM10.5 11v6M13.5 11v6"
              />
            </svg>
          </button>
        </div>
      </div>
      <div class="app-shell__footer-status">
        <span class="app-shell__dependency" :class="dependencyClass">
          <span class="app-shell__dependency-dot" aria-hidden="true" />
          Skills CLI {{ dependencyLabel }}
        </span>
        <button
          type="button"
          class="app-shell__agents-summary"
          :aria-label="`Open Agents settings. Currently selected: ${agentLabels}`"
          :title="agentLabels"
          @click="isAgentsOpen = true"
        >
          <span class="app-shell__manage-agents" aria-hidden="true">
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="M8 2v12M2 8h12" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" />
            </svg>
          </span>
          Agents
          <span v-if="state.preferences.defaultAgents.length" class="app-shell__agent-icons">
            <AgentIcon v-for="id in state.preferences.defaultAgents" :key="id" :agent-id="id" />
          </span>
          <strong v-else class="app-shell__agent-none">None selected</strong>
        </button>
      </div>
    </footer>

    <AddSkillDialog
      v-model:open="isAddDialogOpen"
      :default-group-id="defaultGroupId"
      :skills="state.skills"
      :tags="state.tags"
      :submit-error="addSkillError"
      :deleting="isDeletingCatalogSkills"
      :delete-error="catalogDeleteError"
      @submit="handleAddSkillSubmit"
      @import-pack="handleImportPack"
      @delete-skills="handleDeleteCatalogSkills"
      @edit="handleEditSkillFromCatalog"
      @update-skill="handleUpdateCatalogSkill"
    />

    <EditSkillDialog
      :open="isEditDialogOpen"
      :skill="skillBeingEdited"
      :tags="state.tags"
      :submit-error="editSkillError"
      :target-agents="state.preferences.defaultAgents"
      @update:open="handleEditDialogOpenChange"
      @submit="handleEditSkillSubmit"
      @delete="handleDeleteSkill"
    />

    <InstallConfirmDialog
      v-model:open="isInstallConfirmOpen"
      :skills="selectedSkills"
      :options="installOptionsFromPreferences"
      :project-path="state.projectRoot"
      @confirm="runInstall"
    />

    <PreferencesDialog
      v-model:open="isPreferencesOpen"
      :preferences="state.preferences"
      @update="handlePreferencesUpdate"
      @update-local-source="handleLocalSourceUpdate"
      @refresh-local-source="handleLocalRefresh"
      @export-config="handleExportConfig"
      @import-config="handleImportConfig"
      @install-cli="handleInstallCli"
      @open-packs="handleOpenPacksFromPreferences"
      @open-agents="handleOpenAgentsFromPreferences"
    />
    <AgentsDialog
      v-model:open="isAgentsOpen"
      :model-value="state.preferences.defaultAgents"
      :agent-order="state.preferences.agentOrder"
      :scope="state.preferences.lastScope"
      @update:model-value="handleAgentsUpdate"
      @update:agent-order="handleAgentOrderUpdate"
    />
    <TagsDialog
      v-model:open="isTagsOpen"
      :tags="state.tags"
      :skills="state.skills"
      :error="tagsError"
      :pending-keys="[...pendingTagKeys]"
      @assign="handleAssignTag"
      @unassign="handleUnassignTag"
      @create="handleCreateTag"
      @update="handleUpdateTag"
      @delete="handleDeleteTag"
    />
    <PresetsDialog
      v-model:open="isPresetsOpen"
      :presets="state.presets"
      :installed-skill-names="installedSkillNames"
      :error="presetsError"
      @save="handleSavePreset"
      @load="handleLoadPreset"
      @delete="handleDeletePreset"
    />

    <ToastHost />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";

import AddSkillDialog from "./components/AddSkillDialog/AddSkillDialog.vue";
import AgentIcon from "./components/AgentIcon/AgentIcon.vue";
import AgentsDialog from "./components/AgentsDialog/AgentsDialog.vue";
import AppHeader from "./components/AppHeader/AppHeader.vue";
import EditSkillDialog from "./components/EditSkillDialog/EditSkillDialog.vue";
import InstallConfirmDialog from "./components/InstallConfirmDialog/InstallConfirmDialog.vue";
import InstallProgressPanel from "./components/InstallProgressPanel/InstallProgressPanel.vue";
import TagsDialog from "./components/TagsDialog/TagsDialog.vue";
import PresetsDialog from "./components/PresetsDialog/PresetsDialog.vue";
import PreferencesDialog from "./components/PreferencesDialog/PreferencesDialog.vue";
import SkillFilterBar from "./components/SkillFilterBar/SkillFilterBar.vue";
import SkillGrid from "./components/SkillGrid/SkillGrid.vue";
import SkillToolbar from "./components/SkillToolbar/SkillToolbar.vue";
import ToastHost from "./components/ToastHost/ToastHost.vue";
import { resetInstallationState, useAppState } from "./composables/useAppState";
import { useInstallation } from "./composables/useInstallation";
import { useTags } from "./composables/useTags";
import { usePresets } from "./composables/usePresets";
import { usePreferences } from "./composables/usePreferences";
import { useSkills } from "./composables/useSkills";
import { useToasts } from "./composables/useToasts";
import * as backend from "./services/backend";
import type { InstallRequest, InstallScope, Skill } from "./types";
import { formatAgents, resolveAgentOrder } from "./utils/agents";
import { agentsNeedingInstall } from "./utils/skillInstall";

const state = useAppState();
const {
  selectedSkills,
  loadAll,
  refresh,
  clearSelection,
  toggleSelected,
  addSkill,
  updateSkill,
  deleteSkill,
  checkForUpdates,
} = useSkills();
const tags = useTags();
const presets = usePresets();
const { install, cancel, checkDependencies } = useInstallation();
const { load: loadPreferences, update: updatePreferencesPartial } = usePreferences();
const { push: pushToast } = useToasts();

const isCheckingForUpdates = ref(false);
const isPreferencesOpen = ref(false);
const isAgentsOpen = ref(false);
const isTagsOpen = ref(false);
const tagsError = ref<string | null>(null);
const pendingTagKeys = ref(new Set<string>());
const isPresetsOpen = ref(false);
const presetsError = ref<string | null>(null);
const isAddDialogOpen = ref(false);
const addSkillError = ref<string | null>(null);
const isDeletingCatalogSkills = ref(false);
const catalogDeleteError = ref<string | null>(null);

// Bulk-uninstall (the footer's trash-can toggle) — a separate selection
// concept from `state.selectedSkillIds` (which is for *installing*): only
// already-installed Skills are selectable here, and confirming actually
// removes files from disk via the real `skills remove`, unlike the
// catalog-only `handleDeleteCatalogSkills` above.
const isDeleteMode = ref(false);
const deleteSelectedIds = ref(new Set<string>());
const isUninstalling = ref(false);
const hasInstalledSkills = computed(() => state.skills.some((skill) => skill.installed));

const isEditDialogOpen = ref(false);
const editSkillError = ref<string | null>(null);
const skillBeingEditedId = ref<string | null>(null);
const skillBeingEdited = computed<Skill | null>(
  () => state.skills.find((s) => s.id === skillBeingEditedId.value) ?? null,
);

const isInstallConfirmOpen = ref(false);
const skillQuery = ref("");
const skillSort = ref<"name" | "pack" | "local" | "remote">("name");
const skillView = ref<"grid" | "list">("grid");
const skillSourceFilter = ref<"all" | "local" | "remote">("all");
const skillPackFilter = ref<string | null>(null);
const skillNeedsAgentsFilter = ref(false);
// The user's custom Agents-menu order, filled in with any agent missing
// from it (a never-customized or stale preference) — always a full,
// deterministic ordering so Skill card icons and the Agents menu agree.
const agentOrder = computed(() => resolveAgentOrder(state.preferences.agentOrder));

// The toolbar's view control reads as one 3-way choice (Grid / Compact /
// List), but under the hood it's still the same two independent knobs
// SkillGrid always took — `skillView` (grid vs. list layout) and the
// persisted `compactCards` preference (grid density). Compact is a grid
// variant, not a separate layout, so picking it keeps `skillView` on
// "grid" and only flips the density preference.
const displayMode = computed<"grid" | "compact" | "list">(() => {
  if (skillView.value === "list") return "list";
  return state.preferences.compactCards ? "compact" : "grid";
});

function handleDisplayModeChange(mode: "grid" | "compact" | "list") {
  skillView.value = mode === "list" ? "list" : "grid";
  const wantCompact = mode === "compact";
  if (state.preferences.compactCards !== wantCompact) {
    updatePreferencesPartial({ compactCards: wantCompact }).catch((error) => pushToast(describeError(error), "error"));
  }
}

// Enabled Skills already installed for *some* but not *every* currently
// targeted agent — changing "Agents" in the header can silently turn a
// batch of previously-"Installed" cards into gaps like this. Deliberately
// excludes Skills that were never installed anywhere: those aren't a gap to
// close, they're just not selected yet, which the regular selection flow
// already covers.
const skillsNeedingAgents = computed(() =>
  state.skills.filter(
    (skill) =>
      skill.enabled && skill.installed && agentsNeedingInstall(skill, state.preferences.defaultAgents).length > 0,
  ),
);
const skillsNeedingAgentsCount = computed(() => skillsNeedingAgents.value.length);

// What "Save Preset" would capture right now — every currently installed
// Skill's portable `skillName`, not the current checkbox selection.
const installedSkillNames = computed(() => state.skills.filter((skill) => skill.installed).map((skill) => skill.skillName));

const displayedSkills = computed(() => {
  const query = skillQuery.value.trim().toLocaleLowerCase();
  const tagOrder = new Map(state.tags.map((tag) => [tag.id, tag.order]));
  const firstPackOrder = (skill: Skill) =>
    Math.min(...skill.tags.map((id) => tagOrder.get(id) ?? Number.MAX_SAFE_INTEGER), Number.MAX_SAFE_INTEGER);
  return state.skills
    .filter(
      (skill) =>
        !query ||
        skill.displayName.toLocaleLowerCase().includes(query) ||
        skill.description.toLocaleLowerCase().includes(query),
    )
    .filter((skill) => {
      if (skillSourceFilter.value === "local") return skill.local;
      if (skillSourceFilter.value === "remote") return !skill.local;
      return true;
    })
    .filter((skill) => !skillPackFilter.value || skill.tags.includes(skillPackFilter.value))
    .filter((skill) => !skillNeedsAgentsFilter.value || skillsNeedingAgents.value.includes(skill))
    .slice()
    .sort((a, b) => {
      // Selected skills lead, then installed-but-unselected ones, then
      // everything else — installed status gets the same leading treatment
      // as selection so a just-finished install is immediately visible
      // without hunting through the grid.
      const priority = (skill: Skill) => {
        if (state.selectedSkillIds.has(skill.id)) return 0;
        if (skill.installed) return 1;
        return 2;
      };
      const aPriority = priority(a);
      const bPriority = priority(b);
      if (aPriority !== bPriority) return aPriority - bPriority;
      if (aPriority === 0) {
        const byPack = firstPackOrder(a) - firstPackOrder(b);
        if (byPack) return byPack;
        return a.displayName.localeCompare(b.displayName);
      }
      if (skillSort.value === "pack") {
        const byPack = firstPackOrder(a) - firstPackOrder(b);
        if (byPack) return byPack;
      } else if (skillSort.value === "local" || skillSort.value === "remote") {
        const leads = (skill: Skill) => (skillSort.value === "local" ? skill.local : !skill.local);
        const bySource = Number(!leads(a)) - Number(!leads(b));
        if (bySource) return bySource;
      }
      return a.displayName.localeCompare(b.displayName);
    });
});

const defaultGroupId = computed(() => state.groups.find((g) => g.id === "other")?.id ?? state.groups[0]?.id ?? null);

// Moved here from AppHeader: the footer's Agents button now owns this
// summary (icons instead of a plain label), replacing the header item that
// used to sit next to the app title.
const agentLabels = computed(() => formatAgents(state.preferences.defaultAgents) || "None selected");

// Moved here from AppHeader: the footer's status row now owns this display,
// so it reads straight off state instead of taking dependencyStatus as a prop.
const dependencyClass = computed(() => {
  if (!state.dependencyStatus) return "is-unknown";
  return state.dependencyStatus.available ? "is-ready" : "is-missing";
});

const dependencyLabel = computed(() => {
  if (!state.dependencyStatus) return "checking…";
  return state.dependencyStatus.available ? "● Ready" : "● Not found";
});

// Skills install one at a time (see the backend's sequential install loop),
// so at most one card is ever "currently installing".
const installingSkillId = computed(() =>
  state.installation.isInstalling ? state.installation.currentSkillId : null,
);

const installOptionsFromPreferences = computed(() => ({
  agents: state.preferences.defaultAgents,
  scope: state.preferences.lastScope,
  // Ignored by the backend entirely when scope is Global — see
  // `SkillsCliInstaller::resolve_cwd`.
  projectPath: state.projectRoot,
  copy: state.preferences.copyByDefault,
  dryRun: false,
  confirm: state.preferences.confirmBeforeInstall,
  continueOnError: state.preferences.continueAfterFailure,
}));

function handleProjectPathUpdate(path: string) {
  state.projectRoot = path;
  refresh().catch((error) => pushToast(describeError(error), "error"));
  updatePreferencesPartial({ lastProjectPath: path }).catch((error) => pushToast(describeError(error), "error"));
}

function handleScopeUpdate(scope: InstallScope) {
  if (scope === state.preferences.lastScope) return;
  updatePreferencesPartial({ lastScope: scope })
    .then(() => refresh())
    .catch((error) => pushToast(describeError(error), "error"));
}

// Shared by every Pack badge toggle: all-selected flips to none, anything
// else flips to all — so a half-selected group always completes forward to
// "all" first, matching SkillToolbar's own tagState.
function toggleSelectionForIds(ids: string[]) {
  if (!ids.length) return;
  const allSelected = ids.every((id) => state.selectedSkillIds.has(id));
  const next = new Set(state.selectedSkillIds);
  for (const id of ids) {
    if (allSelected) next.delete(id);
    else next.add(id);
  }
  state.selectedSkillIds = next;
}

function toggleTagSelection(tagId: string) {
  // A skill fully installed for every targeted agent is never selectable
  // (see useSkills' toggleSelected) — a Pack toggle only ever acts on the
  // rest of the pack.
  const skillIds = state.skills
    .filter(
      (skill) => skill.enabled && agentsNeedingInstall(skill, state.preferences.defaultAgents).length > 0 && skill.tags.includes(tagId),
    )
    .map((skill) => skill.id);
  toggleSelectionForIds(skillIds);
}

// A selection action, not a filter one — deliberately independent of
// whatever the Filter bar's search/source/pack/Needs-agents controls are
// currently narrowing the grid to, so it always adds every gap skill in the
// whole catalog. Adds to the current selection rather than replacing it, the
// same way Pack toggle does.
function handleSelectMissingAgents() {
  const skillIds = skillsNeedingAgents.value.map((skill) => skill.id);
  if (!skillIds.length) return;
  state.selectedSkillIds = new Set([...state.selectedSkillIds, ...skillIds]);
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

onMounted(async () => {
  try {
    await Promise.all([loadAll(), loadPreferences(), presets.loadAll()]);
    // loadAll()'s initial load always comes back Project-scoped against the
    // app's own launch directory (the backend has no preferences to consult
    // yet at that point) — once the persisted scope/folder are in, restore
    // them and reload, but only if doing so actually changes anything; most
    // launches have nothing to restore, and re-scanning identical state on
    // every single startup would be pure waste.
    const launchProjectRoot = state.projectRoot;
    if (state.preferences.lastProjectPath) state.projectRoot = state.preferences.lastProjectPath;
    if (state.preferences.lastScope === "global" || state.projectRoot !== launchProjectRoot) {
      await refresh();
    }
  } catch (error) {
    pushToast(describeError(error), "error");
  }
  try {
    await checkDependencies();
  } catch (error) {
    pushToast(describeError(error), "error");
  }
});

function handlePreferencesUpdate(partial: Parameters<typeof updatePreferencesPartial>[0]) {
  updatePreferencesPartial(partial).catch((error) => pushToast(describeError(error), "error"));
}
function handleOpenPacksFromPreferences() {
  isPreferencesOpen.value = false;
  isTagsOpen.value = true;
}
function handleOpenAgentsFromPreferences() {
  isPreferencesOpen.value = false;
  isAgentsOpen.value = true;
}
async function handleExportConfig() {
  try {
    const content = await backend.exportPortableConfiguration();
    const blob = new window.Blob([content], { type: "application/json" });
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "skills-installer-config.json";
    link.click();
    window.URL.revokeObjectURL(url);
    pushToast("Configuration exported", "success");
  } catch (error) {
    pushToast(describeError(error), "error");
  }
}
async function handleImportConfig(content: string) {
  try {
    await backend.importPortableConfiguration(content);
    await Promise.all([loadAll(), loadPreferences()]);
    pushToast("Configuration imported", "success");
  } catch (error) {
    pushToast(describeError(error), "error");
  }
}
function handleAgentsUpdate(agents: string[]) {
  updatePreferencesPartial({ defaultAgents: agents }).catch((error) => pushToast(describeError(error), "error"));
}
function handleAgentOrderUpdate(order: string[]) {
  updatePreferencesPartial({ agentOrder: order }).catch((error) => pushToast(describeError(error), "error"));
}
function handleLocalRefresh() {
  refresh().catch((error) => pushToast(describeError(error), "error"));
}
async function handleLocalSourceUpdate(path: string) {
  try {
    await updatePreferencesPartial({ localSourcePath: path });
    await refresh();
    pushToast("Local Skill Source updated", "success");
  } catch (error) {
    pushToast(describeError(error), "error");
  }
}
async function handleInstallCli() {
  try {
    const result = await backend.installCliCommand();
    const pathNote = result.pathConfigured
      ? ""
      : ` Add ${result.commandPath.replace(/\/skills-installer$/, "")} to your PATH.`;
    pushToast(`Command installed at ${result.commandPath}.${pathNote}`, "success");
  } catch (error) {
    pushToast(describeError(error), "error");
  }
}

async function tagAction(action: () => Promise<unknown>, failure: string) {
  tagsError.value = null;
  try {
    await action();
  } catch (error) {
    tagsError.value = failure;
    pushToast(`${failure} ${describeError(error)}`, "error");
  }
}
async function persistTagToggle(skillId: string, tagId: string, assigned: boolean) {
  const key = `${skillId}:${tagId}`;
  if (pendingTagKeys.value.has(key)) return;
  pendingTagKeys.value = new Set(pendingTagKeys.value).add(key);
  const skill = state.skills.find((item) => item.id === skillId);
  const tag = state.tags.find((item) => item.id === tagId);
  const verb = assigned ? "assign" : "remove";
  const failure = `Could not ${verb} "${tag?.name ?? "Pack"}" ${assigned ? "to" : "from"} "${skill?.displayName ?? "Skill"}".`;
  try {
    await tagAction(() => (assigned ? tags.assign(skillId, tagId) : tags.unassign(skillId, tagId)), failure);
  } finally {
    const next = new Set(pendingTagKeys.value);
    next.delete(key);
    pendingTagKeys.value = next;
  }
}
function handleAssignTag(skillId: string, tagId: string) {
  void persistTagToggle(skillId, tagId, true);
}
function handleUnassignTag(skillId: string, tagId: string) {
  void persistTagToggle(skillId, tagId, false);
}
function handleCreateTag(name: string, color: string) {
  void tagAction(() => tags.create(name, color), "Could not create Pack.");
}
function handleUpdateTag(tagId: string, name: string, color: string) {
  void tagAction(() => tags.update(tagId, name, color), "Could not update Pack");
}
function handleDeleteTag(tagId: string) {
  void tagAction(() => tags.remove(tagId), "Could not delete Pack");
}
function handleReorderTags(tagIds: string[]) {
  void tagAction(() => tags.reorder(tagIds), "Could not reorder Packs.");
}

async function handleSavePreset(name: string, gitUrl: string | null) {
  try {
    await presets.save(name, gitUrl, installedSkillNames.value);
    presetsError.value = null;
    pushToast(`Preset "${name}" saved.`, "success");
  } catch (error) {
    presetsError.value = describeError(error);
  }
}

// Replaces the current selection outright (matching "Load" semantics, not
// "add to") — Skills the Preset references that no longer exist here (a
// different machine, a removed Skill, an unconfigured Local catalog) are
// silently skipped rather than failing the whole load.
function handleLoadPreset(presetId: string) {
  const preset = state.presets.find((p) => p.id === presetId);
  if (!preset) return;
  const names = new Set(preset.skillNames);
  const ids = state.skills.filter((skill) => skill.enabled && names.has(skill.skillName)).map((skill) => skill.id);
  state.selectedSkillIds = new Set(ids);
  isPresetsOpen.value = false;
  const missing = preset.skillNames.length - ids.length;
  if (missing > 0) {
    pushToast(
      `Selected ${ids.length} of ${preset.skillNames.length} Skills from "${preset.name}" — ${missing} not found here.`,
      ids.length ? "success" : "error",
    );
  } else {
    pushToast(`Selected ${ids.length} Skills from "${preset.name}".`, "success");
  }
}

function handleDeletePreset(presetId: string) {
  presets.remove(presetId).catch((error) => pushToast(describeError(error), "error"));
}

function openAddDialog() {
  addSkillError.value = null;
  isAddDialogOpen.value = true;
}

async function handleAddSkillSubmit(payload: backend.AddSkillArgs) {
  try {
    await addSkill(payload);
    isAddDialogOpen.value = false;
    pushToast("Skill added", "success");
  } catch (error) {
    addSkillError.value = describeError(error);
  }
}

async function handleImportPack(payload: backend.ImportPackArgs) {
  try {
    const result = await backend.importPack(payload);
    await loadAll();
    isAddDialogOpen.value = false;
    const skipped = result.skipped.length ? ` ${result.skipped.length} duplicate(s) skipped.` : "";
    pushToast(`${result.added.length} Skills added to ${result.tag.name}.${skipped}`, "success");
  } catch (error) {
    addSkillError.value = describeError(error);
  }
}

// Sequential, not Promise.all: each delete is a read-modify-write against
// the same skills.yaml, so concurrent deletes could race and clobber one
// another's change.
async function handleDeleteCatalogSkills(skillIds: string[]) {
  isDeletingCatalogSkills.value = true;
  catalogDeleteError.value = null;
  try {
    for (const skillId of skillIds) {
      await deleteSkill(skillId);
    }
    pushToast(skillIds.length === 1 ? "Skill deleted" : `${skillIds.length} Skills deleted`, "success");
  } catch (error) {
    catalogDeleteError.value = describeError(error);
  } finally {
    isDeletingCatalogSkills.value = false;
  }
}

async function handleUpdateCatalogSkill(payload: backend.UpdateSkillArgs) {
  try {
    await updateSkill(payload);
    pushToast("Skill updated", "success");
  } catch (error) {
    pushToast(`Could not update Skill. ${describeError(error)}`, "error");
  }
}

// Where Edit Skill was opened from — when set, dismissing it without
// completing an action (Esc, backdrop click, or Cancel) reopens that dialog
// instead of just dropping back to the main page, since the user was in the
// middle of managing Skills there. A successful save or delete instead
// finishes the flow, so those set `editSkillClosedByAction` to skip it.
const editSkillOrigin = ref<"add-skill" | null>(null);
const editSkillClosedByAction = ref(false);

function openEditDialog(skillId: string, origin: "add-skill" | null = null) {
  editSkillError.value = null;
  skillBeingEditedId.value = skillId;
  editSkillOrigin.value = origin;
  isEditDialogOpen.value = true;
}

function handleEditSkillFromCatalog(skillId: string) {
  isAddDialogOpen.value = false;
  openEditDialog(skillId, "add-skill");
}

function handleEditDialogOpenChange(open: boolean) {
  isEditDialogOpen.value = open;
  if (open) return;
  if (editSkillOrigin.value === "add-skill" && !editSkillClosedByAction.value) {
    isAddDialogOpen.value = true;
  }
  editSkillOrigin.value = null;
  editSkillClosedByAction.value = false;
}

async function handleEditSkillSubmit(payload: backend.UpdateSkillArgs) {
  try {
    await updateSkill(payload);
    editSkillClosedByAction.value = true;
    isEditDialogOpen.value = false;
    pushToast("Skill updated", "success");
  } catch (error) {
    editSkillError.value = describeError(error);
  }
}

async function handleDeleteSkill(skillId: string) {
  try {
    await deleteSkill(skillId);
    editSkillClosedByAction.value = true;
    isEditDialogOpen.value = false;
    skillBeingEditedId.value = null;
    pushToast("Skill deleted", "success");
  } catch (error) {
    editSkillError.value = describeError(error);
  }
}

function enterDeleteMode() {
  isDeleteMode.value = true;
  deleteSelectedIds.value = new Set();
}
function exitDeleteMode() {
  isDeleteMode.value = false;
  deleteSelectedIds.value = new Set();
}
function toggleDeleteSelected(skillId: string) {
  const next = new Set(deleteSelectedIds.value);
  if (next.has(skillId)) next.delete(skillId);
  else next.add(skillId);
  deleteSelectedIds.value = next;
}
// "All" means every installed Skill currently visible under the active
// search/filter — matches what the user can actually see, not the whole
// catalog behind an active filter they may not even remember is on.
function selectAllForDelete() {
  deleteSelectedIds.value = new Set(displayedSkills.value.filter((skill) => skill.installed).map((skill) => skill.id));
}
function deselectAllForDelete() {
  deleteSelectedIds.value = new Set();
}
async function confirmUninstall() {
  if (deleteSelectedIds.value.size === 0 || isUninstalling.value) return;
  isUninstalling.value = true;
  try {
    const result = await backend.uninstallSkills({
      selection: { skillIds: [...deleteSelectedIds.value] },
      scope: state.preferences.lastScope,
      projectPath: state.projectRoot,
    });
    exitDeleteMode();
    await refresh();
    if (result.failed > 0) {
      pushToast(result.message ?? `Failed to uninstall ${result.failed} Skill${result.failed === 1 ? "" : "s"}`, "error");
    } else {
      pushToast(result.removed === 1 ? "Skill uninstalled" : `${result.removed} Skills uninstalled`, "success");
    }
  } catch (error) {
    pushToast(describeError(error), "error");
  } finally {
    isUninstalling.value = false;
  }
}

function toggleSkipConfirm(event: Event) {
  const skip = (event.target as HTMLInputElement).checked;
  updatePreferencesPartial({ confirmBeforeInstall: !skip }).catch((error) => pushToast(describeError(error), "error"));
}

function handleInstallClick() {
  if (state.preferences.confirmBeforeInstall) {
    isInstallConfirmOpen.value = true;
  } else {
    void runInstall();
  }
}

// `agentsOverride` is the confirmation dialog's per-install agent selection
// (a skipped chip there excludes that agent from just this run, without
// touching the configured defaults) — falls back to those defaults when the
// dialog was skipped entirely (see handleInstallClick).
async function runInstall(agentsOverride?: string[]) {
  const request: InstallRequest = {
    selection: { skillIds: [...state.selectedSkillIds] },
    options: { ...installOptionsFromPreferences.value, agents: agentsOverride ?? installOptionsFromPreferences.value.agents },
  };
  try {
    await install(request);
    await refresh();
  } catch (error) {
    pushToast(describeError(error), "error");
  }
}

function handleCancelInstall() {
  cancel().catch((error) => pushToast(describeError(error), "error"));
}

async function handleCheckForUpdates() {
  isCheckingForUpdates.value = true;
  try {
    const outdated = await checkForUpdates();
    if (outdated.length === 0) {
      pushToast("All installed Local skills are up to date.", "success");
    } else {
      pushToast(`${outdated.length} Local skill${outdated.length === 1 ? "" : "s"} can be updated.`, "success");
    }
  } catch (error) {
    pushToast(describeError(error), "error");
  } finally {
    isCheckingForUpdates.value = false;
  }
}

async function handleUpdateSkill(skillId: string) {
  if (state.installation.isInstalling) {
    pushToast("Wait for the current installation to finish before updating.", "error");
    return;
  }
  const request: InstallRequest = {
    selection: { skillIds: [skillId] },
    options: installOptionsFromPreferences.value,
  };
  try {
    await install(request);
    const next = new Set(state.skillsWithUpdates);
    next.delete(skillId);
    state.skillsWithUpdates = next;
    await refresh();
  } catch (error) {
    pushToast(describeError(error), "error");
  }
}

function dismissInstallPanel() {
  resetInstallationState();
}
</script>

<style scoped lang="scss" src="./App.scss"></style>
