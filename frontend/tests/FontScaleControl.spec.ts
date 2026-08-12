import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import FontScaleControl from '../src/components/FontScaleControl/FontScaleControl.vue'
import { FONT_SCALE_PRESETS } from '../src/types'

describe('FontScaleControl', () => {
  it('compact variant steps to the next preset on A+', async () => {
    const wrapper = mount(FontScaleControl, { props: { modelValue: 1.0, variant: 'compact' } })
    const [decrease, increase] = wrapper.findAll('.font-scale-control__step')

    await increase.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[1.1]])
    void decrease
  })

  it('compact variant steps to the previous preset on A−', async () => {
    const wrapper = mount(FontScaleControl, { props: { modelValue: 1.1, variant: 'compact' } })
    const [decrease] = wrapper.findAll('.font-scale-control__step')

    await decrease.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[1.0]])
  })

  it('disables A− at the smallest preset and A+ at the largest', () => {
    const smallest = mount(FontScaleControl, { props: { modelValue: FONT_SCALE_PRESETS[0].value, variant: 'compact' } })
    expect(smallest.findAll('.font-scale-control__step')[0].attributes('disabled')).toBeDefined()

    const largest = mount(FontScaleControl, {
      props: { modelValue: FONT_SCALE_PRESETS[FONT_SCALE_PRESETS.length - 1].value, variant: 'compact' },
    })
    expect(largest.findAll('.font-scale-control__step')[1].attributes('disabled')).toBeDefined()
  })

  it('full variant renders every documented preset as a labeled option', () => {
    const wrapper = mount(FontScaleControl, { props: { modelValue: 1.0, variant: 'full' } })
    const labels = wrapper.findAll('.font-scale-control__preset').map((el) => el.text())
    expect(labels).toEqual(FONT_SCALE_PRESETS.map((p) => p.label))
  })

  it('full variant emits the exact preset value when clicked', async () => {
    const wrapper = mount(FontScaleControl, { props: { modelValue: 1.0, variant: 'full' } })
    const buttons = wrapper.findAll('.font-scale-control__preset')
    await buttons[3].trigger('click')
    expect(wrapper.emitted('update:modelValue')).toEqual([[FONT_SCALE_PRESETS[3].value]])
  })
})
