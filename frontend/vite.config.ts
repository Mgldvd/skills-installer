/// <reference types="vitest/config" />
import { resolve } from "node:path";

import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    // Keeps every generated/non-versioned artifact under a single
    // `.generated/` at the repo root instead of `frontend/dist` — see
    // `tauri.conf.json`'s `frontendDist`, which must point here too.
    outDir: resolve(import.meta.dirname, "../.generated/frontend"),
    // outDir sits outside this project's root, so Vite otherwise refuses to
    // clear it automatically before each build.
    emptyOutDir: true,
    rollupOptions: {
      // The style guide (design.html) is a separate, standalone entry —
      // not part of the app's own routing — so it needs to be listed
      // explicitly to end up in the production build alongside index.html.
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        design: resolve(import.meta.dirname, "design.html"),
      },
    },
  },
  test: {
    environment: "jsdom",
    include: ["tests/**/*.spec.ts"],
    setupFiles: ["tests/setup.ts"],
    globals: false,
  },
});
