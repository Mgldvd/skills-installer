import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { describe, expect, it } from 'vitest'

import PreferencesDialog from '../src/components/PreferencesDialog/PreferencesDialog.vue'
import { defaultPreferences } from '../src/types'

describe('PreferencesDialog', () => {
  it('manages the Local Skill Source from Preferences', async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: { ...defaultPreferences(), localSourcePath: '/old/source' } },
    })
    await nextTick()

    const input = wrapper.find('#local-skill-source')
    expect(input.element).toHaveProperty('value', '/old/source')
    await input.setValue('/new/source')
    await wrapper.findAll('.preferences-dialog__source-actions button')[0].trigger('click')

    expect(wrapper.emitted('updateLocalSource')).toEqual([['/new/source']])
  })

  it('offers a refresh action for the configured local source', async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: defaultPreferences() },
    })

    await wrapper.findAll('.preferences-dialog__source-actions button')[1].trigger('click')
    expect(wrapper.emitted('refreshLocalSource')).toHaveLength(1)
  })

  it('offers Linux CLI installation', async () => {
    const wrapper = mount(PreferencesDialog, {
      props: { open: true, preferences: defaultPreferences() },
    })

    await wrapper.get('.preferences-dialog__cli-button').trigger('click')
    expect(wrapper.emitted('installCli')).toHaveLength(1)
  })

  it('updates the compact card preference', async () => {
    const wrapper = mount(PreferencesDialog, { props: { open: true, preferences: defaultPreferences() } })
    const checkbox = wrapper.findAll('.preferences-dialog__checkbox').find((item) => item.text().includes('Compact cards'))!
    await checkbox.get('input').setValue(true)
    expect(wrapper.emitted('update')).toContainEqual([{ compactCards: true }])
  })
})
