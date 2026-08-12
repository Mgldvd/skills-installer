import { flushPromises, mount } from '@vue/test-utils'
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
