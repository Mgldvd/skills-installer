import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import GroupEditBar from '../src/components/GroupEditBar/GroupEditBar.vue'
import type { GroupChip } from '../src/components/GroupEditBar/GroupEditBar.vue'

function groups(): GroupChip[] {
  return [
    { id: 'frontend', name: 'Frontend', color: '#E75480', count: 2 },
    { id: 'testing', name: 'Testing', color: '#6B82D9', count: 1 },
  ]
}

describe('GroupEditBar', () => {
  it('renders one chip per group with its color and count', () => {
    const wrapper = mount(GroupEditBar, { props: { groups: groups() } })
    const labels = wrapper.findAll('.group-edit-bar__label').map((el) => el.text())
    expect(labels).toEqual(['Frontend', 'Testing'])
    expect(wrapper.find('.group-edit-bar__dot').attributes('style')).toContain('background-color: rgb(231, 84, 128)')
  })

  it('emits editGroup with the group id when its edit affordance is clicked', async () => {
    const wrapper = mount(GroupEditBar, { props: { groups: groups() } })
    await wrapper.findAll('.group-edit-bar__edit-btn')[0].trigger('click')
    expect(wrapper.emitted('editGroup')).toEqual([['frontend']])
  })

  it('renders nothing when there are no groups', () => {
    const wrapper = mount(GroupEditBar, { props: { groups: [] } })
    expect(wrapper.find('.group-edit-bar').exists()).toBe(false)
  })
})
