// jsdom implements the `open` property/attribute on <dialog> but not the
// imperative showModal()/close() methods our dialog components rely on.
// Stubbed once, globally, rather than per spec file.
if (!HTMLDialogElement.prototype.showModal) {
  HTMLDialogElement.prototype.showModal = function stubShowModal(this: HTMLDialogElement) {
    this.open = true
  }
}
if (!HTMLDialogElement.prototype.close) {
  HTMLDialogElement.prototype.close = function stubClose(this: HTMLDialogElement) {
    this.open = false
  }
}
