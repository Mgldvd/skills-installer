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
})
