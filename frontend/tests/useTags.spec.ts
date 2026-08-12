import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppState } from '../src/composables/useAppState'
import { useTags } from '../src/composables/useTags'
import { makeSkill } from './fixtures'
import { resetState } from './test-utils'

vi.mock('../src/services/backend', () => ({
  assignTagToSkill: vi.fn(), unassignTagFromSkill: vi.fn(), createTag: vi.fn(), updateTag: vi.fn(), deleteTag: vi.fn(),
}))
import * as backend from '../src/services/backend'

describe('useTags', () => {
  const state=useAppState()
  beforeEach(()=>{resetState(state);vi.clearAllMocks();state.skills=[makeSkill({id:'a',tags:[]})]})

  it('assigns and unassigns through the backend exactly once and updates only that skill', async()=>{
    vi.mocked(backend.assignTagToSkill).mockResolvedValue(makeSkill({id:'a',tags:['testing']}))
    vi.mocked(backend.unassignTagFromSkill).mockResolvedValue(makeSkill({id:'a',tags:[]}))
    const tags=useTags()
    await tags.assign('a','testing')
    expect(backend.assignTagToSkill).toHaveBeenCalledOnce()
    expect(state.skills[0].tags).toEqual(['testing'])
    await tags.unassign('a','testing')
    expect(backend.unassignTagFromSkill).toHaveBeenCalledOnce()
    expect(state.skills[0].tags).toEqual([])
  })

  it('makes a newly created Tag globally available without assigning it',async()=>{
    const created={id:'security',name:'Security',color:'#6870C4',order:40,enabled:true}
    vi.mocked(backend.createTag).mockResolvedValue(created)
    await useTags().create('Security','#6870C4')
    expect(state.tags).toEqual([created])
    expect(state.skills[0].tags).toEqual([])
    expect(backend.assignTagToSkill).not.toHaveBeenCalled()
  })

  it('does not mutate local state when persistence fails',async()=>{
    vi.mocked(backend.assignTagToSkill).mockRejectedValue(new Error('write failed'))
    await expect(useTags().assign('a','testing')).rejects.toThrow('write failed')
    expect(state.skills[0].tags).toEqual([])
  })
})
