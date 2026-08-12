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
import { ref } from 'vue'

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
}>()

const dialogEl = ref<HTMLDialogElement | null>(null)
const fileInput=ref<HTMLInputElement|null>(null)

useNativeDialog(dialogEl, () => props.open)

function close() {
  emit('update:open', false)
}

const accents:{id:AccentColor;label:string;color:string}[]=[{id:'pink',label:'Pink',color:'#E75480'},{id:'coral',label:'Coral',color:'#D96C6C'},{id:'blue',label:'Blue',color:'#5F82C9'},{id:'teal',label:'Teal',color:'#4E9C9A'},{id:'violet',label:'Violet',color:'#9368B7'},{id:'green',label:'Green',color:'#56A37B'}]
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
