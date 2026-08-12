export function enableKeyboardNavigationFocus(documentElement: HTMLElement, eventTarget: Document = document) {
  const keyboardClass = 'is-keyboard-navigation'

  eventTarget.addEventListener('keydown', (event) => {
    if (event.key === 'Tab') documentElement.classList.add(keyboardClass)
  })
  eventTarget.addEventListener('pointerdown', () => documentElement.classList.remove(keyboardClass))
}
