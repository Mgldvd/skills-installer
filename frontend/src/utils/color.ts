/** Mirrors the backend's curated palette (src-tauri/src/config/color.rs) so
 * client-computed colors read as part of the same system as
 * server-assigned ones (group colors). */
const CURATED_PALETTE: readonly string[] = [
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

const FNV_OFFSET_BASIS = 0xcbf29ce484222325n
const FNV_PRIME = 0x100000001b3n
const MASK_64 = (1n << 64n) - 1n

/** Deterministic (never random) color assignment, ported byte-for-byte from
 * the backend's `deterministic_color_for_group_id` (same FNV-1a 64-bit hash
 * over UTF-8 bytes, same palette) so the two stay visually consistent
 * without a shared runtime. */
export function deterministicColor(id: string): string {
  let hash = FNV_OFFSET_BASIS
  for (const byte of new TextEncoder().encode(id)) {
    hash ^= BigInt(byte)
    hash = (hash * FNV_PRIME) & MASK_64
  }
  const index = Number(hash % BigInt(CURATED_PALETTE.length))
  return CURATED_PALETTE[index]
}
