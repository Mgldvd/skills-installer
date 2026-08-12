import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import SkillToolbar from '../src/components/SkillToolbar/SkillToolbar.vue'
import type { SkillTag } from '../src/types'
import { makeSkill } from './fixtures'

const tags: SkillTag[] = [
  { id: 'recommended', name: 'Recommended', color: '#E75480', order: 10, enabled: true },
  { id: 'testing', name: 'Testing', color: '#56A37B', order: 20, enabled: true },
]

describe('SkillToolbar Pack preselection', () => {
  it('renders globally ordered Pack controls', () => {
    const wrapper = mount(SkillToolbar, { props: { tags, skills: [makeSkill({ tags: ['recommended'] })], selectedIds: [] } })
    expect(wrapper.findAll('.skill-toolbar__tag').map((item) => item.text())).toEqual(['Recommended1', 'Testing0'])
  })

  it('shows the active state when every Skill belonging to a Pack is selected', () => {
    const wrapper = mount(SkillToolbar, { props: { tags, skills: [makeSkill({ id: 'a', tags: ['recommended'] })], selectedIds: ['a'] } })
    expect(wrapper.find('.skill-toolbar__tag').attributes('aria-pressed')).toBe('true')
    expect(wrapper.find('.skill-toolbar__tag').classes()).toContain('pack-badge--selected')
  })

  it('exposes partial Pack selection as a non-color mixed state', () => {
    const skills = [makeSkill({ id: 'a', tags: ['recommended'] }), makeSkill({ id: 'b', tags: ['recommended'] })]
    const wrapper = mount(SkillToolbar, { props: { tags, skills, selectedIds: ['a'] } })
    const badge = wrapper.find('.skill-toolbar__tag')
    expect(badge.attributes('aria-pressed')).toBe('mixed')
    expect(badge.classes()).toContain('pack-badge--partial')
    expect(badge.find('.pack-badge__state').exists()).toBe(true)
  })

  it('emits one Pack toggle without opening another menu', async () => {
    const wrapper = mount(SkillToolbar, { props: { tags, skills: [makeSkill({ tags: ['recommended'] })], selectedIds: [] } })
    await wrapper.find('.skill-toolbar__tag').trigger('click')
    expect(wrapper.emitted('toggleTag')).toEqual([['recommended']])
    expect(wrapper.find('dialog').exists()).toBe(false)
  })

  it('keeps Clear with Preselect and disables it when nothing is selected', async () => {
    const wrapper = mount(SkillToolbar, { props: { tags, skills: [makeSkill()], selectedIds: [] } })
    const clear = wrapper.get('.skill-toolbar__clear')
    expect(clear.attributes('disabled')).toBeDefined()

    await wrapper.setProps({ selectedIds: ['triage'] })
    expect(clear.attributes('disabled')).toBeUndefined()
    await clear.trigger('click')
    expect(wrapper.emitted('clearSelection')).toHaveLength(1)
  })

  it('places filter and Name/Pack sorting controls beside preselection', async () => {
    const wrapper = mount(SkillToolbar, { props: { tags, skills: [makeSkill()], selectedIds: [], query: '', sortBy: 'name' } })
    const search = wrapper.get('input[type="search"]')
    await search.setValue('triage')
    expect(wrapper.emitted('update:query')).toEqual([['triage']])
    const sort = wrapper.get('select[aria-label="Sort Skills"]')
    expect(sort.findAll('option').map((option) => option.text())).toEqual(['Name', 'Pack'])
    await sort.setValue('pack')
    expect(wrapper.emitted('update:sortBy')).toEqual([['pack']])
  })
})
