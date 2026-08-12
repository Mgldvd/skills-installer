import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import AddSkillDialog from '../src/components/AddSkillDialog/AddSkillDialog.vue'
import { makeGroup } from './fixtures'

vi.mock('../src/services/backend', () => ({
  previewSkillUrl: vi.fn(),
}))

import * as backend from '../src/services/backend'

describe('AddSkillDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows a detected-skill preview once the URL resolves', async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: 'https://www.skills.sh/mattpocock/skills/triage',
      owner: 'mattpocock',
      repository: 'skills',
      skillName: 'triage',
      repositoryUrl: 'https://github.com/mattpocock/skills',
    })

    const wrapper = mount(AddSkillDialog, {
      props: { open: true, groups: [makeGroup()] },
    })

    await wrapper.find('input[type="url"]').setValue('https://www.skills.sh/mattpocock/skills/triage')
    await wrapper.find('input[type="url"]').trigger('input')
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled())
    await flushPromises()

    expect(wrapper.text()).toContain('triage')
    expect(wrapper.text()).toContain('mattpocock/skills')
    expect(wrapper.find('.add-skill-dialog__error').exists()).toBe(false)
  })

  it('shows a validation error and disables submit for a rejected URL', async () => {
    vi.mocked(backend.previewSkillUrl).mockRejectedValue(new Error('expected https://www.skills.sh/<owner>/<repository>/<skill-name>'))

    const wrapper = mount(AddSkillDialog, {
      props: { open: true, groups: [makeGroup()] },
    })

    await wrapper.find('input[type="url"]').setValue('https://example.com/not-a-skill')
    await wrapper.find('input[type="url"]').trigger('input')
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled())
    await flushPromises()

    expect(wrapper.find('.add-skill-dialog__error').exists()).toBe(true)
    const submitButton = wrapper.find('button[type="submit"]')
    expect(submitButton.attributes('disabled')).toBeDefined()
  })

  it('emits submit with the derived group and trimmed fields', async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: 'https://www.skills.sh/mattpocock/skills/triage',
      owner: 'mattpocock',
      repository: 'skills',
      skillName: 'triage',
      repositoryUrl: 'https://github.com/mattpocock/skills',
    })

    const wrapper = mount(AddSkillDialog, {
      props: { open: true, groups: [makeGroup({ id: 'testing' })] },
    })

    await wrapper.find('input[type="url"]').setValue('https://www.skills.sh/mattpocock/skills/triage')
    await wrapper.find('input[type="url"]').trigger('input')
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled())
    await flushPromises()

    await wrapper.find('form').trigger('submit')

    const emitted = wrapper.emitted('submit')
    expect(emitted).toBeTruthy()
    expect(emitted![0][0]).toMatchObject({
      url: 'https://www.skills.sh/mattpocock/skills/triage',
      groupId: 'testing',
    })
  })
})
