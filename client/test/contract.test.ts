import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { COMMAND_NAMES } from "../src/lib/types";
import { createMockBackend } from "../src/lib/mock/backend";

const read = (p: string) => readFileSync(new URL(p, import.meta.url), "utf8");
const kebab = (s: string) => s.replaceAll("_", "-");

describe("IPC kontrakt", () => {
  it("mock backend implementuje právě příkazy z kontraktu", () => {
    const names = Object.keys(createMockBackend().handlers).sort();
    expect([...COMMAND_NAMES].sort()).toEqual(names);
  });

  it("Rust shell registruje právě tytéž příkazy (build.rs, lib.rs, capabilities)", () => {
    const build = read("../src-tauri/build.rs");
    const lib = read("../src-tauri/src/lib.rs");
    const caps = JSON.parse(read("../src-tauri/capabilities/default.json")) as { permissions: string[] };
    const handler = /generate_handler!\[([\s\S]*?)\]/.exec(lib)?.[1] ?? "";
    for (const c of COMMAND_NAMES) {
      expect(build, `build.rs: ${c}`).toContain(`"${c}"`);
      expect(handler, `generate_handler: ${c}`).toMatch(new RegExp(`\\b${c}\\b`));
      expect(caps.permissions, `capabilities: ${c}`).toContain(`allow-${kebab(c)}`);
    }
    const allowed = caps.permissions.filter((p) => p.startsWith("allow-"));
    expect(allowed).toHaveLength(COMMAND_NAMES.length);
    expect(handler.split(",").map((s) => s.trim()).filter(Boolean)).toHaveLength(COMMAND_NAMES.length);
  });
});

describe("hranice důvěry WebView (§13.3, §13.5)", () => {
  it("CSP zakazuje síť a inline kód", () => {
    const conf = JSON.parse(read("../src-tauri/tauri.conf.json")) as { app: { security: { csp: string }; withGlobalTauri: boolean } };
    const csp = conf.app.security.csp;
    expect(csp).toContain("default-src 'none'");
    expect(csp).toContain("connect-src ipc: http://ipc.localhost");
    expect(csp).not.toMatch(/unsafe-inline|unsafe-eval|https?:\/\/(?!ipc\.localhost)|\*/);
    expect(conf.app.withGlobalTauri).toBe(false);
  });

  it("capabilities nepovolují pluginy fs, shell, http, opener, updater ani global-shortcut", () => {
    const caps = read("../src-tauri/capabilities/default.json");
    for (const p of ["fs", "shell", "http", "opener", "updater", "global-shortcut", "core:default"]) {
      expect(caps).not.toContain(`"${p}`);
      expect(caps).not.toContain(`${p}:`);
    }
  });

  it("zdroj UI nevolá síť ani nevkládá HTML", () => {
    const files = import.meta.glob("../src/**/*.{ts,svelte}", { query: "?raw", import: "default", eager: true }) as Record<string, string>;
    expect(Object.keys(files).length).toBeGreaterThan(10);
    for (const [path, src] of Object.entries(files)) {
      if (path.includes("/mock/")) continue;
      expect(src, path).not.toMatch(/\bfetch\s*\(|XMLHttpRequest|new WebSocket|EventSource|sendBeacon|\{@html|innerHTML|https?:\/\/(?!schema|www\.w3\.org)/);
    }
  });

  it("index.html nenačítá nic zvenku", () => {
    const html = read("../index.html");
    expect(html).not.toMatch(/(src|href)="https?:/);
    expect(html).not.toMatch(/<link[^>]+(stylesheet|preconnect)[^>]+https?:/);
  });
});
