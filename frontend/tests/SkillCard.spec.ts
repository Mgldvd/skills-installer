import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import SkillCard from '../src/components/SkillCard/SkillCard.vue'
import { makeSkill } from './fixtures'

describe('SkillCard', () => {
  it('renders the Skill name and description without category metadata', () => {
    const wrapper = mount(SkillCard, {
      props: {
        skill: makeSkill({ displayName: 'Issue Triage', description: 'Helps triage issues.' }),
        selected: false,
        editMode: false,
      },
    })

    expect(wrapper.text()).toContain('Issue Triage')
    expect(wrapper.text()).toContain('Helps triage issues.')
    expect(wrapper.find('.skill-card__meta').exists()).toBe(false)
  })

  it('emits toggle on click', async () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ id: 'triage' }), selected: false, editMode: false },
    })

    await wrapper.trigger('click')

    expect(wrapper.emitted('toggle')).toEqual([['triage']])
  })

  it('emits toggle on Enter and Space key presses (keyboard accessible)', async () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ id: 'triage' }), selected: false, editMode: false },
    })

    await wrapper.trigger('keydown.enter')
    await wrapper.trigger('keydown.space')

    expect(wrapper.emitted('toggle')).toEqual([['triage'], ['triage']])
  })

  it('reflects selection with both a class and aria-pressed (not color alone)', () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill(), selected: true, editMode: false },
    })

    expect(wrapper.classes()).toContain('is-selected')
    expect(wrapper.attributes('aria-pressed')).toBe('true')
  })

  it('shows Edit/More actions only in edit mode', async () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill(), selected: false, editMode: false },
    })
    expect(wrapper.find('.skill-card__actions').exists()).toBe(false)

    await wrapper.setProps({ editMode: true })
    expect(wrapper.find('.skill-card__actions').exists()).toBe(true)
  })

  it('marks local skills distinctly from remote ones', () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ local: true }), selected: false, editMode: false },
    })
    expect(wrapper.text()).toContain('Local')
  })

  it('does not mark ordinary remote skills as Local or Remote', () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ local: false, repository: 'skills' }), selected: false, editMode: false },
    })
    expect(wrapper.text()).not.toContain('Local')
    expect(wrapper.text()).not.toContain('Remote')
  })

  it('keeps Installed visible only for installed Skills', async () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ installed: true }), selected: false, editMode: false },
    })
    expect(wrapper.text()).toContain('✓ Installed')
    await wrapper.setProps({ skill: makeSkill({ installed: false }) })
    expect(wrapper.text()).not.toContain('Installed')
  })

  it('never renders the Other fallback or any configured group name', () => {
    const wrapper = mount(SkillCard, {
      props: { skill: makeSkill({ groupId: 'other' }), selected: false, editMode: false },
    })
    expect(wrapper.text()).not.toContain('Other')
    expect(wrapper.find('[class*="group"]').exists()).toBe(false)
  })

  it('never renders Tag or legacy Pack markers in the installation card', () => {
    const wrapper = mount(SkillCard, { props: { skill: makeSkill({ tags: ['secret-tag', '#E75480'] }), selected: false, editMode: false } })
    expect(wrapper.text()).not.toContain('secret-tag')
    expect(wrapper.attributes('style') ?? '').not.toContain('#E75480')
    expect(wrapper.find('.tag-chip').exists()).toBe(false)
    expect(wrapper.find('[class*="pack"]').exists()).toBe(false)
  })
})
