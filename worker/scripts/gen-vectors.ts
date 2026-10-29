/**
 * Vygeneruje spec/test-vectors/*.json z referenčních funkcí Workeru.
 * Spuštění: npm run vectors. Výsledek je v gitu; test/vectors.test.ts hlídá, že ho kód stále splňuje.
 * Vektory musí projít i node (Rust) – proto jsou vstupy pevné a podpisy deterministické (Ed25519).
 */
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { canonicalJson } from "../src/canonical";
import { DOMAIN, blake3, hostnameOf, kidOf, nodeIdOf, sha256, signerFromSeed, signingInput } from "../src/crypto";
import { b64uEncode, concat, toHex, u64le, utf8 } from "../src/encoding";
import { requestPayload } from "../src/auth";
import { solveHashcash } from "../src/hashcash";
import { anchorsFor } from "../src/hrw";
import { issueTicket } from "../src/ticket";
import { computeCheckpoints } from "../src/challenge/cpu";
import { buildTable, buildTree, makeOpening, spaceT0 } from "../src/challenge/space";
import { pushIdOf } from "../src/push/handler";

const out = join(dirname(fileURLToPath(import.meta.url)), "../../spec/test-vectors");
mkdirSync(out, { recursive: true });

const PKCS8 = new Uint8Array([0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20]);
const seedOf = (n: number) => new Uint8Array(32).fill(n);
const ZONE = "melonet.cz";

async function keyOf(seed: Uint8Array) {
  const s = await signerFromSeed(b64uEncode(seed));
  return { seed: b64uEncode(seed), pub: s.publicKey, signer: s };
}

function write(name: string, data: unknown): void {
  writeFileSync(join(out, name), JSON.stringify(data, null, 2) + "\n");
  console.log("wrote", name);
}

async function main() {
  void PKCS8;
  const keys = await Promise.all([1, 2, 3].map((n) => keyOf(seedOf(n))));

  write("identity.json", {
    description: "node_id = base32(blake3(pubkey)); hostname = 'node' + node_id[0..16] + '.' + zone",
    zone: ZONE,
    cases: keys.map((k) => ({
      seed: k.seed,
      pubkey: b64uEncode(k.pub),
      node_id: nodeIdOf(k.pub),
      hostname: hostnameOf(nodeIdOf(k.pub), ZONE),
    })),
  });

  const canonCases: unknown[] = [
    { b: 1, a: [true, null, "x"], c: { z: 1, y: 2 } },
    { "é": "ž\n\"\\", n: -5, big: 9007199254740991 },
    [],
    {},
    "plain",
  ];
  write("canonical-json.json", {
    description: "RFC 8785 omezené na celá čísla (spec §1.2)",
    cases: canonCases.map((input) => ({ input, canonical: canonicalJson(input as never) })),
  });

  const k1 = keys[0]!;
  const nonce = sha256(utf8("register-nonce"));
  const hostname = hostnameOf(nodeIdOf(k1.pub), ZONE);
  const regPayload = concat(nonce, utf8(hostname), k1.pub);
  const regSig = await k1.signer.sign(DOMAIN.register, regPayload);
  const hcBits = 12;
  write("register.json", {
    description: "Podpis potvrzení registrace a hashcash (spec §4.2)",
    seed: k1.seed,
    pubkey: b64uEncode(k1.pub),
    hostname,
    nonce: b64uEncode(nonce),
    signing_input_hex: toHex(signingInput(DOMAIN.register, regPayload)),
    signature: b64uEncode(regSig),
    hashcash: {
      bits: hcBits,
      counter: solveHashcash(nonce, k1.pub, hcBits),
      note: "sha256(nonce ‖ pubkey ‖ counter_le_u64) má >= bits úvodních nulových bitů; counter je nejmenší platný",
    },
  });

  const proofChallenge = sha256(utf8("proof-challenge"));
  const proofPayload = concat(proofChallenge, utf8(hostname), k1.pub);
  write("proof.json", {
    description: "Důkaz držení klíče na hostname (spec §4.3)",
    seed: k1.seed,
    hostname,
    challenge: b64uEncode(proofChallenge),
    signature: b64uEncode(await k1.signer.sign(DOMAIN.proof, proofPayload)),
  });

  const body = utf8('{"push_id":"x"}');
  const t = 1760000000;
  const n = b64uEncode(sha256(utf8("req-nonce")).slice(0, 16));
  const reqPayload = requestPayload("POST", "/wake?x=1", t, n, body);
  const reqSig = await k1.signer.sign(DOMAIN.req, reqPayload);
  write("request-auth.json", {
    description: "Podepsaný HTTP request (spec §3)",
    seed: k1.seed,
    method: "POST",
    path_and_query: "/wake?x=1",
    body: new TextDecoder().decode(body),
    t,
    n,
    payload: new TextDecoder().decode(reqPayload),
    authorization: `Ancord-Ed25519 k=${b64uEncode(k1.pub)},t=${t},n=${n},s=${b64uEncode(reqSig)}`,
  });

  const hb = { t: "hb", seq: 7, ts: 1760000000, epoch: 41, load: { conns: 12, max_conns: 2000, cpu_pct: 8, mem_pct: 31 } };
  write("heartbeat.json", {
    description: "Heartbeat: sig přes kanonické JSON bez pole sig (spec §5)",
    seed: k1.seed,
    message: hb,
    canonical: canonicalJson(hb),
    sig: b64uEncode(await k1.signer.sign(DOMAIN.heartbeat, utf8(canonicalJson(hb)))),
  });

  const worker = await keyOf(seedOf(9));
  const dirBody = {
    v: 1,
    epoch: 41,
    epoch_seconds: 300,
    issued_at: 41 * 300,
    params: { r: 3, quorum: 2 },
    nodes: keys.map((k) => ({ id: nodeIdOf(k.pub), pk: b64uEncode(k.pub), host: hostnameOf(nodeIdOf(k.pub), ZONE) })).sort((a, b) => (a.id < b.id ? -1 : 1)),
  };
  write("directory.json", {
    description: "Podepsaný adresář (spec §6)",
    worker_seed: worker.seed,
    worker_pubkey: b64uEncode(worker.pub),
    kid: kidOf(worker.pub),
    body: dirBody,
    canonical: canonicalJson(dirBody),
    sig: b64uEncode(await worker.signer.sign(DOMAIN.directory, utf8(canonicalJson(dirBody)))),
  });

  const ek = b64uEncode(keys[1]!.pub);
  const ticketNonce = sha256(utf8("ticket-nonce")).slice(0, 16);
  const ticket = await issueTicket(worker.signer, nodeIdOf(k1.pub), ek, 1760000120, ticketNonce);
  write("ticket.json", {
    description: "Ticket (spec §8)",
    worker_seed: worker.seed,
    worker_pubkey: b64uEncode(worker.pub),
    node_id: nodeIdOf(k1.pub),
    ek,
    exp: 1760000120,
    nonce: b64uEncode(ticketNonce),
    ticket,
    valid_at: 1760000000,
    expired_at: 1760000120,
  });

  const mailbox = sha256(utf8("mailbox-1"));
  const ids = keys.map((k) => nodeIdOf(k.pub));
  write("hrw.json", {
    description: "Anchory mailboxu (spec §7)",
    mailbox_id: b64uEncode(mailbox),
    node_ids: ids,
    cases: [41, 42, 43].map((epoch) => ({
      epoch,
      r: 2,
      weights: ids.map((id) => ({
        id,
        weight_hex: toHex(blake3(concat(utf8("ancord/v1/hrw"), new Uint8Array([0]), mailbox, utf8(id), new Uint8Array([0]), u64le(epoch)))),
      })),
      anchors: anchorsFor(mailbox, ids, epoch, 2),
    })),
  });

  const cpuSeed = sha256(utf8("cpu-seed"));
  write("hashchain.json", {
    description: "CPU hash-chain (spec §9.1): h0 = sha256(seed ‖ lane_le_u32), h_{i+1} = sha256(h_i)",
    seed: b64uEncode(cpuSeed),
    lanes: 2,
    iters: 1000,
    every: 250,
    checkpoints: computeCheckpoints(cpuSeed, 2, 1000, 250),
  });

  const spaceSeed = sha256(utf8("space-seed"));
  const leaves = 16;
  const table = buildTable(spaceSeed, leaves);
  const levels = buildTree(table);
  write("space.json", {
    description: "Proof of Space (spec §9.2): tabulka, Merkle kořen a otevření",
    seed: b64uEncode(spaceSeed),
    leaves,
    t0: b64uEncode(spaceT0(spaceSeed)),
    table: Array.from({ length: leaves }, (_, i) => b64uEncode(table.subarray(i * 32, (i + 1) * 32))),
    root: b64uEncode(levels[levels.length - 1]![0]!),
    openings: [3, 9, 15].map((i) => makeOpening(spaceSeed, table, levels, i)),
  });

  // ratchet.json a pairwise.json generují Pythonové reference v spec/reference/ (X25519 a Double Ratchet)

  const pushKey = await keyOf(seedOf(5));
  const pushId = pushIdOf(pushKey.pub);
  const regBody = { op: "register", platform: "fcm", push_id: pushId, token: "device-token-123", ts: 1760000000 };
  write("push.json", {
    description: "Registrace push tokenu (spec §10.1)",
    push_seed: pushKey.seed,
    push_pubkey: b64uEncode(pushKey.pub),
    push_id: pushId,
    register_payload: canonicalJson(regBody),
    register_sig: b64uEncode(await pushKey.signer.sign(DOMAIN.pushReg, utf8(canonicalJson(regBody)))),
    delete_payload: canonicalJson({ op: "delete", push_id: pushId, ts: 1760000000 }),
    delete_sig: b64uEncode(await pushKey.signer.sign(DOMAIN.pushReg, utf8(canonicalJson({ op: "delete", push_id: pushId, ts: 1760000000 })))),
  });
}

await main();
