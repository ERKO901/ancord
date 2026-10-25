import { sha256 } from "../crypto";
import { b64uDecodeLen, b64uEncode, concat, randomInt, u32le } from "../encoding";

export interface CpuParams {
  seed: string;
  iters: number;
  every: number;
  lanes: number;
}

/** h₀ pro danou dráhu: sha256(seed ‖ lane_le_u32). */
export function laneStart(seed: Uint8Array, lane: number): Uint8Array {
  return sha256(concat(seed, u32le(lane)));
}

export function advance(h: Uint8Array, steps: number): Uint8Array {
  let x = h;
  for (let i = 0; i < steps; i++) x = sha256(x);
  return x;
}

/** Totéž nativním digestem runtime: asi 4× rychlejší než JS implementace, proto ho používá ověřování. */
export async function advanceNative(h: Uint8Array, steps: number): Promise<Uint8Array> {
  let x = h;
  for (let i = 0; i < steps; i++) x = new Uint8Array(await crypto.subtle.digest("SHA-256", x));
  return x;
}

/** Referenční výpočet (node a testy): checkpointy všech drah. */
export function computeCheckpoints(seed: Uint8Array, lanes: number, iters: number, every: number): string[][] {
  const out: string[][] = [];
  for (let lane = 0; lane < lanes; lane++) {
    const cps: string[] = [];
    let h = laneStart(seed, lane);
    for (let j = 0; j < iters / every; j++) {
      h = advance(h, every);
      cps.push(b64uEncode(h));
    }
    out.push(cps);
  }
  return out;
}

export type CpuVerdict = { ok: true } | { ok: false; reason: string };

/** Zkontroluje tvar odpovědi a přepočítá `segments` náhodných úseků. */
export async function verifyCpu(
  seedB64: string,
  lanes: number,
  iters: number,
  every: number,
  segments: number,
  response: unknown,
): Promise<CpuVerdict> {
  const seed = b64uDecodeLen(seedB64, 32);
  if (!seed) return { ok: false, reason: "bad seed" };
  const perLane = iters / every;
  if (!Array.isArray(response) || response.length !== lanes) return { ok: false, reason: "wrong lane count" };
  const cps: Uint8Array[][] = [];
  for (const lane of response) {
    if (!Array.isArray(lane) || lane.length !== perLane) return { ok: false, reason: "wrong checkpoint count" };
    const decoded: Uint8Array[] = [];
    for (const c of lane) {
      const b = typeof c === "string" ? b64uDecodeLen(c, 32) : null;
      if (!b) return { ok: false, reason: "malformed checkpoint" };
      decoded.push(b);
    }
    cps.push(decoded);
  }
  const total = lanes * perLane;
  const picks = new Set<number>();
  const want = Math.min(segments, total);
  while (picks.size < want) picks.add(randomInt(total));
  for (const p of picks) {
    const lane = Math.floor(p / perLane);
    const j = p % perLane;
    const laneCps = cps[lane]!;
    const start = j === 0 ? laneStart(seed, lane) : laneCps[j - 1]!;
    const end = await advanceNative(start, every);
    const claimed = laneCps[j]!;
    if (end.length !== claimed.length || end.some((v, i) => v !== claimed[i])) {
      return { ok: false, reason: `segment ${lane}/${j} mismatch` };
    }
  }
  return { ok: true };
}
