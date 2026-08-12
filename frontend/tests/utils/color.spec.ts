import { describe, expect, it } from 'vitest'

import { deterministicColor } from '../../src/utils/color'

// Mirrors src-tauri/src/config/color.rs's own test cases: same algorithm,
// same palette, ported so the two stay in sync without a shared runtime.
describe('deterministicColor', () => {
  it('is stable across calls for the same id', () => {
    expect(deterministicColor('legacy-group')).toBe(deterministicColor('legacy-group'))
  })

  it('varies by id', () => {
    expect(deterministicColor('alpha')).not.toBe(deterministicColor('bravo-team'))
  })

  it('always returns a 6-digit uppercase hex color from the curated palette', () => {
    const color = deterministicColor('ui')
    expect(color).toMatch(/^#[0-9A-F]{6}$/)
  })

  it('matches the backend palette exactly, so generated colors read as one system', () => {
    // Ported 1:1 from CURATED_PALETTE in src-tauri/src/config/color.rs.
    const backendPalette = [
      '#E75480',
      '#E8785A',
      '#DB8A3E',
      '#C99A3B',
      '#4E9A70',
      '#3F9490',
      '#627FA4',
      '#6B82D9',
      '#8C6FB0',
      '#8A7F84',
    ]
    for (const id of ['a', 'b', 'c', 'ui', 'frontend', 'legacy-group', 'alpha', 'bravo-team']) {
      expect(backendPalette).toContain(deterministicColor(id))
    }
  })
})
