import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import GroupBadge from '../src/components/GroupBadge/GroupBadge.vue'

describe('GroupBadge', () => {
  it('renders a colored dot reflecting the group color', () => {
    const wrapper = mount(GroupBadge, { props: { name: 'Frontend', color: '#E75480' } })
    const dot = wrapper.find('.group-badge__dot')
    expect(dot.attributes('style')).toContain('background-color: rgb(231, 84, 128)')
  })

  it('renders the group name as text, not color-only', () => {
    const wrapper = mount(GroupBadge, { props: { name: 'Frontend', color: '#E75480' } })
    expect(wrapper.text()).toContain('Frontend')
  })

  it('updates its dot color reactively when the color prop changes', async () => {
    const wrapper = mount(GroupBadge, { props: { name: 'Frontend', color: '#E75480' } })
    await wrapper.setProps({ color: '#4E9A70' })
    expect(wrapper.find('.group-badge__dot').attributes('style')).toContain('background-color: rgb(78, 154, 112)')
  })
})
