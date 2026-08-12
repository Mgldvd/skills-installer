import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import CloseButton from '../src/components/CloseButton/CloseButton.vue'

describe('CloseButton', () => {
  it('provides a shared accessible close control', async () => {
    const wrapper = mount(CloseButton, { props: { ariaLabel: 'Close dialog' } })

    expect(wrapper.element.tagName).toBe('BUTTON')
    expect(wrapper.attributes('type')).toBe('button')
    expect(wrapper.attributes('aria-label')).toBe('Close dialog')
    expect(wrapper.find('svg[aria-hidden="true"]').exists()).toBe(true)
    await wrapper.trigger('click')
    expect(wrapper.emitted('click')).toHaveLength(1)
  })
})
