import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../src/services/backend', () => ({
  getApplicationState: vi.fn(),
  refresh: vi.fn(),
  addSkill: vi.fn(),
  updateSkill: vi.fn(),
  deleteSkill: vi.fn(),
  createGroup: vi.fn(),
  updateGroup: vi.fn(),
  deleteGroup: vi.fn(),
  getPreferences: vi.fn(),
  updatePreferences: vi.fn(),
  getDependencyStatus: vi.fn(),
  installSkills: vi.fn(),
  cancelInstallation: vi.fn(),
  previewSkillUrl: vi.fn(),
}))

import App from '../src/App.vue'
import { useAppState } from '../src/composables/useAppState'
import * as backend from '../src/services/backend'
import { defaultPreferences } from '../src/types'
import { makeGroup, makeSkill } from './fixtures'
import { resetState } from './test-utils'

function mockLoadedApp() {
  vi.mocked(backend.getApplicationState).mockResolvedValue({
    version: 1,
    defaults: { agent: null, copy: true, scope: 'project' },
    groups: [makeGroup({ id: 'other', name: 'Other' }), makeGroup({ id: 'testing', name: 'Testing' })],
    skills: [
      makeSkill({ id: 'a', displayName: 'Alpha', groupId: 'testing', preselected: true }),
      makeSkill({ id: 'b', displayName: 'Beta', groupId: 'other', preselected: false }),
    ],
    sourcePath: null,
    isEmbeddedDefault: true,
    projectRoot: '/home/user/project',
  })
  vi.mocked(backend.getPreferences).mockResolvedValue(defaultPreferences())
  vi.mocked(backend.getDependencyStatus).mockResolvedValue({
    available: true,
    source: 'installedExecutable',
    executablePath: '/usr/bin/skills',
    version: '1.0.0',
    detail: null,
  })
}

describe('App', () => {
  const state = useAppState()

  beforeEach(() => {
    resetState(state)
    vi.clearAllMocks()
  })

  it('selects exactly the preselected skills on load', async () => {
    mockLoadedApp()
    mount(App)
    await flushPromises()

    expect(state.selectedSkillIds).toEqual(new Set(['a']))
  })

  it('keeps selected cards first, ordered by Pack and then alphabetically', async () => {
    mockLoadedApp()
    const wrapper = mount(App)
    await flushPromises()
    state.tags = [
      { id: 'frontend', name: 'Frontend', color: '#3B82F6', order: 10, enabled: true },
      { id: 'testing', name: 'Testing', color: '#22C55E', order: 20, enabled: true },
    ]
    state.skills = [
      makeSkill({ id: 'z', displayName: 'Zulu', tags: ['testing'] }),
      makeSkill({ id: 'b', displayName: 'Beta', tags: ['frontend'] }),
      makeSkill({ id: 'a', displayName: 'Alpha', tags: ['frontend'] }),
      makeSkill({ id: 'u', displayName: 'Unselected', tags: ['frontend'] }),
    ]
    state.selectedSkillIds = new Set(['z', 'b', 'a'])
    await nextTick()

    expect(wrapper.findAll('.skill-card__title').map((title) => title.text())).toEqual(['Alpha', 'Beta', 'Zulu', 'Unselected'])
  })

  it('moves a newly selected card into the leading selected block', async () => {
    mockLoadedApp()
    const wrapper = mount(App)
    await flushPromises()
    state.skills = state.skills.map((skill) => skill.id === 'b' ? { ...skill, displayName: 'Aardvark' } : skill)
    await nextTick()
    const cardsBefore = wrapper.findAll('.skill-card__title').map((title) => title.text())
    expect(cardsBefore).toEqual(['Alpha', 'Aardvark'])

    await wrapper.findAll('.skill-card__selection-surface')[1].trigger('click')
    await nextTick()
    expect(wrapper.findAll('.skill-card__title').map((title) => title.text())).toEqual(['Aardvark', 'Alpha'])
    expect(wrapper.findAll('.skill-card')[0].classes()).toContain('is-selected')
  })

  it('opens Add Skill directly from the green footer action', async () => {
    mockLoadedApp()
    const wrapper = mount(App)
    await flushPromises()

    const addButton=wrapper.findAll('.app-shell__footer-btn').find(button=>button.text()==='Add Skill')!
    expect(addButton.classes()).toContain('app-shell__footer-btn--add')
    await addButton.trigger('click')
    await flushPromises()
    expect(wrapper.find('.add-skill-dialog').attributes('open')).toBeDefined()
  })

  it('"Clear" empties the current selection', async () => {
    mockLoadedApp()
    const wrapper = mount(App)
    await flushPromises()

    const clearButton = wrapper.get('.skill-toolbar__clear')
    await clearButton.trigger('click')

    expect(state.selectedSkillIds.size).toBe(0)
    expect(wrapper.findAll('.app-shell__footer-btn').some((button) => button.text() === 'Clear')).toBe(false)
  })

  it('clicking Install Selected opens the install confirmation dialog when confirmBeforeInstall is enabled', async () => {
    mockLoadedApp()
    const wrapper = mount(App)
    await flushPromises()

    const installButton = wrapper.findAll('.app-shell__footer-btn').find((b) => b.text() === 'Install Selected')
    expect(wrapper.get('.app-shell__footer-actions .app-shell__selected-count').text()).toBe('1 selected')
    await installButton!.trigger('click')
    await flushPromises()

    const dialog = wrapper.find('.install-confirm-dialog')
    expect((dialog.element as HTMLDialogElement).open).toBe(true)
  })

  it('renders a non-color-only Skills CLI status indicator reflecting dependency availability', async () => {
    mockLoadedApp()
    const wrapper = mount(App)
    await flushPromises()

    expect(wrapper.text()).toContain('Skills CLI')
    expect(wrapper.text()).toContain('Ready')
  })

  it('surfaces a load error as a toast rather than a silent failure', async () => {
    vi.mocked(backend.getApplicationState).mockRejectedValue(new Error('disk on fire'))
    vi.mocked(backend.getPreferences).mockResolvedValue(defaultPreferences())
    vi.mocked(backend.getDependencyStatus).mockResolvedValue({
      available: false,
      source: 'unavailable',
      executablePath: null,
      version: null,
      detail: 'not found',
    })

    const wrapper = mount(App)
    await flushPromises()

    expect(wrapper.text()).toContain('disk on fire')
  })
})
