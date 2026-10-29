#!/usr/bin/env python3
"""Referenční Double Ratchet pro spec §13.2. Nezávislý na Rustu.

Přegeneruje spec/test-vectors/ratchet.json: `python3 spec/reference/ratchet.py > spec/test-vectors/ratchet.json`
Vyžaduje balík `cryptography`.
"""
import base64, hashlib, json, struct
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey, X25519PublicKey
from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
from cryptography.hazmat.primitives.hashes import SHA256
from cryptography.hazmat.primitives.kdf.hkdf import HKDF
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

MAX_SKIP = 200


def b64u(b): return base64.urlsafe_b64encode(b).rstrip(b"=").decode()
def unb64u(s): return base64.urlsafe_b64decode(s + "=" * (-len(s) % 4))
def sha(*parts): return hashlib.sha256(b"".join(parts)).digest()
def hkdf(salt, ikm, info, n): return HKDF(algorithm=SHA256(), length=n, salt=salt, info=info).derive(ikm)
def xkey(seed): return X25519PrivateKey.from_private_bytes(seed)
def xpub(priv): return priv.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
def dh(priv, pub): return priv.exchange(X25519PublicKey.from_public_bytes(pub))


def mailbox_id(ss, d): return sha(b"ancord/v1/mailbox\0", ss, b"\0", d.encode())
def kdf_rk(rk, d):
    okm = hkdf(rk, d, b"ancord/v1/ratchet/root", 96)
    return okm[:32], okm[32:64], okm[64:]
def header_key0(ss, d): return hkdf(b"ancord/v1/ratchet", ss, b"header/" + d.encode(), 32)
def header_nonce(mid): return sha(b"ancord/v1/hdr\0", mid)[:12]
def header_aad(mbox, mid): return b"ancord/v1/hdr\0" + mbox + mid
def open_header(hk, mbox, mid, enc):
    try:
        h = ChaCha20Poly1305(hk).decrypt(header_nonce(mid), enc, header_aad(mbox, mid))
    except Exception:
        return None
    return h[:32], struct.unpack(">III", h[32:])
def msg_key(ck):
    mk = hkdf(None, ck, b"ancord/v1/ratchet/msg", 44)
    return mk[:32], mk[32:]
def next_ck(ck): return hkdf(None, ck, b"ancord/v1/ratchet/chain", 32)
def bootstrap_seed(ss): return hkdf(b"ancord/v1/ratchet", ss, b"initial-dh", 32)


class Party:
    """Jedna strana konverzace; `seeds` jsou tajné klíče, které strana postupně použije jako nové ratchet páry."""

    def __init__(self, ss, initiator, seeds, peer_pub=None, own_priv=None):
        self.ss = ss
        self.send_dir, self.recv_dir = ("i2r", "r2i") if initiator else ("r2i", "i2r")
        self.seeds = list(seeds)
        self.rk = hkdf(b"ancord/v1/ratchet", ss, b"root", 32)
        self.dhr = self.cks = self.ckr = None
        self.ns = self.nr = self.pn = self.seq = 0
        self.skipped = {}
        self.hks = self.hkr = None
        self.nhks, self.nhkr = header_key0(ss, self.send_dir), header_key0(ss, self.recv_dir)
        if initiator:
            self.dhs = xkey(self.seeds.pop(0))
            self.dhr = peer_pub
            self.hks = self.nhks
            self.rk, self.cks, self.nhks = kdf_rk(self.rk, dh(self.dhs, peer_pub))
        else:
            self.dhs = own_priv

    def seal(self, mid, plaintext):
        key, nonce = msg_key(self.cks)
        mbox = mailbox_id(self.ss, self.send_dir)
        header = xpub(self.dhs) + struct.pack(">III", self.pn, self.ns, self.seq)
        enc_header = ChaCha20Poly1305(self.hks).encrypt(header_nonce(mid), header, header_aad(mbox, mid))
        aad = b"ancord/v1/msg\0" + mbox + mid + enc_header
        body = ChaCha20Poly1305(key).encrypt(nonce, plaintext, aad)
        out = dict(key_hex=key.hex(), nonce_hex=nonce.hex(), aad_hex=aad.hex(), header_hex=header.hex(),
                   header_key_hex=self.hks.hex(), header_nonce_hex=header_nonce(mid).hex())
        self.cks, self.ns, self.seq = next_ck(self.cks), self.ns + 1, self.seq + 1
        return b64u(enc_header + body), out

    def skip(self, until):
        if self.ckr is None:
            return
        assert until - self.nr <= MAX_SKIP
        while self.nr < until:
            self.skipped[(self.hkr, self.nr)] = msg_key(self.ckr)
            self.ckr, self.nr = next_ck(self.ckr), self.nr + 1

    def open(self, mid, ct):
        wire = unb64u(ct)
        enc, body = wire[:60], wire[60:]
        mbox = mailbox_id(self.ss, self.recv_dir)
        aad = b"ancord/v1/msg\0" + mbox + mid + enc
        # 1) klíče hlaviček uložených přeskočených zpráv
        for hk in {hk for hk, _ in self.skipped}:
            h = open_header(hk, mbox, mid, enc)
            if h and (hk, h[1][1]) in self.skipped:
                key, nonce = self.skipped.pop((hk, h[1][1]))
                return ChaCha20Poly1305(key).decrypt(nonce, body, aad), h[1][2]
        # 2) aktuální přijímací řetěz, 3) další řetěz = DH krok
        h = open_header(self.hkr, mbox, mid, enc) if self.hkr else None
        if h is None:
            h = open_header(self.nhkr, mbox, mid, enc)
            assert h is not None
            self.skip(h[1][0])
            self.pn, self.ns, self.nr, self.dhr = self.ns, 0, 0, h[0]
            self.hks, self.hkr = self.nhks, self.nhkr
            self.rk, self.ckr, self.nhkr = kdf_rk(self.rk, dh(self.dhs, self.dhr))
            self.dhs = xkey(self.seeds.pop(0))
            self.rk, self.cks, self.nhks = kdf_rk(self.rk, dh(self.dhs, self.dhr))
        _, (pn, n, seq) = h
        assert n >= self.nr
        self.skip(n)
        key, nonce = msg_key(self.ckr)
        self.ckr, self.nr = next_ck(self.ckr), n + 1
        return ChaCha20Poly1305(key).decrypt(nonce, body, aad), seq


def scenario(name, ss, initiator_peer_pub, responder_priv, a_seeds, b_seeds, script):
    """`script`: ("a"|"b", "send", text) nebo ("a"|"b", "recv", index odeslané zprávy). Příjem v pořadí skriptu."""
    parties = {
        "a": Party(ss, True, a_seeds, peer_pub=initiator_peer_pub),
        "b": Party(ss, False, b_seeds, own_priv=responder_priv),
    }
    sent, steps = [], []
    for who, op, arg in script:
        p = parties[who]
        if op == "send":
            mid = sha(f"{name}-message-{len(sent)}".encode())[:16]
            ct, dbg = p.seal(mid, arg.encode())
            sent.append((who, mid, ct))
            steps.append({"who": who, "op": "send", "id": b64u(mid), "plaintext_b64": b64u(arg.encode()), "ct": ct, **dbg})
        else:
            sender, mid, ct = sent[arg]
            assert sender != who
            plain, seq = p.open(mid, ct)
            steps.append({"who": who, "op": "recv", "message": arg, "plaintext_b64": b64u(plain), "seq": seq})
    return {"name": name, "steps": steps}


ss = sha(b"pairing-secret")
spk_b = xkey(sha(b"responder-first-ratchet-key"))
seeds = lambda who, k: [sha(f"{who}-ratchet-{i}".encode()) for i in range(k)]

# 1) spk jako první ratchet klíč (X3DH), pořadí zpráv: plná obousměrná výměna, DH kroky v obou směrech
basic = scenario(
    "basic", ss, xpub(spk_b), spk_b, seeds("a", 4), seeds("b", 4),
    [("a", "send", "ahoj"), ("a", "send", ""), ("b", "recv", 0), ("b", "recv", 1),
     ("b", "send", "odpověď 1"), ("b", "send", "odpověď 2"), ("a", "recv", 2), ("a", "recv", 3),
     ("a", "send", "zpráva č. 3 \U0001F512"), ("b", "recv", 4),
     ("b", "send", "odpověď 3"), ("a", "recv", 5)],
)

# 2) vyřazení z pořadí, zpráva z předchozího řetězu doručená po DH kroku, přeskočené klíče přes hranici řetězů
disorder = scenario(
    "disorder", ss, xpub(spk_b), spk_b, seeds("a", 4), seeds("b", 4),
    [("a", "send", "m0"), ("a", "send", "m1"), ("a", "send", "m2"),
     ("b", "recv", 2), ("b", "recv", 0),
     ("b", "send", "r0"), ("b", "send", "r1"), ("a", "recv", 4), ("a", "recv", 3),
     ("a", "send", "m3"), ("a", "send", "m4"),
     ("b", "recv", 6), ("b", "recv", 1), ("b", "recv", 5)],
)

# 3) ručně sdílené tajemství: první ratchet klíč responderu je odvozený z `ss`
boot = xkey(bootstrap_seed(ss))
shared = scenario(
    "shared-secret", ss, xpub(boot), boot, seeds("a", 3), seeds("b", 3),
    [("a", "send", "ahoj"), ("b", "recv", 0), ("b", "send", "čau"), ("a", "recv", 1), ("a", "send", "zpět"), ("b", "recv", 2)],
)

print(json.dumps({
    "description": "Double Ratchet se šifrovanou hlavičkou (spec §13.2): X25519 + HKDF-SHA256 + ChaCha20-Poly1305, nezávislý Python (spec/reference/ratchet.py)",
    "header_keys": {d: b64u(header_key0(ss, d)) for d in ("i2r", "r2i")},
    "shared_secret": b64u(ss),
    "max_skip": MAX_SKIP,
    "mailboxes": {d: b64u(mailbox_id(ss, d)) for d in ("i2r", "r2i")},
    "bootstrap_seed": b64u(bootstrap_seed(ss)),
    "responder_ratchet_seed": b64u(sha(b"responder-first-ratchet-key")),
    "key_seeds": {
        "description": "Nové ratchet páry, které strana postupně použije (iniciátor první hned při založení, ostatní při DH kroku)",
        "a": [b64u(s) for s in seeds("a", 4)], "b": [b64u(s) for s in seeds("b", 4)],
    },
    "scenarios": [basic, disorder, shared],
}, indent=2, ensure_ascii=False))
