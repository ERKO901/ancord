import { sha256, leadingZeroBits } from "./crypto";
import { concat, u64le } from "./encoding";
import type { Config } from "./config";

/** sha256(nonce ‖ pubkey ‖ counter_le_u64) s `bits` úvodními nulovými bity (spec §4.2). */
export function hashcashValid(nonce: Uint8Array, pubkey: Uint8Array, counter: number, bits: number): boolean {
  if (bits <= 0) return true;
  if (!Number.isSafeInteger(counter) || counter < 0) return false;
  return leadingZeroBits(sha256(concat(nonce, pubkey, u64le(counter)))) >= bits;
}

/** Adaptivní obtížnost: základ + log2(1 + registrace v okně / jednotka), omezeno stropem. */
export function requiredBits(cfg: Config["hashcash"], recentRegistrations: number): number {
  if (!cfg.enabled) return 0;
  const surge = Math.floor(Math.log2(1 + recentRegistrations / cfg.surgeUnit));
  return Math.min(cfg.maxBits, cfg.baseBits + surge);
}

/** Řešič pro testy a referenční node. */
export function solveHashcash(nonce: Uint8Array, pubkey: Uint8Array, bits: number): number {
  for (let counter = 0; ; counter++) {
    if (hashcashValid(nonce, pubkey, counter, bits)) return counter;
  }
}
