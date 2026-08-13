import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import AddSkillDialog from '../src/components/AddSkillDialog/AddSkillDialog.vue'
import { makeGroup, makeSkill } from './fixtures'

vi.mock('../src/services/backend', () => ({
  previewSkillUrl: vi.fn(),
  previewPackUrl: vi.fn(),
}))

import * as backend from '../src/services/backend'

describe('AddSkillDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('places a clickable Skills.sh link beside the title', () => {
    const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()] } })
    const heading = wrapper.get('.add-skill-dialog__heading')
    const link = heading.get('a')
    expect(heading.get('h2').text()).toBe('Add Skill')
    expect(link.text()).toBe('Skills.sh')
    expect(link.attributes()).toMatchObject({ href: 'https://skills.sh', target: '_blank', rel: 'noopener noreferrer' })
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
    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe('triage')
    expect((wrapper.find('textarea').element as HTMLTextAreaElement).value).toBe('mattpocock/skills')
    expect(wrapper.find('.add-skill-dialog__error').exists()).toBe(false)
  })

  it('detects a Pack URL and emits a catalog import instead of a Skill install', async () => {
    vi.mocked(backend.previewPackUrl).mockResolvedValue({
      canonicalUrl: 'https://skills.sh/p/example123',
      suggestedName: 'Pack example123',
      skills: [
        { name: 'bash-defensive-patterns', description: 'Defensive Bash.' },
        { name: 'bash-scripting', description: 'Bash workflows.' },
      ],
    })
    const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup({ id: 'other' })] } })

    await wrapper.find('input[type="url"]').setValue('https://skills.sh/p/example123')
    await vi.waitFor(() => expect(backend.previewPackUrl).toHaveBeenCalled())
    await flushPromises()

    expect(wrapper.text()).toContain('Detected Pack: 2 Skills')
    expect(wrapper.text()).toContain('Nothing is installed into the current project')
    expect(wrapper.get('button[type="submit"]').text()).toBe('Import Pack')
    await wrapper.find('form').trigger('submit')
    expect(wrapper.emitted('importPack')?.[0]?.[0]).toMatchObject({
      url: 'https://skills.sh/p/example123',
      packName: 'Pack example123',
      groupId: 'other',
    })
    expect(wrapper.emitted('submit')).toBeUndefined()
  })

  it('keeps detected suggestions editable', async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: 'https://www.skills.sh/mattpocock/skills/triage',
      owner: 'mattpocock', repository: 'skills', skillName: 'triage', repositoryUrl: 'https://github.com/mattpocock/skills',
    })
    const wrapper = mount(AddSkillDialog, { props: { open: true, groups: [makeGroup()] } })

    await wrapper.find('input[type="url"]').setValue('https://www.skills.sh/mattpocock/skills/triage')
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled())
    await flushPromises()
    await wrapper.find('input[type="text"]').setValue('Issue Triage')
    await wrapper.find('textarea').setValue('Custom description')

    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe('Issue Triage')
    expect((wrapper.find('textarea').element as HTMLTextAreaElement).value).toBe('Custom description')
  })

  it('warns about an existing canonical Skill and blocks duplicate submission', async () => {
    vi.mocked(backend.previewSkillUrl).mockResolvedValue({
      canonicalUrl: 'https://www.skills.sh/mattpocock/skills/triage',
      owner: 'mattpocock', repository: 'skills', skillName: 'triage', repositoryUrl: 'https://github.com/mattpocock/skills',
    })
    const wrapper = mount(AddSkillDialog, {
      props: {
        open: true,
        groups: [makeGroup()],
        skills: [makeSkill({ displayName: 'Existing Triage', skillsUrl: 'https://www.skills.sh/mattpocock/skills/triage' })],
      },
    })

    await wrapper.find('input[type="url"]').setValue('https://www.skills.sh/mattpocock/skills/triage')
    await vi.waitFor(() => expect(backend.previewSkillUrl).toHaveBeenCalled())
    await flushPromises()

    expect(wrapper.get('.add-skill-dialog__duplicate').text()).toContain('Existing Triage')
    expect(wrapper.get('button[type="submit"]').attributes('disabled')).toBeDefined()
    await wrapper.find('form').trigger('submit')
    expect(wrapper.emitted('submit')).toBeUndefined()
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
      displayName: 'triage',
      description: 'mattpocock/skills',
      groupId: 'testing',
    })
  })
})
