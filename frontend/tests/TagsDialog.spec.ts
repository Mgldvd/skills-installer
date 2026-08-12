import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import TagsDialog from '../src/components/TagsDialog/TagsDialog.vue'
import { makeSkill } from './fixtures'

const tags = [
  { id: 'recommended', name: 'Recommended', color: '#E75480', order: 10, enabled: true },
  { id: 'frontend', name: 'Frontend', color: '#5F82C9', order: 20, enabled: true },
  { id: 'testing', name: 'Testing', color: '#56A37B', order: 30, enabled: true },
  { id: 'hidden', name: 'Hidden', color: '#808089', order: 40, enabled: false },
]

function matrix() {
  return mount(TagsDialog, {
    props: {
      open: true,
      tags,
      skills: [
        makeSkill({ id: 'a', displayName: 'Skill A', description: 'Must not render', tags: ['recommended'] }),
        makeSkill({ id: 'b', displayName: 'Skill B', description: 'Also hidden', tags: ['frontend', 'testing'] }),
      ],
    },
  })
}

describe('Packs assignment matrix', () => {
  it('renders only skill names and every enabled Pack in consistent order', () => {
    const wrapper = matrix()
    const rows = wrapper.findAll('.skill-row')
    expect(rows).toHaveLength(2)
    expect(wrapper.text()).not.toContain('Must not render')
    expect(wrapper.text()).not.toContain('Also hidden')
    expect(rows.map(row => row.findAll('.tag-toggle').map(toggle => toggle.findAll('span').at(-1)?.text()))).toEqual([
      ['Recommended', 'Frontend', 'Testing'],
      ['Recommended', 'Frontend', 'Testing'],
    ])
    expect(rows.every(row => !row.text().includes('Hidden'))).toBe(true)
  })

  it('renders the requested active/inactive assignment states', () => {
    const [skillA, skillB] = matrix().findAll('.skill-row')
    expect(skillA.findAll('.tag-toggle').map(toggle => toggle.attributes('aria-pressed'))).toEqual(['true', 'false', 'false'])
    expect(skillB.findAll('.tag-toggle').map(toggle => toggle.attributes('aria-pressed'))).toEqual(['false', 'true', 'true'])
    expect(skillA.findAll('.is-assigned')).toHaveLength(1)
    expect(skillB.findAll('.is-assigned')).toHaveLength(2)
  })

  it('assigns an inactive Pack with one click and opens no assignment surface', async () => {
    const wrapper = matrix()
    const testing = wrapper.findAll('.skill-row')[0].findAll('.tag-toggle')[2]
    await testing.trigger('click')
    expect(wrapper.emitted('assign')).toEqual([['a', 'testing']])
    expect(wrapper.find('.assign-popover').exists()).toBe(false)
    expect(wrapper.find('select').exists()).toBe(false)
  })

  it('unassigns an active Pack with one click and no confirmation', async () => {
    const wrapper = matrix()
    const frontend = wrapper.findAll('.skill-row')[1].findAll('.tag-toggle')[1]
    await frontend.trigger('click')
    expect(wrapper.emitted('unassign')).toEqual([['b', 'frontend']])
  })

  it('keeps global creation and removes every per-Skill Add Pack control', async () => {
    const wrapper = matrix()
    expect(wrapper.get('.tags-dialog__header').text()).toContain('+ New Pack')
    expect(wrapper.text()).not.toContain('+ Add Tag')
    expect(wrapper.text()).not.toContain('+ Create New Tag')
    await wrapper.get('.tags-dialog__header .button').trigger('click')
    expect(wrapper.find('.tags-dialog__editor').exists()).toBe(true)
  })

  it('shows a newly supplied global Pack inactive on every row', async () => {
    const wrapper = matrix()
    await wrapper.setProps({ tags: [...tags, { id:'security', name:'Security', color:'#6870C4', order:50, enabled:true }] })
    const rows = wrapper.findAll('.skill-row')
    expect(rows.every(row => row.findAll('.tag-toggle').at(-1)?.text() === 'Security')).toBe(true)
    expect(rows.every(row => row.findAll('.tag-toggle').at(-1)?.attributes('aria-pressed') === 'false')).toBe(true)
  })

  it('searches by skill name only', async () => {
    const wrapper = matrix()
    await wrapper.get('[aria-label="Search skills"]').setValue('skill b')
    expect(wrapper.findAll('.skill-row')).toHaveLength(1)
    expect(wrapper.text()).toContain('Skill B')
    await wrapper.get('[aria-label="Search skills"]').setValue('hidden')
    expect(wrapper.findAll('.skill-row')).toHaveLength(0)
  })

  it('disables only the pending relationship without moving layout', () => {
    const wrapper = mount(TagsDialog, { props: { open:true, tags, pendingKeys:['a:frontend'], skills:[makeSkill({id:'a',displayName:'Skill A',tags:[]})] } })
    const controls=wrapper.findAll('.tag-toggle')
    expect(controls[1].attributes('disabled')).toBeDefined()
    expect(controls[1].classes()).toContain('is-pending')
    expect(controls[0].attributes('disabled')).toBeUndefined()
  })
})
