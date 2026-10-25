import { blake3 } from "./crypto";
import { compareBytes, concat, u64le, utf8 } from "./encoding";

const DOMAIN = utf8("ancord/v1/hrw");

/** Anchory mailboxu v epoše (spec §7): `r` nodů s nejvyšší HRW váhou. */
export function anchorsFor(mailboxId: Uint8Array, nodeIds: string[], epoch: number, r: number): string[] {
  const scored = nodeIds.map((id) => ({
    id,
    w: blake3(concat(DOMAIN, new Uint8Array([0]), mailboxId, utf8(id), new Uint8Array([0]), u64le(epoch))),
  }));
  scored.sort((a, b) => compareBytes(b.w, a.w) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  return scored.slice(0, r).map((s) => s.id);
}
