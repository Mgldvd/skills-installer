import { invoke } from "@tauri-apps/api/core";

/**
 * The only place `invoke()` is called directly. Every Tauri command gets a
 * typed wrapper in `services/backend.ts` instead of being invoked ad hoc
 * from components.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}
