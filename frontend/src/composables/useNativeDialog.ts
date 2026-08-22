import { onMounted, watch, type Ref } from "vue";

/**
 * Wires a `<dialog ref>` to an `open` boolean prop/getter. A plain
 * `watch(source, cb, { immediate: true })` gets this wrong: the immediate
 * invocation runs during `setup()`, before the template ref is bound, so a
 * dialog that happens to mount already-open would silently never call
 * `showModal()`. This covers both the initial-mount case (via `onMounted`)
 * and every later toggle (via a plain, non-immediate `watch`), sharing one
 * `sync` so both paths behave identically.
 */
export function useNativeDialog(dialogEl: Ref<HTMLDialogElement | null>, isOpen: () => boolean, onOpen?: () => void) {
  function sync(open: boolean) {
    const el = dialogEl.value;
    if (!el) return;
    if (open) {
      onOpen?.();
      if (!el.open) el.showModal();
    } else if (el.open) {
      el.close();
    }
  }

  onMounted(() => sync(isOpen()));
  watch(isOpen, sync);
}
