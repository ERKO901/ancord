import { describe, expect, it } from "vitest";
import { requestPayload, verifyNodeRequest } from "../src/auth";
import { canonicalJson } from "../src/canonical";
import { DOMAIN, blake3, hostnameOf, kidOf, nodeIdOf, sha256, signerFromSeed, signingInput, verifySig } from "../src/crypto";
import { b64uDecode, b64uDecodeLen, b64uEncode, concat, toHex, u64le, utf8 } from "../src/encoding";
import { hashcashValid } from "../src/hashcash";
import { anchorsFor } from "../src/hrw";
import { issueTicket, verifyTicket } from "../src/ticket";
import { computeCheckpoints, verifyCpu } from "../src/challenge/cpu";
import { buildTable, buildTree, spaceT0, verifyOpenings } from "../src/challenge/space";
import { pushIdOf } from "../src/push/handler";
import canonicalVec from "../../spec/test-vectors/canonical-json.json";
import directoryVec from "../../spec/test-vectors/directory.json";
import hashchainVec from "../../spec/test-vectors/hashchain.json";
import heartbeatVec from "../../spec/test-vectors/heartbeat.json";
import hrwVec from "../../spec/test-vectors/hrw.json";
import identityVec from "../../spec/test-vectors/identity.json";
import proofVec from "../../spec/test-vectors/proof.json";
import pushVec from "../../spec/test-vectors/push.json";
import registerVec from "../../spec/test-vectors/register.json";
import requestVec from "../../spec/test-vectors/request-auth.json";
import spaceVec from "../../spec/test-vectors/space.json";
import ticketVec from "../../spec/test-vectors/ticket.json";

const dec = (s: string, len?: number) => (len ? b64uDecodeLen(s, len)! : b64uDecode(s)!);

describe("sdílené testovací vektory (spec/test-vectors)", () => {
  it("identity: node_id a hostname", () => {
    for (const c of identityVec.cases) {
      const pk = dec(c.pubkey, 32);
      expect(nodeIdOf(pk)).toBe(c.node_id);
      expect(hostnameOf(c.node_id, identityVec.zone)).toBe(c.hostname);
      expect(c.node_id).toHaveLength(52);
    }
  });

  it("kanonické JSON", () => {
    for (const c of canonicalVec.cases) expect(canonicalJson(c.input as never)).toBe(c.canonical);
    expect(() => canonicalJson({ a: 1.5 })).toThrow();
    expect(() => canonicalJson({ a: Number.MAX_SAFE_INTEGER + 2 })).toThrow();
  });

  it("registrace: signing input, podpis a hashcash", async () => {
    const pk = dec(registerVec.pubkey, 32);
    const nonce = dec(registerVec.nonce, 32);
    const payload = concat(nonce, utf8(registerVec.hostname), pk);
    expect(toHex(signingInput(DOMAIN.register, payload))).toBe(registerVec.signing_input_hex);
    expect(await verifySig(pk, DOMAIN.register, payload, dec(registerVec.signature, 64))).toBe(true);
    const signer = await signerFromSeed(registerVec.seed);
    expect(b64uEncode(await signer.sign(DOMAIN.register, payload))).toBe(registerVec.signature);
    const h = registerVec.hashcash;
    expect(hashcashValid(nonce, pk, h.counter, h.bits)).toBe(true);
    if (h.counter > 0) expect(hashcashValid(nonce, pk, h.counter - 1, h.bits)).toBe(false);
    expect(toHex(sha256(concat(nonce, pk, u64le(h.counter)))).length).toBe(64);
  });

  it("důkaz hostname", async () => {
    const signer = await signerFromSeed(proofVec.seed);
    const payload = concat(dec(proofVec.challenge, 32), utf8(proofVec.hostname), signer.publicKey);
    expect(await verifySig(signer.publicKey, DOMAIN.proof, payload, dec(proofVec.signature, 64))).toBe(true);
    // doménová separace: stejný podpis neplatí jako registrace
    expect(await verifySig(signer.publicKey, DOMAIN.register, payload, dec(proofVec.signature, 64))).toBe(false);
  });

  it("podepsaný request", async () => {
    const signer = await signerFromSeed(requestVec.seed);
    const payload = requestPayload(requestVec.method, requestVec.path_and_query, requestVec.t, requestVec.n, utf8(requestVec.body));
    expect(new TextDecoder().decode(payload)).toBe(requestVec.payload);
    const req = new Request(`https://x.test${requestVec.path_and_query}`, { method: requestVec.method, headers: { authorization: requestVec.authorization }, body: requestVec.body });
    const auth = await verifyNodeRequest(req, utf8(requestVec.body), requestVec.t + 5, 60);
    expect(auth.pubkeyB64).toBe(b64uEncode(signer.publicKey));
    await expect(verifyNodeRequest(req, utf8(requestVec.body), requestVec.t + 500, 60)).rejects.toThrow();
    await expect(verifyNodeRequest(req, utf8("tampered"), requestVec.t, 60)).rejects.toThrow();
  });

  it("heartbeat", async () => {
    const signer = await signerFromSeed(heartbeatVec.seed);
    expect(canonicalJson(heartbeatVec.message as never)).toBe(heartbeatVec.canonical);
    expect(await verifySig(signer.publicKey, DOMAIN.heartbeat, utf8(heartbeatVec.canonical), dec(heartbeatVec.sig, 64))).toBe(true);
  });

  it("adresář", async () => {
    const pk = dec(directoryVec.worker_pubkey, 32);
    expect(kidOf(pk)).toBe(directoryVec.kid);
    expect(canonicalJson(directoryVec.body as never)).toBe(directoryVec.canonical);
    expect(await verifySig(pk, DOMAIN.directory, utf8(directoryVec.canonical), dec(directoryVec.sig, 64))).toBe(true);
    const ids = directoryVec.body.nodes.map((n) => n.id);
    expect(ids).toEqual([...ids].sort());
  });

  it("ticket: přesný řetězec, ověření a expirace", async () => {
    const signer = await signerFromSeed(ticketVec.worker_seed);
    const t = await issueTicket(signer, ticketVec.node_id, ticketVec.ek, ticketVec.exp, dec(ticketVec.nonce, 16));
    expect(t).toBe(ticketVec.ticket);
    const pk = dec(ticketVec.worker_pubkey, 32);
    expect((await verifyTicket(pk, ticketVec.ticket, ticketVec.node_id, ticketVec.valid_at)).ok).toBe(true);
    expect((await verifyTicket(pk, ticketVec.ticket, ticketVec.node_id, ticketVec.expired_at)).ok).toBe(false);
    expect((await verifyTicket(pk, ticketVec.ticket, "other", ticketVec.valid_at)).ok).toBe(false);
  });

  it("HRW anchory", () => {
    const mailbox = dec(hrwVec.mailbox_id, 32);
    for (const c of hrwVec.cases) {
      for (const w of c.weights) {
        const weight = blake3(concat(utf8("ancord/v1/hrw"), new Uint8Array([0]), mailbox, utf8(w.id), new Uint8Array([0]), u64le(c.epoch)));
        expect(toHex(weight)).toBe(w.weight_hex);
      }
      expect(anchorsFor(mailbox, hrwVec.node_ids, c.epoch, c.r)).toEqual(c.anchors);
    }
    // při přidání nodu se anchory mění minimálně (vlastnost HRW)
    const before = anchorsFor(mailbox, hrwVec.node_ids, 41, 2);
    const after = anchorsFor(mailbox, [...hrwVec.node_ids, "zzzz"], 41, 2);
    expect(after.filter((a) => before.includes(a)).length).toBeGreaterThanOrEqual(1);
  });

  it("hash-chain", async () => {
    const seed = dec(hashchainVec.seed, 32);
    expect(computeCheckpoints(seed, hashchainVec.lanes, hashchainVec.iters, hashchainVec.every)).toEqual(hashchainVec.checkpoints);
    const all = hashchainVec.lanes * (hashchainVec.iters / hashchainVec.every);
    expect((await verifyCpu(hashchainVec.seed, hashchainVec.lanes, hashchainVec.iters, hashchainVec.every, all, hashchainVec.checkpoints)).ok).toBe(true);
  });

  it("Proof of Space: tabulka, kořen, otevření", () => {
    const seed = dec(spaceVec.seed, 32);
    expect(b64uEncode(spaceT0(seed))).toBe(spaceVec.t0);
    const table = buildTable(seed, spaceVec.leaves);
    expect(Array.from({ length: spaceVec.leaves }, (_, i) => b64uEncode(table.subarray(i * 32, (i + 1) * 32)))).toEqual(spaceVec.table);
    const levels = buildTree(table);
    expect(b64uEncode(levels[levels.length - 1]![0]!)).toBe(spaceVec.root);
    const idx = spaceVec.openings.map((o) => o.i);
    expect(verifyOpenings(seed, dec(spaceVec.root, 32), spaceVec.leaves, idx, spaceVec.openings).ok).toBe(true);
  });

  it("push: push_id a podpisy registrace a smazání", async () => {
    const signer = await signerFromSeed(pushVec.push_seed);
    expect(pushIdOf(signer.publicKey)).toBe(pushVec.push_id);
    expect(await verifySig(signer.publicKey, DOMAIN.pushReg, utf8(pushVec.register_payload), dec(pushVec.register_sig, 64))).toBe(true);
    expect(await verifySig(signer.publicKey, DOMAIN.pushReg, utf8(pushVec.delete_payload), dec(pushVec.delete_sig, 64))).toBe(true);
    expect(await verifySig(signer.publicKey, DOMAIN.pushReg, utf8(pushVec.register_payload), dec(pushVec.delete_sig, 64))).toBe(false);
  });
});
