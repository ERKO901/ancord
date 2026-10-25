import { canonicalBytes } from "./canonical";
import { DOMAIN, verifySig, type Signer } from "./crypto";
import { b64uDecode, b64uEncode, fromUtf8, randomBytes, timingSafeEqual, utf8 } from "./encoding";

export interface TicketBody {
  v: 1;
  nid: string;
  ek: string;
  exp: number;
  n: string;
}

export async function issueTicket(
  signer: Signer,
  nodeId: string,
  clientKey: string,
  expiresAt: number,
  nonce: Uint8Array = randomBytes(16),
): Promise<string> {
  const body: TicketBody = { v: 1, nid: nodeId, ek: clientKey, exp: expiresAt, n: b64uEncode(nonce) };
  const payload = canonicalBytes(body as unknown as Record<string, string | number>);
  const sig = await signer.sign(DOMAIN.ticket, payload);
  return `${b64uEncode(payload)}.${b64uEncode(sig)}`;
}

export type TicketCheck = { ok: true; body: TicketBody } | { ok: false; reason: string };

/** Ověření ticketu tak, jak ho dělá node (referenční implementace pro testy a vektory). */
export async function verifyTicket(
  workerKey: Uint8Array,
  ticket: string,
  expectedNodeId: string,
  nowS: number,
): Promise<TicketCheck> {
  const parts = ticket.split(".");
  if (parts.length !== 2) return { ok: false, reason: "malformed" };
  const payload = b64uDecode(parts[0]!);
  const sig = b64uDecode(parts[1]!);
  if (!payload || !sig) return { ok: false, reason: "malformed" };
  if (!(await verifySig(workerKey, DOMAIN.ticket, payload, sig))) return { ok: false, reason: "bad signature" };
  let body: TicketBody;
  try {
    body = JSON.parse(fromUtf8(payload)) as TicketBody;
  } catch {
    return { ok: false, reason: "malformed" };
  }
  if (body.v !== 1 || typeof body.nid !== "string" || typeof body.exp !== "number") return { ok: false, reason: "malformed" };
  // musí jít o přesně kanonické bajty, jinak by jeden ticket měl víc podob
  if (!timingSafeEqual(payload, utf8(JSON.stringify(sortKeys(body))))) return { ok: false, reason: "not canonical" };
  if (body.nid !== expectedNodeId) return { ok: false, reason: "wrong node" };
  if (body.exp <= nowS) return { ok: false, reason: "expired" };
  return { ok: true, body };
}

function sortKeys(o: TicketBody): Record<string, unknown> {
  return Object.fromEntries(Object.entries(o).sort(([a], [b]) => (a < b ? -1 : 1)));
}
