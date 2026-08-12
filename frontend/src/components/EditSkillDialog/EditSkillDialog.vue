<template>
  <dialog ref="dialogEl" class="edit-skill-dialog" @close="emit('update:open', false)">
    <form v-if="skill" class="edit-skill-dialog__form" @submit.prevent="handleSubmit">
      <h2 class="edit-skill-dialog__title">Edit Skill</h2>

      <label v-if="!skill.local" class="edit-skill-dialog__field">
        <span>Skills.sh URL</span>
        <input v-model="url" type="url" class="edit-skill-dialog__input" @input="handleUrlInput" />
      </label>
      <p v-if="urlError" class="edit-skill-dialog__error" role="alert">{{ urlError }}</p>
      <div v-else-if="preview" class="edit-skill-dialog__preview">
        <p>Detected skill: <strong>{{ preview.skillName }}</strong></p>
        <p>Repository: <strong>{{ preview.owner }}/{{ preview.repository }}</strong></p>
      </div>

      <label class="edit-skill-dialog__field">
        <span>Display Name</span>
        <input v-model="displayName" type="text" required class="edit-skill-dialog__input" />
      </label>

      <label class="edit-skill-dialog__field">
        <span>Description</span>
        <textarea v-model="description" class="edit-skill-dialog__input edit-skill-dialog__textarea" rows="2" />
      </label>

      <div class="edit-skill-dialog__checkboxes">
        <label class="edit-skill-dialog__checkbox">
          <input v-model="preselected" type="checkbox" />
          <span>Preselected by default</span>
        </label>
        <label class="edit-skill-dialog__checkbox">
          <input v-model="enabled" type="checkbox" />
          <span>Enabled</span>
        </label>
      </div>

      <p v-if="submitError" class="edit-skill-dialog__error" role="alert">{{ submitError }}</p>

      <div class="edit-skill-dialog__actions">
        <button type="button" class="edit-skill-dialog__btn" @click="close">Cancel</button>
        <button type="submit" class="edit-skill-dialog__btn edit-skill-dialog__btn--primary" :disabled="!canSubmit">
          Save Changes
        </button>
      </div>
    </form>
  </dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

import { useNativeDialog } from '../../composables/useNativeDialog'
import * as backend from '../../services/backend'
import type { ParsedSkillSource, Skill } from '../../types'

const props = defineProps<{
  open: boolean
  skill: Skill | null
  submitError?: string | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  submit: [payload: backend.UpdateSkillArgs]
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
const urlChanged = ref(false)

let debounceHandle: ReturnType<typeof setTimeout> | null = null

function loadFromSkill() {
  const skill = props.skill
  if (!skill) return
  url.value = skill.skillsUrl
  displayName.value = skill.displayName
  description.value = skill.description
  groupId.value = skill.groupId
  preselected.value = skill.preselected
  enabled.value = skill.enabled
  preview.value = null
  urlError.value = null
  urlChanged.value = false
}

useNativeDialog(dialogEl, () => props.open, loadFromSkill)

function handleUrlInput() {
  urlChanged.value = true
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

const canSubmit = computed(() => displayName.value.trim().length > 0 && !urlError.value && Boolean(groupId.value))

function close() {
  emit('update:open', false)
}

function handleSubmit() {
  if (!canSubmit.value || !props.skill || !groupId.value) return
  emit('submit', {
    skillId: props.skill.id,
    url: urlChanged.value ? url.value.trim() : undefined,
    displayName: displayName.value.trim(),
    description: description.value.trim(),
    groupId: groupId.value,
    preselected: preselected.value,
    enabled: enabled.value,
  })
}

</script>

<style scoped lang="scss" src="./EditSkillDialog.scss"></style>
