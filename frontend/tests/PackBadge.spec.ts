import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import PackBadge from '../src/components/PackBadge/PackBadge.vue'

describe('PackBadge', () => {
  it('uses one static structure with the configured Pack color', () => {
    const wrapper = mount(PackBadge, { props: { name: 'Frontend', color: '#3B82F6', compact: true } })

    expect(wrapper.element.tagName).toBe('SPAN')
    expect(wrapper.get('.pack-badge__label').text()).toBe('Frontend')
    expect(wrapper.get('.pack-badge__dot').attributes('style')).toBeUndefined()
    expect(wrapper.attributes('style')).toContain('--pack-color: #3B82F6')
    expect(wrapper.attributes('aria-pressed')).toBeUndefined()
  })

  it('uses button semantics, aria-pressed, and a non-color check when selected', async () => {
    const wrapper = mount(PackBadge, { props: { name: 'Frontend', color: '#3B82F6', interactive: true, selected: true } })

    expect(wrapper.element.tagName).toBe('BUTTON')
    expect(wrapper.attributes('type')).toBe('button')
    expect(wrapper.attributes('aria-pressed')).toBe('true')
    expect(wrapper.find('.pack-badge__state svg').exists()).toBe(true)
    await wrapper.trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
  })
})
