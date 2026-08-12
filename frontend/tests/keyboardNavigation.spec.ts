import { describe, expect, it } from 'vitest'

import { enableKeyboardNavigationFocus } from '../src/utils/keyboardNavigation'

describe('keyboard navigation focus mode', () => {
  it('appears after Tab and is removed on pointer interaction', () => {
    const root = document.createElement('div')
    const events = document.implementation.createHTMLDocument()
    enableKeyboardNavigationFocus(root, events)

    events.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
    expect(root.classList.contains('is-keyboard-navigation')).toBe(false)
    events.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab' }))
    expect(root.classList.contains('is-keyboard-navigation')).toBe(true)
    events.dispatchEvent(new Event('pointerdown'))
    expect(root.classList.contains('is-keyboard-navigation')).toBe(false)
  })
})
