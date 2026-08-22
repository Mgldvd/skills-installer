import { reactive } from "vue";

export type ToastVariant = "info" | "success" | "error";

export interface Toast {
  id: number;
  message: string;
  variant: ToastVariant;
}

const toasts = reactive<Toast[]>([]);
let nextId = 1;

/**
 * For lightweight, non-critical confirmations only (Skill added, Group
 * updated, ...) — important errors must also get a persistent surface
 * (a dialog or inline banner), never rely on a toast alone.
 */
export function useToasts() {
  function push(message: string, variant: ToastVariant = "info", durationMs = 4000) {
    const id = nextId++;
    toasts.push({ id, message, variant });
    setTimeout(() => dismiss(id), durationMs);
  }

  function dismiss(id: number) {
    const index = toasts.findIndex((t) => t.id === id);
    if (index !== -1) toasts.splice(index, 1);
  }

  return { toasts, push, dismiss };
}
