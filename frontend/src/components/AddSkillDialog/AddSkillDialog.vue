<template>
  <dialog ref="dialogEl" class="add-skill-dialog" @close="emit('update:open', false)">
    <form class="add-skill-dialog__form" @submit.prevent="handleSubmit">
      <h2 class="add-skill-dialog__title">Add Skill</h2>

      <label class="add-skill-dialog__field">
        <span>Skills.sh URL</span>
        <input
          v-model="url"
          type="url"
          required
          class="add-skill-dialog__input"
          placeholder="https://www.skills.sh/owner/repository/skill"
          @input="handleUrlInput"
        />
      </label>

      <p v-if="urlError" class="add-skill-dialog__error" role="alert">{{ urlError }}</p>
      <div v-else-if="preview" class="add-skill-dialog__preview">
        <p>Detected skill: <strong>{{ preview.skillName }}</strong></p>
        <p>Repository: <strong>{{ preview.owner }}/{{ preview.repository }}</strong></p>
      </div>

      <label class="add-skill-dialog__field">
        <span>Display Name</span>
        <input
          v-model="displayName"
          type="text"
          class="add-skill-dialog__input"
          :placeholder="preview?.skillName ?? 'Display name'"
        />
      </label>

      <label class="add-skill-dialog__field">
        <span>Description</span>
        <textarea v-model="description" class="add-skill-dialog__input add-skill-dialog__textarea" rows="2" />
      </label>

      <div class="add-skill-dialog__checkboxes">
        <label class="add-skill-dialog__checkbox">
          <input v-model="preselected" type="checkbox" />
          <span>Preselected by default</span>
        </label>
        <label class="add-skill-dialog__checkbox">
          <input v-model="enabled" type="checkbox" />
          <span>Enabled</span>
        </label>
      </div>

      <p v-if="submitError" class="add-skill-dialog__error" role="alert">{{ submitError }}</p>

      <div class="add-skill-dialog__actions">
        <button type="button" class="add-skill-dialog__btn" @click="close">Cancel</button>
        <button type="submit" class="add-skill-dialog__btn add-skill-dialog__btn--primary" :disabled="!canSubmit">
          Add Skill
        </button>
      </div>
    </form>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import { useNativeDialog } from '../../composables/useNativeDialog'
import * as backend from '../../services/backend'
import type { ParsedSkillSource, SkillGroup } from '../../types'

const props = defineProps<{
  open: boolean
  groups?: SkillGroup[]
  defaultGroupId?: string | null
  submitError?: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  submit: [payload: backend.AddSkillArgs]
}>()

const dialogEl = ref<HTMLDialogElement | null>(null)
const url = ref('')
const displayName = ref('')
const description = ref('')
const groupId = ref<string | null>(null)
const preselected = ref(false)
const enabled = ref(true)
const preview = ref<ParsedSkillSource | null>(null)
const urlError = ref<string | null>(null)

let debounceHandle: ReturnType<typeof setTimeout> | null = null

function resetForm() {
  url.value = ''
  displayName.value = ''
  description.value = ''
  groupId.value = props.defaultGroupId ?? props.groups?.[0]?.id ?? null
  preselected.value = false
  enabled.value = true
  preview.value = null
  urlError.value = null
}

useNativeDialog(dialogEl, () => props.open, resetForm)

function handleUrlInput() {
  if (debounceHandle) clearTimeout(debounceHandle)
  debounceHandle = setTimeout(async () => {
    if (!url.value.trim()) {
      preview.value = null
      urlError.value = null
      return
    }
    try {
      preview.value = await backend.previewSkillUrl(url.value.trim())
      urlError.value = null
    } catch (error) {
      preview.value = null
      urlError.value = error instanceof Error ? error.message : String(error)
    }
  }, 300)
}

const canSubmit = computed(() => Boolean(preview.value && groupId.value && !urlError.value))

function close() {
  emit('update:open', false)
}

function handleSubmit() {
  if (!canSubmit.value || !groupId.value) return
  emit('submit', {
    url: url.value.trim(),
    displayName: displayName.value.trim() || undefined,
    description: description.value.trim() || undefined,
    groupId: groupId.value,
    tags: [],
    preselected: preselected.value,
    enabled: enabled.value,
  })
}

</script>

<style scoped lang="scss" src="./AddSkillDialog.scss"></style>
