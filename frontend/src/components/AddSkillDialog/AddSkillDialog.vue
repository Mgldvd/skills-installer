<template>
  <dialog ref="dialogEl" class="add-skill-dialog" @close="emit('update:open', false)" @click="handleBackdropClick">
    <form class="add-skill-dialog__form" @submit.prevent="handleSubmit">
      <header class="add-skill-dialog__header">
        <div class="add-skill-dialog__heading">
          <h2 class="add-skill-dialog__title">Add Skill</h2>
          <a href="https://skills.sh" target="_blank" rel="noopener noreferrer">Skills.sh</a>
          <p>Add a Skill from its Skills.sh address.</p>
        </div>
        <div class="add-skill-dialog__header-actions">
          <button type="button" class="add-skill-dialog__btn" @click="close">Cancel</button>
          <button type="submit" class="add-skill-dialog__btn add-skill-dialog__btn--primary" :disabled="!canSubmit">
            Add Skill
          </button>
          <CloseButton aria-label="Close Add Skill" @click="close" />
        </div>
      </header>

      <div class="add-skill-dialog__body">
        <label class="add-skill-dialog__field add-skill-dialog__field--wide">
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

        <p v-if="urlError" class="add-skill-dialog__error add-skill-dialog__wide" role="alert">{{ urlError }}</p>
        <div v-else-if="preview" class="add-skill-dialog__preview add-skill-dialog__wide">
          <p>Detected skill: <strong>{{ preview.skillName }}</strong></p>
          <p>Repository: <strong>{{ preview.owner }}/{{ preview.repository }}</strong></p>
        </div>
        <p v-if="duplicateSkill" class="add-skill-dialog__duplicate add-skill-dialog__wide" role="alert">
          This Skill is already added as <strong>{{ duplicateSkill.displayName }}</strong>.
        </p>

        <label class="add-skill-dialog__field">
          <span>Display Name</span>
          <input
            v-model="displayName"
            type="text"
            class="add-skill-dialog__input"
            :placeholder="preview?.skillName ?? 'Display name'"
            @input="displayNameEdited = true"
          />
        </label>

        <label class="add-skill-dialog__field">
          <span>Description</span>
          <textarea v-model="description" class="add-skill-dialog__input add-skill-dialog__textarea" rows="2" @input="descriptionEdited = true" />
        </label>

        <div class="add-skill-dialog__checkboxes add-skill-dialog__wide">
          <label class="add-skill-dialog__checkbox">
            <input v-model="preselected" type="checkbox" />
            <span>Preselected by default</span>
          </label>
          <label class="add-skill-dialog__checkbox">
            <input v-model="enabled" type="checkbox" />
            <span>Enabled</span>
          </label>
        </div>

        <p v-if="submitError" class="add-skill-dialog__error add-skill-dialog__wide" role="alert">{{ submitError }}</p>
      </div>
    </form>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import { useNativeDialog } from '../../composables/useNativeDialog'
import * as backend from '../../services/backend'
import type { ParsedSkillSource, Skill, SkillGroup } from '../../types'
import CloseButton from '../CloseButton/CloseButton.vue'

const props = defineProps<{
  open: boolean
  groups?: SkillGroup[]
  skills?: Skill[]
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
const displayNameEdited = ref(false)
const descriptionEdited = ref(false)

let debounceHandle: ReturnType<typeof setTimeout> | null = null
let previewRequest = 0

function resetForm() {
  previewRequest += 1
  if (debounceHandle) clearTimeout(debounceHandle)
  url.value = ''
  displayName.value = ''
  description.value = ''
  groupId.value = props.defaultGroupId ?? props.groups?.[0]?.id ?? null
  preselected.value = false
  enabled.value = true
  preview.value = null
  urlError.value = null
  displayNameEdited.value = false
  descriptionEdited.value = false
}

useNativeDialog(dialogEl, () => props.open, resetForm)

function handleUrlInput() {
  const request = ++previewRequest
  if (debounceHandle) clearTimeout(debounceHandle)
  debounceHandle = setTimeout(async () => {
    if (!url.value.trim()) {
      preview.value = null
      urlError.value = null
      if (!displayNameEdited.value) displayName.value = ''
      if (!descriptionEdited.value) description.value = ''
      return
    }
    try {
      const detected = await backend.previewSkillUrl(url.value.trim())
      if (request !== previewRequest) return
      preview.value = detected
      if (!displayNameEdited.value) displayName.value = detected.skillName
      if (!descriptionEdited.value) description.value = `${detected.owner}/${detected.repository}`
      urlError.value = null
    } catch (error) {
      if (request !== previewRequest) return
      preview.value = null
      urlError.value = error instanceof Error ? error.message : String(error)
    }
  }, 300)
}

const duplicateSkill = computed(() => {
  if (!preview.value) return null
  return props.skills?.find((skill) => skill.skillsUrl === preview.value?.canonicalUrl) ?? null
})
const canSubmit = computed(() => Boolean(preview.value && groupId.value && !urlError.value && !duplicateSkill.value))

function close() {
  emit('update:open', false)
}

function handleBackdropClick(event: MouseEvent) {
  if (event.target === dialogEl.value) close()
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
