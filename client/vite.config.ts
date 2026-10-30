import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "es2022", sourcemap: false },
  resolve: { conditions: ["browser"] },
  test: { environment: "jsdom", include: ["test/**/*.test.ts"], setupFiles: ["test/setup.ts"] },
});
