import type { Env as AncordEnv } from "../src/env";

declare global {
  namespace Cloudflare {
    interface Env extends AncordEnv {}
    interface GlobalProps {
      mainModule: typeof import("../src/index");
    }
  }
}

export {};
