<template>
  <header class="app-header">
    <h1 class="app-header__title">Skills Installer</h1>
    <div class="app-header__item app-header__destination">
      <span class="app-header__label">Install to</span>
      <form v-if="editingPath" class="app-header__path-editor" @submit.prevent="savePath">
        <input ref="pathInput" v-model="draftPath" aria-label="Project installation destination" @keydown.esc.prevent="cancelPath" />
        <button type="submit">Use</button><button type="button" @click="cancelPath">Cancel</button>
      </form>
      <button v-else type="button" class="app-header__value app-header__path" :disabled="scope === 'global'" :title="scope === 'global' ? 'Change to Project scope to select a folder' : 'Change installation destination'" @click="beginPathEdit">
        {{ scope === 'global' ? 'Global installation' : projectPath || '(select a project)' }}
        <span v-if="scope === 'project'" aria-hidden="true">✎</span>
      </button>
    </div>
    <div class="app-header__item app-header__agents"><span class="app-header__label">Agents</span><strong class="app-header__value">{{ agentLabels }}</strong></div>
    <div class="app-header__status-row">
      <span class="app-header__dependency" :class="dependencyClass">
        <span class="app-header__dependency-dot" aria-hidden="true" />
        Skills CLI {{ dependencyLabel }}
      </span>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'

import { SUPPORTED_AGENTS, type DependencyStatus, type InstallScope } from '../../types'

const props = defineProps<{
  projectPath: string
  dependencyStatus: DependencyStatus | null
  scope: InstallScope
  agents: string[]
}>()
const emit=defineEmits<{ 'update:projectPath':[path:string] }>()
const editingPath=ref(false),draftPath=ref(''),pathInput=ref<HTMLInputElement|null>(null)
async function beginPathEdit(){if(props.scope==='global')return;draftPath.value=props.projectPath;editingPath.value=true;await nextTick();pathInput.value?.focus();pathInput.value?.select()}
function cancelPath(){editingPath.value=false}
function savePath(){const path=draftPath.value.trim();if(path){emit('update:projectPath',path);editingPath.value=false}}
const agentLabels=computed(()=>props.agents.map(id=>SUPPORTED_AGENTS.find(agent=>agent.id===id)?.label??id).join(', ')||'None selected')

const dependencyClass = computed(() => {
  if (!props.dependencyStatus) return 'is-unknown'
  return props.dependencyStatus.available ? 'is-ready' : 'is-missing'
})

const dependencyLabel = computed(() => {
  if (!props.dependencyStatus) return 'checking…'
  return props.dependencyStatus.available ? '● Ready' : '● Not found'
})
</script>

<style scoped lang="scss" src="./AppHeader.scss"></style>
