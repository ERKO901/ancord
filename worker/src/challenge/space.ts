import { sha256 } from "../crypto";
import { b64uDecodeLen, b64uEncode, concat, randomInt, timingSafeEqual, u64le, utf8 } from "../encoding";

export const LEAF_BYTES = 32;
const D0 = utf8("ancord/space/0");
const DI = utf8("ancord/space/i");

export function spaceT0(seed: Uint8Array): Uint8Array {
  return sha256(concat(D0, seed));
}

/** j(i) = u64_le(T[i−1][0..8]) mod i */
export function refIndex(prev: Uint8Array, i: number): number {
  const v = new DataView(prev.buffer, prev.byteOffset, 8).getBigUint64(0, true);
  return Number(v % BigInt(i));
}

export function spaceT(seed: Uint8Array, i: number, prev: Uint8Array, ref: Uint8Array): Uint8Array {
  return sha256(concat(DI, seed, u64le(i), prev, ref));
}

export function leafHash(t: Uint8Array): Uint8Array {
  return sha256(concat(new Uint8Array([0]), t));
}

export function nodeHash(l: Uint8Array, r: Uint8Array): Uint8Array {
  return sha256(concat(new Uint8Array([1]), l, r));
}

export function depthOf(leaves: number): number {
  return Math.log2(leaves);
}

export function isPowerOfTwo(n: number): boolean {
  return Number.isInteger(n) && n >= 2 && (n & (n - 1)) === 0;
}

/** Referenční výpočet tabulky (node a testy). */
export function buildTable(seed: Uint8Array, leaves: number): Uint8Array {
  const table = new Uint8Array(leaves * LEAF_BYTES);
  table.set(spaceT0(seed), 0);
  for (let i = 1; i < leaves; i++) {
    const prev = table.subarray((i - 1) * LEAF_BYTES, i * LEAF_BYTES);
    const j = refIndex(prev, i);
    table.set(spaceT(seed, i, prev, table.subarray(j * LEAF_BYTES, (j + 1) * LEAF_BYTES)), i * LEAF_BYTES);
  }
  return table;
}

/** levels[0] jsou hashe listů, poslední úroveň je kořen. */
export function buildTree(table: Uint8Array): Uint8Array[][] {
  const n = table.length / LEAF_BYTES;
  let level: Uint8Array[] = [];
  for (let i = 0; i < n; i++) level.push(leafHash(table.subarray(i * LEAF_BYTES, (i + 1) * LEAF_BYTES)));
  const levels = [level];
  while (level.length > 1) {
    const next: Uint8Array[] = [];
    for (let i = 0; i < level.length; i += 2) next.push(nodeHash(level[i]!, level[i + 1]!));
    levels.push(next);
    level = next;
  }
  return levels;
}

export function merklePath(levels: Uint8Array[][], index: number): Uint8Array[] {
  const path: Uint8Array[] = [];
  let idx = index;
  for (let k = 0; k < levels.length - 1; k++) {
    path.push(levels[k]![idx ^ 1]!);
    idx >>= 1;
  }
  return path;
}

export function verifyPath(root: Uint8Array, index: number, value: Uint8Array, path: Uint8Array[], leaves: number): boolean {
  if (path.length !== depthOf(leaves) || index < 0 || index >= leaves) return false;
  let h = leafHash(value);
  let idx = index;
  for (const sib of path) {
    h = idx & 1 ? nodeHash(sib, h) : nodeHash(h, sib);
    idx >>= 1;
  }
  return timingSafeEqual(h, root);
}

export interface Opening {
  i: number;
  leaf: string;
  path: string[];
  prev: string;
  prev_path: string[];
  ref: string;
  ref_path: string[];
}

/** Referenční odpověď na otevření (node a testy). */
export function makeOpening(seed: Uint8Array, table: Uint8Array, levels: Uint8Array[][], i: number): Opening {
  const at = (k: number) => table.subarray(k * LEAF_BYTES, (k + 1) * LEAF_BYTES);
  const j = refIndex(at(i - 1), i);
  const enc = (p: Uint8Array[]) => p.map(b64uEncode);
  void seed;
  return {
    i,
    leaf: b64uEncode(at(i)),
    path: enc(merklePath(levels, i)),
    prev: b64uEncode(at(i - 1)),
    prev_path: enc(merklePath(levels, i - 1)),
    ref: b64uEncode(at(j)),
    ref_path: enc(merklePath(levels, j)),
  };
}

function decodeBytes(v: unknown, len: number): Uint8Array | null {
  return typeof v === "string" ? b64uDecodeLen(v, len) : null;
}

function decodePath(v: unknown, depth: number): Uint8Array[] | null {
  if (!Array.isArray(v) || v.length !== depth) return null;
  const out: Uint8Array[] = [];
  for (const e of v) {
    const b = decodeBytes(e, 32);
    if (!b) return null;
    out.push(b);
  }
  return out;
}

export type SpaceVerdict = { ok: true } | { ok: false; reason: string };

/** Ověří otevření všech požadovaných indexů vůči uloženému kořeni. */
export function verifyOpenings(
  seed: Uint8Array,
  root: Uint8Array,
  leaves: number,
  indices: number[],
  response: unknown,
): SpaceVerdict {
  if (!Array.isArray(response) || response.length !== indices.length) return { ok: false, reason: "wrong opening count" };
  const depth = depthOf(leaves);
  for (let n = 0; n < indices.length; n++) {
    const o = response[n] as Record<string, unknown> | null;
    const i = indices[n]!;
    if (!o || typeof o !== "object" || o.i !== i) return { ok: false, reason: "opening order mismatch" };
    const leaf = decodeBytes(o.leaf, 32);
    const prev = decodeBytes(o.prev, 32);
    const ref = decodeBytes(o.ref, 32);
    const path = decodePath(o.path, depth);
    const prevPath = decodePath(o.prev_path, depth);
    const refPath = decodePath(o.ref_path, depth);
    if (!leaf || !prev || !ref || !path || !prevPath || !refPath) return { ok: false, reason: "malformed opening" };
    const j = refIndex(prev, i);
    if (!verifyPath(root, i, leaf, path, leaves)) return { ok: false, reason: `bad path for leaf ${i}` };
    if (!verifyPath(root, i - 1, prev, prevPath, leaves)) return { ok: false, reason: `bad path for prev of ${i}` };
    if (!verifyPath(root, j, ref, refPath, leaves)) return { ok: false, reason: `bad path for ref of ${i}` };
    if (i - 1 === 0 && !timingSafeEqual(prev, spaceT0(seed))) return { ok: false, reason: "T[0] mismatch" };
    if (j === 0 && !timingSafeEqual(ref, spaceT0(seed))) return { ok: false, reason: "T[0] mismatch" };
    if (!timingSafeEqual(spaceT(seed, i, prev, ref), leaf)) return { ok: false, reason: `T[${i}] inconsistent` };
  }
  return { ok: true };
}

/** `count` náhodných indexů z [1, leaves), bez opakování. */
export function pickIndices(leaves: number, count: number): number[] {
  const want = Math.min(count, leaves - 1);
  const s = new Set<number>();
  while (s.size < want) s.add(1 + randomInt(leaves - 1));
  return [...s];
}

/** Největší mocnina dvou ≤ n, ale aspoň `floor`. */
export function leavesFor(claimedBytes: number, minLeaves: number, maxLeaves: number): number {
  const target = Math.min(Math.floor(claimedBytes / LEAF_BYTES), maxLeaves);
  let l = minLeaves;
  while (l * 2 <= target) l *= 2;
  return l;
}
