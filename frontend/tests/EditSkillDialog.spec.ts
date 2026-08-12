import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { describe, expect, it } from 'vitest'

import EditSkillDialog from '../src/components/EditSkillDialog/EditSkillDialog.vue'
import { makeSkill } from './fixtures'

describe('EditSkillDialog', () => {
  it('saves pack assignments together with the skill information', async () => {
    const wrapper = mount(EditSkillDialog, {
      props: {
        open: true,
        skill: makeSkill({ id: 'triage', tags: ['workflow'] }),
        tags: [
          { id: 'workflow', name: 'Workflow', color: '#E75480', order: 1, enabled: true },
          { id: 'testing', name: 'Testing', color: '#5F82C9', order: 2, enabled: true },
        ],
      },
    })
    await nextTick()
    const packBadges = wrapper.findAll('.edit-skill-dialog__pack-list .pack-badge')
    expect(packBadges[0].attributes('aria-pressed')).toBe('true')
    expect(packBadges[0].classes()).toContain('pack-badge--selected')
    expect(packBadges[0].classes()).not.toContain('pack-badge--muted')
    expect(packBadges[1].attributes('aria-pressed')).toBe('false')
    expect(packBadges[1].classes()).toContain('pack-badge--muted')
    expect(wrapper.find('.edit-skill-dialog__pack-list input[type="checkbox"]').exists()).toBe(false)
    await packBadges[1].trigger('click')
    await wrapper.find('form').trigger('submit')

    const payload = wrapper.emitted('submit')?.[0]?.[0] as { tags: string[] }
    expect(payload.tags).toEqual(['workflow', 'testing'])
  })

  it('opens as an informative preview with editing visible', async () => {
    const wrapper = mount(EditSkillDialog, {
      props: { open: true, skill: makeSkill({ displayName: 'Issue Triage', description: 'A concise explanation of the Skill.' }) },
    })
    await nextTick()

    expect(wrapper.get('.edit-skill-dialog__hero h2').text()).toBe('Issue Triage')
    expect(wrapper.get('.edit-skill-dialog__description').text()).toBe('A concise explanation of the Skill.')
    expect(wrapper.get('.edit-skill-dialog__hero a').attributes()).toMatchObject({
      href: 'https://www.skills.sh/mattpocock/skills/triage',
      target: '_blank',
      rel: 'noopener noreferrer',
    })
    expect(wrapper.get('.edit-skill-dialog__actions').findAll('button').map((button) => button.text())).toEqual(['Delete Skill', 'Cancel', 'Save Changes'])
    expect(wrapper.find('.edit-skill-dialog__details-toggle').exists()).toBe(false)
    expect(wrapper.get('.edit-skill-dialog__fields').isVisible()).toBe(true)
  })

  it('edits the single concise description shown in the preview and cards', async () => {
    const wrapper = mount(EditSkillDialog, { props: { open: true, skill: makeSkill() } })
    await nextTick()
    const textarea = wrapper.get('textarea')
    await textarea.setValue('Concise card summary')
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('submit')?.[0]?.[0]).toMatchObject({
      description: 'Concise card summary',
    })
    expect(wrapper.findAll('textarea')).toHaveLength(1)
  })

  it('requires inline confirmation before deleting the Skill', async () => {
    const wrapper = mount(EditSkillDialog, {
      props: { open: true, skill: makeSkill({ id: 'triage', displayName: 'Issue Triage' }) },
    })
    await nextTick()

    await wrapper.get('.edit-skill-dialog__btn--delete').trigger('click')
    expect(wrapper.emitted('delete')).toBeUndefined()
    expect(wrapper.get('.edit-skill-dialog__delete-confirm').text()).toContain('Delete “Issue Triage”?')
    await wrapper.get('.edit-skill-dialog__btn--danger').trigger('click')
    expect(wrapper.emitted('delete')).toEqual([['triage']])
  })

  it('allows cancelling a pending deletion', async () => {
    const wrapper = mount(EditSkillDialog, { props: { open: true, skill: makeSkill({ id: 'triage' }) } })
    await nextTick()

    await wrapper.get('.edit-skill-dialog__btn--delete').trigger('click')
    await wrapper.get('.edit-skill-dialog__actions .edit-skill-dialog__btn').trigger('click')
    expect(wrapper.find('.edit-skill-dialog__delete-confirm').exists()).toBe(false)
    expect(wrapper.emitted('delete')).toBeUndefined()
  })

  it('closes when clicking the dialog backdrop but not its content', async () => {
    const wrapper = mount(EditSkillDialog, { props: { open: true, skill: makeSkill() } })
    await nextTick()
    await wrapper.get('.edit-skill-dialog__hero').trigger('click')
    expect(wrapper.emitted('update:open')).toBeUndefined()
    await wrapper.get('.edit-skill-dialog').trigger('click')
    expect(wrapper.emitted('update:open')).toEqual([[false]])
  })
})
