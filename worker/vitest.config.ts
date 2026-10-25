import { cloudflareTest } from "@cloudflare/vitest-plugin";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: "./wrangler.jsonc" },
      miniflare: {
        bindings: {
          PROFILE: "dev",
          WORKER_SIGNING_KEY: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
          CHALLENGE_HMAC_KEY: "ISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0-Pz8",
          PUSH_ENC_KEY: "QEFCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaW1xdXl8",
          ADMIN_TOKEN: "test-admin-token",
        },
      },
    }),
  ],
  test: {
    include: ["test/**/*.test.ts"],
    testTimeout: 30_000,
  },
});
