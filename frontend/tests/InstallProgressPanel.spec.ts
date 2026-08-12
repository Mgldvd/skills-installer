import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import InstallProgressPanel from '../src/components/InstallProgressPanel/InstallProgressPanel.vue'
import type { InstallationState } from '../src/composables/useAppState'

function baseInstallation(overrides: Partial<InstallationState> = {}): InstallationState {
  return {
    isInstalling: false,
    currentSkillId: null,
    currentIndex: 0,
    total: 0,
    outputLines: [],
    perSkillStatus: {},
    displayNames: {},
    result: null,
    error: null,
    ...overrides,
  }
}

describe('InstallProgressPanel', () => {
  it('shows an in-progress headline with current/total counts while installing', () => {
    const wrapper = mount(InstallProgressPanel, {
      props: { installation: baseInstallation({ isInstalling: true, currentIndex: 1, total: 3 }) },
    })
    expect(wrapper.text()).toContain('Installing 1/3')
  })

  it('renders per-skill status with a checkmark for success and an X for failure', () => {
    const wrapper = mount(InstallProgressPanel, {
      props: {
        installation: baseInstallation({
          perSkillStatus: { a: 'installed', b: 'failed' },
          displayNames: { a: 'Skill A', b: 'Skill B' },
        }),
      },
    })
    const text = wrapper.text()
    expect(text).toContain('✓')
    expect(text).toContain('✗')
    expect(text).toContain('Skill A')
    expect(text).toContain('Skill B')
  })

  it('streams stripped output lines grouped under their skill', () => {
    const wrapper = mount(InstallProgressPanel, {
      props: {
        installation: baseInstallation({
          perSkillStatus: { a: 'installed' },
          displayNames: { a: 'Skill A' },
          outputLines: [
            { skillId: 'a', line: 'Cloning repository...', stream: 'stdout' },
            { skillId: 'a', line: 'Done.', stream: 'stdout' },
          ],
        }),
      },
    })
    expect(wrapper.text()).toContain('Cloning repository...')
    expect(wrapper.text()).toContain('Done.')
  })

  it('distinguishes executed commands and stderr in the live output', () => {
    const wrapper = mount(InstallProgressPanel, {
      props: {
        installation: baseInstallation({
          perSkillStatus: { a: 'failed' },
          displayNames: { a: 'Skill A' },
          outputLines: [
            { skillId: 'a', line: '$ npx skills add owner/repo --skill a', stream: 'command' },
            { skillId: 'a', line: 'Error: command exited with code 1', stream: 'stderr' },
          ],
        }),
      },
    })

    expect(wrapper.find('.is-command').text()).toContain('npx skills add')
    expect(wrapper.find('.is-stderr').text()).toContain('exited with code 1')
  })

  it('renders the final requested/installed/already-installed/failed summary', () => {
    const wrapper = mount(InstallProgressPanel, {
      props: {
        installation: baseInstallation({
          result: { requested: 4, installed: 2, alreadyInstalled: 1, failed: 1, cancelled: false, perSkill: [] },
        }),
      },
    })
    expect(wrapper.text()).toContain('4 requested')
    expect(wrapper.text()).toContain('2 installed')
    expect(wrapper.text()).toContain('1 already installed')
    expect(wrapper.text()).toContain('1 failed')
  })

  it('renders a persistent error message, not just a transient state', () => {
    const wrapper = mount(InstallProgressPanel, {
      props: { installation: baseInstallation({ error: 'The Skills CLI was not found.' }) },
    })
    expect(wrapper.find('[role="alert"]').text()).toBe('The Skills CLI was not found.')
  })

  it('shows Cancel while installing and a dismiss control once finished', async () => {
    const installing = mount(InstallProgressPanel, { props: { installation: baseInstallation({ isInstalling: true }) } })
    expect(installing.find('.install-progress-panel__cancel').exists()).toBe(true)

    const finished = mount(InstallProgressPanel, {
      props: { installation: baseInstallation({ result: { requested: 1, installed: 1, alreadyInstalled: 0, failed: 0, cancelled: false, perSkill: [] } }) },
    })
    const dismiss = finished.get('[aria-label="Dismiss installation output"]')
    expect(dismiss.classes()).toContain('close-button')
    await dismiss.trigger('click')
    expect(finished.emitted('dismiss')).toBeTruthy()
  })
})
