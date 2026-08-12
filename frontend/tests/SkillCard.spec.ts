import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import SkillCard from '../src/components/SkillCard/SkillCard.vue'
import { makeSkill } from './fixtures'

describe('SkillCard', () => {
  it('renders the skill name and description in dedicated sections', () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ displayName: 'Issue Triage', description: 'Helps triage issues.' }), selected: false },
    })

    expect(wrapper.find('.skill-card__title').text()).toBe('Issue Triage')
    expect(wrapper.find('.skill-card__description-text').text()).toBe('Helps triage issues.')
  })

  it('selects from the title and dead card surface without hijacking description', async () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ id: 'triage' }), selected: false } })

    await wrapper.find('.skill-card__description').trigger('click')
    expect(wrapper.emitted('toggle')).toBeUndefined()
    expect(wrapper.emitted('description')).toEqual([['triage']])

    await wrapper.find('.skill-card__selection-surface').trigger('click')
    expect(wrapper.emitted('toggle')).toEqual([['triage']])
  })

  it('exposes selection through aria-pressed and a clean top-right ribbon', () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill(), selected: true } })
    const selection = wrapper.find('.skill-card__selection-surface')

    expect(wrapper.classes()).toContain('is-selected')
    expect(selection.attributes('aria-pressed')).toBe('true')
    expect(wrapper.find('.skill-card__selection-ribbon').exists()).toBe(true)
    expect(wrapper.find('.skill-card__selection-checkbox').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('Selected for installation')
  })

  it('disables selection for disabled skills while keeping details and editing available', () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ enabled: false }), selected: false } })

    expect(wrapper.find('.skill-card__selection-surface').attributes('disabled')).toBeDefined()
    expect(wrapper.find('.skill-card__description').attributes('disabled')).toBeUndefined()
    expect(wrapper.find('.skill-card__edit').attributes('disabled')).toBeUndefined()
  })

  it('always exposes the per-skill edit action', async () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ id: 'triage' }), selected: false } })
    await wrapper.find('.skill-card__edit').trigger('click')
    expect(wrapper.emitted('edit')).toEqual([['triage']])
  })

  it('renders assigned pack names and local/install status', () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ local: true, installed: true, tags: ['workflow'] }),
        selected: false,
        tags: [{ id: 'workflow', name: 'Workflow', color: '#E75480', order: 1, enabled: true }],
      },
    })

    expect(wrapper.text()).toContain('Local')
    expect(wrapper.text()).toContain('Installed')
    expect(wrapper.text()).toContain('Workflow')
  })

  it('does not render raw pack ids when pack metadata is unavailable', () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ tags: ['secret-tag', '#E75480'] }), selected: false },
    })

    expect(wrapper.text()).not.toContain('secret-tag')
    expect(wrapper.attributes('style') ?? '').not.toContain('#E75480')
    expect(wrapper.find('.pack-badge').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('No packs')
  })

  it('renders Pack markers as static compact badges with their configured color', () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ tags: ['workflow'] }),
        selected: false,
        tags: [{ id: 'workflow', name: 'Workflow', color: '#14B8A6', order: 1, enabled: true }],
      },
    })
    const badge = wrapper.get('.pack-badge')
    expect(badge.element.tagName).toBe('SPAN')
    expect(badge.classes()).toContain('pack-badge--compact')
    expect(badge.attributes('style')).toContain('--pack-color: #14B8A6')
  })
})
