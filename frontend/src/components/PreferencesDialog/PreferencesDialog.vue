<template>
  <dialog ref="dialogEl" class="preferences-dialog" @close="emit('update:open', false)">
    <div class="preferences-dialog__form">
      <h2 class="preferences-dialog__title">Preferences</h2>

      <div class="preferences-dialog__field">
        <span class="preferences-dialog__label">Interface size</span>
        <FontScaleControl
          variant="full"
          :model-value="preferences.fontScale"
          @update:model-value="(v) => emit('update', { fontScale: v })"
        />
      </div>

      <label class="preferences-dialog__checkbox">
        <input type="checkbox" :checked="preferences.copyByDefault" @change="handleCopyChange" />
        <span>Copy instead of link</span>
      </label>

      <div class="preferences-dialog__field">
        <span class="preferences-dialog__label">Accent</span>
        <div class="preferences-dialog__accents" role="radiogroup" aria-label="Application accent">
          <button v-for="item in accents" :key="item.id" type="button" :class="{ 'is-active': preferences.accent === item.id }" :aria-label="item.label" :aria-pressed="preferences.accent === item.id" :style="{ backgroundColor:item.color }" @click="emit('update',{accent:item.id})" />
        </div>
      </div>

      <div class="preferences-dialog__field">
        <span class="preferences-dialog__label">Configuration</span>
        <div class="preferences-dialog__config-actions"><button type="button" @click="emit('exportConfig')">Export</button><button type="button" @click="fileInput?.click()">Import</button><input ref="fileInput" type="file" accept="application/json,.json" hidden @change="handleImport" /></div>
      </div>

      <div class="preferences-dialog__field">
        <label class="preferences-dialog__label" for="local-skill-source">Local Skill Source</label>
        <span class="preferences-dialog__help">This folder is a catalog of installable Skills, not an installation destination.</span>
        <input
          id="local-skill-source"
          v-model="localSource"
          class="preferences-dialog__input"
          type="text"
          placeholder="~/.control/skill"
        />
        <div class="preferences-dialog__source-actions">
          <button type="button" :disabled="!canSaveLocalSource" @click="saveLocalSource">Change Folder</button>
          <button type="button" @click="emit('refreshLocalSource')">Refresh</button>
        </div>
      </div>

      <div class="preferences-dialog__field">
        <span class="preferences-dialog__label">Installation scope</span>
        <div class="preferences-dialog__segmented" role="radiogroup" aria-label="Installation scope">
          <button
            type="button"
            class="preferences-dialog__segment"
            :class="{ 'is-active': preferences.defaultScope === 'project' }"
            @click="emit('update', { defaultScope: 'project' })"
          >
            Project
          </button>
          <button
            type="button"
            class="preferences-dialog__segment"
            :class="{ 'is-active': preferences.defaultScope === 'global' }"
            @click="emit('update', { defaultScope: 'global' })"
          >
            Global
          </button>
        </div>
      </div>

      <label class="preferences-dialog__checkbox">
        <input type="checkbox" :checked="preferences.confirmBeforeInstall" @change="handleConfirmChange" />
        <span>Confirm before installation</span>
      </label>

      <label class="preferences-dialog__checkbox">
        <input type="checkbox" :checked="preferences.continueAfterFailure" @change="handleContinueChange" />
        <span>Continue after one skill fails</span>
      </label>

      <div class="preferences-dialog__actions">
        <button type="button" class="preferences-dialog__btn preferences-dialog__btn--primary" @click="close">Done</button>
      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import { useNativeDialog } from '../../composables/useNativeDialog'
import type { AccentColor, UiPreferences } from '../../types'
import FontScaleControl from '../FontScaleControl/FontScaleControl.vue'

const props = defineProps<{
  open: boolean
  preferences: UiPreferences
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  update: [partial: Partial<UiPreferences>]
  exportConfig: []
  importConfig: [content:string]
  updateLocalSource: [path: string]
  refreshLocalSource: []
}>()

const dialogEl = ref<HTMLDialogElement | null>(null)
const fileInput=ref<HTMLInputElement|null>(null)
const localSource = ref('')

function loadLocalSource() {
  localSource.value = props.preferences.localSourcePath ?? '~/.control/skill'
}

useNativeDialog(dialogEl, () => props.open, loadLocalSource)
watch(() => props.preferences.localSourcePath, loadLocalSource)

function close() {
  emit('update:open', false)
}

const canSaveLocalSource = computed(() => localSource.value.trim().length > 0 && localSource.value.trim() !== props.preferences.localSourcePath)

function saveLocalSource() {
  if (canSaveLocalSource.value) emit('updateLocalSource', localSource.value.trim())
}

const accents:{id:AccentColor;label:string;color:string}[]=[{id:'pink',label:'Pink',color:'#F43F75'},{id:'coral',label:'Red / Coral',color:'#F05252'},{id:'blue',label:'Blue',color:'#3B82F6'},{id:'teal',label:'Teal',color:'#14B8A6'},{id:'violet',label:'Violet',color:'#A855F7'},{id:'green',label:'Green',color:'#22C55E'}]
function handleCopyChange(event: Event) {
  emit('update', { copyByDefault: (event.target as HTMLInputElement).checked })
}
function handleConfirmChange(event: Event) {
  emit('update', { confirmBeforeInstall: (event.target as HTMLInputElement).checked })
}
function handleContinueChange(event: Event) {
  emit('update', { continueAfterFailure: (event.target as HTMLInputElement).checked })
}
async function handleImport(event:Event){const file=(event.target as HTMLInputElement).files?.[0];if(file)emit('importConfig',await file.text());(event.target as HTMLInputElement).value=''}
</script>

<style scoped lang="scss" src="./PreferencesDialog.scss"></style>
