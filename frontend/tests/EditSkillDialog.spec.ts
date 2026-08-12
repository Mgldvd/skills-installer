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

    const packInputs = wrapper.findAll('.edit-skill-dialog__pack input')
    expect((packInputs[0].element as HTMLInputElement).checked).toBe(true)
    await packInputs[1].setValue(true)
    await wrapper.find('form').trigger('submit')

    const payload = wrapper.emitted('submit')?.[0]?.[0] as { tags: string[] }
    expect(payload.tags).toEqual(['workflow', 'testing'])
  })
})
