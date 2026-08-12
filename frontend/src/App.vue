<template>
  <div class="app-shell">
    <AppHeader :project-path="state.projectRoot" :dependency-status="state.dependencyStatus" :scope="state.preferences.defaultScope" :agents="state.preferences.defaultAgents" @update:project-path="handleProjectPathUpdate" />

    <main class="app-shell__main">
      <SkillToolbar :tags="state.tags" :skills="state.skills" :selected-ids="[...state.selectedSkillIds]" :query="skillQuery" :sort-by="skillSort" @toggle-tag="toggleTagSelection" @clear-selection="clearSelection" @update:query="skillQuery = $event" @update:sort-by="skillSort = $event" />

      <SkillGrid
        :skills="displayedSkills"
        :tags="state.tags"
        :selected-ids="state.selectedSkillIds"
        @toggle="toggleSelected"
        @edit="openEditDialog"
        @description="openDescriptionDialog"
      />

      <InstallProgressPanel
        v-if="state.installation.isInstalling || state.installation.result || state.installation.error"
        :installation="state.installation"
        @cancel="handleCancelInstall"
        @dismiss="dismissInstallPanel"
      />
    </main>

    <footer class="app-shell__footer">
      <div class="app-shell__footer-left">
        <button type="button" class="app-shell__footer-btn app-shell__footer-btn--add" @click="openAddDialog">
          Add Skill
        </button>
        <button type="button" class="app-shell__footer-btn" @click="isPreferencesOpen = !isPreferencesOpen">
          Preferences
        </button>
        <button type="button" class="app-shell__footer-btn" @click="isAgentsOpen = true">Agents</button>
        <button type="button" class="app-shell__footer-btn" @click="isTagsOpen = !isTagsOpen">Packs</button>
        <span class="app-shell__selected-count">{{ selectedSkills.length }} selected</span>
      </div>
      <div class="app-shell__footer-actions">
        <button
          type="button"
          class="app-shell__footer-btn app-shell__footer-btn--primary"
          :disabled="selectedSkills.length === 0 || state.installation.isInstalling"
          @click="handleInstallClick"
        >
          Install Selected
        </button>
      </div>
    </footer>

    <AddSkillDialog
      v-model:open="isAddDialogOpen"
      :default-group-id="defaultGroupId"
      :submit-error="addSkillError"
      @submit="handleAddSkillSubmit"
    />

    <EditSkillDialog
      v-model:open="isEditDialogOpen"
      :skill="skillBeingEdited"
      :tags="state.tags"
      :submit-error="editSkillError"
      @submit="handleEditSkillSubmit"
    />

    <SkillDescriptionDialog v-model:open="isDescriptionDialogOpen" :skill="skillBeingDescribed" />

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
    />
    <AgentsDialog v-model:open="isAgentsOpen" :model-value="state.preferences.defaultAgents" :scope="state.preferences.defaultScope" @update:model-value="handleAgentsUpdate" />
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

    <ToastHost />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import AddSkillDialog from './components/AddSkillDialog/AddSkillDialog.vue'
import AgentsDialog from './components/AgentsDialog/AgentsDialog.vue'
import AppHeader from './components/AppHeader/AppHeader.vue'
import EditSkillDialog from './components/EditSkillDialog/EditSkillDialog.vue'
import InstallConfirmDialog from './components/InstallConfirmDialog/InstallConfirmDialog.vue'
import InstallProgressPanel from './components/InstallProgressPanel/InstallProgressPanel.vue'
import TagsDialog from './components/TagsDialog/TagsDialog.vue'
import PreferencesDialog from './components/PreferencesDialog/PreferencesDialog.vue'
import SkillGrid from './components/SkillGrid/SkillGrid.vue'
import SkillDescriptionDialog from './components/SkillDescriptionDialog/SkillDescriptionDialog.vue'
import SkillToolbar from './components/SkillToolbar/SkillToolbar.vue'
import ToastHost from './components/ToastHost/ToastHost.vue'
import { resetInstallationState, useAppState } from './composables/useAppState'
import { useInstallation } from './composables/useInstallation'
import { useTags } from './composables/useTags'
import { usePreferences } from './composables/usePreferences'
import { useSkills } from './composables/useSkills'
import { useToasts } from './composables/useToasts'
import * as backend from './services/backend'
import type { InstallRequest, Skill } from './types'

const state = useAppState()
const {
  selectedSkills,
  loadAll,
  refresh,
  clearSelection,
  toggleSelected,
  addSkill,
  updateSkill,
} = useSkills()
const tags = useTags()
const { install, cancel, checkDependencies } = useInstallation()
const { load: loadPreferences, update: updatePreferencesPartial } = usePreferences()
const { push: pushToast } = useToasts()

const isPreferencesOpen = ref(false)
const isAgentsOpen = ref(false)
const isTagsOpen = ref(false)
const tagsError = ref<string | null>(null)
const pendingTagKeys = ref(new Set<string>())
const isAddDialogOpen = ref(false)
const addSkillError = ref<string | null>(null)

const isEditDialogOpen = ref(false)
const editSkillError = ref<string | null>(null)
const skillBeingEditedId = ref<string | null>(null)
const skillBeingEdited = computed<Skill | null>(
  () => state.skills.find((s) => s.id === skillBeingEditedId.value) ?? null,
)
const isDescriptionDialogOpen = ref(false)
const skillBeingDescribedId = ref<string | null>(null)
const skillBeingDescribed = computed<Skill | null>(
  () => state.skills.find((skill) => skill.id === skillBeingDescribedId.value) ?? null,
)

const isInstallConfirmOpen = ref(false)
const skillQuery = ref('')
const skillSort = ref<'name' | 'pack'>('name')

const displayedSkills = computed(() => {
  const query = skillQuery.value.trim().toLocaleLowerCase()
  const tagOrder = new Map(state.tags.map((tag) => [tag.id, tag.order]))
  const firstPackOrder = (skill: Skill) => Math.min(...skill.tags.map((id) => tagOrder.get(id) ?? Number.MAX_SAFE_INTEGER), Number.MAX_SAFE_INTEGER)
  return state.skills
    .filter((skill) => !query || skill.displayName.toLocaleLowerCase().includes(query) || skill.description.toLocaleLowerCase().includes(query))
    .slice()
    .sort((a, b) => {
      if (skillSort.value === 'pack') {
        const byPack = firstPackOrder(a) - firstPackOrder(b)
        if (byPack) return byPack
      }
      return a.displayName.localeCompare(b.displayName)
    })
})

const defaultGroupId = computed(() => state.groups.find((g) => g.id === 'other')?.id ?? state.groups[0]?.id ?? null)

const installOptionsFromPreferences = computed(() => ({
  agents: state.preferences.defaultAgents,
  projectPath: state.preferences.defaultScope === 'project' ? state.projectRoot : null,
  copy: state.preferences.copyByDefault,
  scope: state.preferences.defaultScope,
  dryRun: false,
  confirm: state.preferences.confirmBeforeInstall,
  continueOnError: state.preferences.continueAfterFailure,
}))

function handleProjectPathUpdate(path: string) {
  state.projectRoot = path
}

function toggleTagSelection(tagId: string) {
  const skillIds = state.skills.filter((skill) => skill.enabled && skill.tags.includes(tagId)).map((skill) => skill.id)
  if (!skillIds.length) return
  const allSelected = skillIds.every((id) => state.selectedSkillIds.has(id))
  const next = new Set(state.selectedSkillIds)
  for (const id of skillIds) {
    if (allSelected) next.delete(id)
    else next.add(id)
  }
  state.selectedSkillIds = next
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

onMounted(async () => {
  try {
    await Promise.all([loadAll(), loadPreferences()])
  } catch (error) {
    pushToast(describeError(error), 'error')
  }
  try {
    await checkDependencies()
  } catch (error) {
    pushToast(describeError(error), 'error')
  }
})

function handlePreferencesUpdate(partial: Parameters<typeof updatePreferencesPartial>[0]) {
  updatePreferencesPartial(partial).catch((error) => pushToast(describeError(error), 'error'))
}
async function handleExportConfig(){try{const content=await backend.exportPortableConfiguration();const blob=new window.Blob([content],{type:'application/json'});const url=window.URL.createObjectURL(blob);const link=document.createElement('a');link.href=url;link.download='skills-installer-config.json';link.click();window.URL.revokeObjectURL(url);pushToast('Configuration exported','success')}catch(error){pushToast(describeError(error),'error')}}
async function handleImportConfig(content:string){try{await backend.importPortableConfiguration(content);await Promise.all([loadAll(),loadPreferences()]);pushToast('Configuration imported','success')}catch(error){pushToast(describeError(error),'error')}}
function handleAgentsUpdate(agents:string[]){updatePreferencesPartial({defaultAgents:agents}).catch(error=>pushToast(describeError(error),'error'))}
function handleLocalRefresh(){refresh().catch(error=>pushToast(describeError(error),'error'))}
async function handleLocalSourceUpdate(path:string){try{await updatePreferencesPartial({localSourcePath:path});await refresh();pushToast('Local Skill Source updated','success')}catch(error){pushToast(describeError(error),'error')}}

async function tagAction(action: () => Promise<unknown>, failure: string) { tagsError.value=null;try{await action()}catch(error){tagsError.value=failure;pushToast(`${failure} ${describeError(error)}`,'error')} }
async function persistTagToggle(skillId:string,tagId:string,assigned:boolean){
  const key=`${skillId}:${tagId}`
  if(pendingTagKeys.value.has(key))return
  pendingTagKeys.value=new Set(pendingTagKeys.value).add(key)
  const skill=state.skills.find(item=>item.id===skillId)
  const tag=state.tags.find(item=>item.id===tagId)
  const verb=assigned?'assign':'remove'
  const failure=`Could not ${verb} "${tag?.name??'Pack'}" ${assigned?'to':'from'} "${skill?.displayName??'Skill'}".`
  try{await tagAction(()=>assigned?tags.assign(skillId,tagId):tags.unassign(skillId,tagId),failure)}finally{const next=new Set(pendingTagKeys.value);next.delete(key);pendingTagKeys.value=next}
}
function handleAssignTag(skillId:string,tagId:string){void persistTagToggle(skillId,tagId,true)}
function handleUnassignTag(skillId:string,tagId:string){void persistTagToggle(skillId,tagId,false)}
function handleCreateTag(name:string,color:string){void tagAction(()=>tags.create(name,color),'Could not create Pack.')}
function handleUpdateTag(tagId:string,name:string,color:string){void tagAction(()=>tags.update(tagId,name,color),'Could not update Pack')}
function handleDeleteTag(tagId:string){void tagAction(()=>tags.remove(tagId),'Could not delete Pack')}

function openAddDialog() {
  addSkillError.value = null
  isAddDialogOpen.value = true
}

async function handleAddSkillSubmit(payload: backend.AddSkillArgs) {
  try {
    await addSkill(payload)
    isAddDialogOpen.value = false
    pushToast('Skill added', 'success')
  } catch (error) {
    addSkillError.value = describeError(error)
  }
}

function openEditDialog(skillId: string) {
  editSkillError.value = null
  skillBeingEditedId.value = skillId
  isEditDialogOpen.value = true
}

function openDescriptionDialog(skillId: string) {
  skillBeingDescribedId.value = skillId
  isDescriptionDialogOpen.value = true
}

async function handleEditSkillSubmit(payload: backend.UpdateSkillArgs) {
  try {
    await updateSkill(payload)
    isEditDialogOpen.value = false
    pushToast('Skill updated', 'success')
  } catch (error) {
    editSkillError.value = describeError(error)
  }
}

function handleInstallClick() {
  if (state.preferences.confirmBeforeInstall) {
    isInstallConfirmOpen.value = true
  } else {
    void runInstall()
  }
}

async function runInstall() {
  const request: InstallRequest = {
    selection: { skillIds: [...state.selectedSkillIds] },
    options: installOptionsFromPreferences.value,
  }
  try {
    await install(request)
    await refresh()
  } catch (error) {
    pushToast(describeError(error), 'error')
  }
}

function handleCancelInstall() {
  cancel().catch((error) => pushToast(describeError(error), 'error'))
}

function dismissInstallPanel() {
  resetInstallationState()
}
</script>

<style scoped lang="scss" src="./App.scss"></style>
