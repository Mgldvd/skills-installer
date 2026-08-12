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
      '#F43F75',
      '#F05252',
      '#F97316',
      '#F59E0B',
      '#22C55E',
      '#14B8A6',
      '#06B6D4',
      '#3B82F6',
      '#6366F1',
      '#A855F7',
    ]
    for (const id of ['a', 'b', 'c', 'ui', 'frontend', 'legacy-group', 'alpha', 'bravo-team']) {
      expect(backendPalette).toContain(deterministicColor(id))
    }
  })
})
