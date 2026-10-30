import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Commands, Events } from "./types";

type Args<K extends keyof Commands> = Commands[K][0];
type Result<K extends keyof Commands> = Commands[K][1];

/** Jediná cesta UI do Rustu. Typované podle kontraktu v `types.ts`. */
export function call<K extends keyof Commands>(
  cmd: K,
  ...args: Args<K> extends void ? [] : [Args<K>]
): Promise<Result<K>> {
  return invoke<Result<K>>(cmd, args[0] as Record<string, unknown> | undefined);
}

export function on<K extends keyof Events>(
  name: K,
  handler: (payload: Events[K]) => void,
): Promise<UnlistenFn> {
  return listen<Events[K]>(name, (e) => handler(e.payload));
}

/** Mimo Tauri (vite dev v prohlížeči) se IPC nahradí pamětí. V release buildu se tato větev odstraní. */
export async function installDevBackend(): Promise<void> {
  if (!import.meta.env.DEV) return;
  if ("__TAURI_INTERNALS__" in window) return;
  const { installMockBackend } = await import("./mock/backend");
  await installMockBackend();
}
