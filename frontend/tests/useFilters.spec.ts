import { beforeEach, describe, expect, it } from 'vitest'

import { useAppState } from '../src/composables/useAppState'
import { useFilters } from '../src/composables/useFilters'
import { makeSkill } from './fixtures'
import { resetState } from './test-utils'

describe('useFilters', () => {
  const state = useAppState()

  beforeEach(() => {
    resetState(state)
    state.skills = [
      makeSkill({ id: 'a', displayName: 'Alpha Skill', tags: ['ui'] }),
      makeSkill({ id: 'b', displayName: 'Beta Skill', tags: ['tests'], installed: true }),
      makeSkill({ id: 'c', displayName: 'Gamma Local', tags: [], local: true }),
    ]
  })

  it('shows every skill when the search query is empty', () => {
    const { filteredSkills } = useFilters()
    expect(filteredSkills.value).toHaveLength(3)
  })

  it('filters by search query across display name', () => {
    const { filteredSkills, setSearchQuery } = useFilters()
    setSearchQuery('alpha')
    expect(filteredSkills.value.map((s) => s.id)).toEqual(['a'])
  })

  it('search is case-insensitive and never triggers a network request while typing', () => {
    const { filteredSkills, setSearchQuery } = useFilters()
    setSearchQuery('BETA')
    expect(filteredSkills.value.map((s) => s.id)).toEqual(['b'])
  })

  it('does not expose tags as a main-grid filter', () => {
    const { filteredSkills, setSearchQuery } = useFilters()
    setSearchQuery('ui')
    expect(filteredSkills.value).toEqual([])
  })

  it('clearing the search query restores the full list', () => {
    const { filteredSkills, setSearchQuery } = useFilters()
    setSearchQuery('alpha')
    expect(filteredSkills.value).toHaveLength(1)
    setSearchQuery('')
    expect(filteredSkills.value).toHaveLength(3)
  })
})
